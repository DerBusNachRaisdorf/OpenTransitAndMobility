use std::{fmt::Write as _, future::Future};

use public_transport::database::DatabaseError;
use sqlx::{
    postgres::{PgArguments, PgQueryResult, PgRow},
    query::{Query, QueryAs},
    Error, Executor, FromRow, Postgres,
};

pub mod agency;
pub mod collector;
pub mod line;
pub mod origin;
pub mod service;
pub mod shape;
pub mod shared_mobility;
pub mod stop;
pub mod trip;
pub mod trip_update;

// TODO: replace `RETURNING *` to explicitly specify column names in all queries.

pub(crate) fn convert_error(why: sqlx::Error) -> DatabaseError {
    match why {
        sqlx::Error::RowNotFound => DatabaseError::NotFound,
        _ => DatabaseError::Other(Box::new(why)),
    }
}

// bulk insert

pub async fn insert_all_returning<'c, E, T, B, O>(
    executor: E,
    table: &str,
    columns: &[&str],
    values: &[T],
    bind: B,
    conflict_set: &[&str],
) -> Result<Vec<O>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
    for<'a> B: Fn(
        QueryAs<'a, Postgres, O, PgArguments>,
        &T,
    ) -> QueryAs<'a, Postgres, O, PgArguments>,
    for<'r> O: FromRow<'r, PgRow> + Send + Unpin,
{
    // build query string
    let mut query_str =
        format!("INSERT INTO {} ({}) VALUES ", table, columns.join(", "));
    let mut placeholder_index = 1;
    for i in 0..values.len() {
        if i > 0 {
            query_str.push_str(", ");
        }
        query_str.push('(');
        for j in 0..columns.len() {
            if j > 0 {
                query_str.push_str(", ");
            }
            write!(&mut query_str, "${}", placeholder_index).unwrap();
            placeholder_index += 1;
        }
        query_str.push(')');
    }
    if conflict_set.len() != 0 {
        write!(
            &mut query_str,
            " ON CONFLICT ({}) DO UPDATE SET {}",
            conflict_set.join(", "),
            columns
                .iter()
                .filter(|column| !conflict_set.contains(column))
                .map(|column| format!("{} = EXCLUDED.{}", column, column))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    let _ = write!(&mut query_str, " RETURNING {}", columns.join(", "));
    query_str.push_str(";");

    //println!("query: {}", query_str);

    // query
    let mut query = sqlx::query_as::<Postgres, O>(&query_str);
    for value in values {
        query = bind(query, value);
    }
    query.fetch_all(executor).await
}

pub async fn insert_all<'c, E, T, B>(
    executor: E,
    columns: &[&str],
    values: &[T],
    bind: B,
    conflict_set: &[&str],
) -> Result<PgQueryResult, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
    for<'a> B:
        Fn(Query<'a, Postgres, PgArguments>, &T) -> Query<'a, Postgres, PgArguments>,
{
    // build query string
    let mut query_str = format!("INSERT INTO({}) VALUES ", columns.join(", "));
    let mut placeholder_index = 1;
    for i in 0..values.len() {
        if i > 0 {
            query_str.push_str(", ");
        }
        query_str.push('(');
        for j in 0..columns.len() {
            if j > 0 {
                query_str.push_str(", ");
            }
            write!(&mut query_str, "${}", placeholder_index).unwrap();
            placeholder_index += 1;
        }
        query_str.push(')');
    }
    if conflict_set.len() != 0 {
        write!(
            &mut query_str,
            "ON CONFLICT ({}) DO UPDATE SET {}",
            conflict_set.join(", "),
            conflict_set
                .iter()
                .map(|column| format!("{} = EXCLUDED.{}", column, column))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    query_str.push_str(";");

    // query
    let mut query = sqlx::query::<Postgres>(&query_str);
    for value in values {
        query = bind(query, value);
    }
    query.execute(executor).await
}

// sql framework

const MAX_CHUNK_SIZE: usize = 100;

pub struct InsertInto<'a> {
    table: &'a str,
    columns: &'a [&'a str],
}

impl<'a> InsertInto<'a> {
    pub fn values<V>(self, values: &'a [&'a V]) -> MultiRowInsert<'a, V> {
        MultiRowInsert {
            insert: self,
            values,
        }
    }
}

pub struct MultiRowInsert<'a, V> {
    insert: InsertInto<'a>,
    values: &'a [&'a V],
}

impl<'a, V> MultiRowInsert<'a, V> {
    pub fn binder<F: FnMut(PgArguments, &V) -> PgArguments>(
        self,
        binder: F,
    ) -> BoundMultiRowInsert<'a, V, F> {
        BoundMultiRowInsert {
            insert: self,
            binder,
        }
    }
}

pub struct BoundMultiRowInsert<'a, V, F>
where
    F: FnMut(PgArguments, &V) -> PgArguments,
{
    insert: MultiRowInsert<'a, V>,
    binder: F,
}

impl<'a, V, F> BoundMultiRowInsert<'a, V, F>
where
    F: FnMut(PgArguments, &V) -> PgArguments,
{
    pub async fn execute<'c, X, R>(
        mut self,
        mut execute_callback: X,
    ) -> Result<u64, Error>
    where
        X: FnMut(Query<Postgres, PgArguments>) -> R,
        R: Future<Output = Result<u64, Error>> + Send,
    {
        for chunk in self.insert.values.chunks(MAX_CHUNK_SIZE) {
            let mut query = format!(
                "INSERT INTO {} ({}) VALUES ",
                self.insert.insert.table,
                self.insert.insert.columns.join(", ")
            );
            let mut args = PgArguments::default();
            let mut placeholder_index = 1;

            for (i, value) in chunk.iter().enumerate() {
                args = (self.binder)(args, value);

                if i > 0 {
                    query.push_str(", ");
                }
                query.push('(');
                for j in 0..self.insert.insert.columns.len() {
                    if j > 0 {
                        query.push_str(", ");
                    }
                    write!(&mut query, "${}", placeholder_index).unwrap();
                    placeholder_index += 1;
                }
                query.push(')');
            }

            let stmt = sqlx::query_with(&query, args);
            execute_callback(stmt).await?;
        }
        todo!()
    }
}
