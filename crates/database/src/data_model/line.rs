use crate::{
    queries::line::{
        exists, exists_with_origin, get, get_all, get_by_name_and_agency, get_by_stop_id,
        id_by_original_id, insert, put, put_original_id, update,
    },
    PgDatabaseTransaction,
};
use async_trait::async_trait;
use model::{
    agency::Agency,
    line::{Line, LineType},
    origin::{Origin, OriginalIdMapping},
    stop::Stop,
    DatabaseEntry, WithId, WithOrigin,
};
use public_transport::database::{LineRepo, Repo, Result, SubjectRepo};
use sqlx::prelude::FromRow;
use utility::id::{Id, IdWrapper};

use crate::PgDatabaseAutocommit;

use super::DatabaseRow;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "line_type", rename_all = "snake_case")]
pub enum RowLineType {
    TramStreetcarOrLightrail,
    SubwayOrMetro,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLiftOrSuspendedCableCar,
    Funicular,
    Trolleybus,
    Monorail,
}

impl RowLineType {
    pub fn to_line_type(self) -> LineType {
        match self {
            Self::TramStreetcarOrLightrail => LineType::TramStreetcarOrLighrail,
            Self::SubwayOrMetro => LineType::SubwayOrMetro,
            Self::Rail => LineType::Rail,
            Self::Bus => LineType::Bus,
            Self::Ferry => LineType::Ferry,
            Self::CableTram => LineType::CableTram,
            Self::AerialLiftOrSuspendedCableCar => LineType::AerialLiftOrSuspendedCableCar,
            Self::Funicular => LineType::Funicular,
            Self::Trolleybus => LineType::Trolleybus,
            Self::Monorail => LineType::Monorail,
        }
    }

    pub fn from_line_type(kind: LineType) -> Self {
        match kind {
            LineType::TramStreetcarOrLighrail => Self::TramStreetcarOrLightrail,
            LineType::SubwayOrMetro => Self::SubwayOrMetro,
            LineType::Rail => Self::Rail,
            LineType::Bus => Self::Bus,
            LineType::Ferry => Self::Ferry,
            LineType::CableTram => Self::CableTram,
            LineType::AerialLiftOrSuspendedCableCar => Self::AerialLiftOrSuspendedCableCar,
            LineType::Funicular => Self::Funicular,
            LineType::Trolleybus => Self::Trolleybus,
            LineType::Monorail => Self::Monorail,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct LineRow {
    pub id: String,
    pub origin: String,
    pub name: Option<String>,
    pub kind: RowLineType,
    pub agency_id: Option<String>,
}

impl DatabaseRow for LineRow {
    type Model = Line;

    fn get_id(&self) -> Id<Self::Model> {
        Id::new(self.id.clone())
    }

    fn get_origin(&self) -> Id<Origin> {
        Id::new(self.origin.clone())
    }

    fn to_model(self) -> Self::Model {
        Line {
            name: self.name,
            kind: self.kind.to_line_type(),
            agency_id: self.agency_id.map(|inner| Id::new(inner)),
        }
    }

    fn from_model(line: WithOrigin<Self::Model>) -> Self {
        Self {
            id: "".to_owned(),
            origin: line.origin.raw(),
            name: line.content.name,
            kind: RowLineType::from_line_type(line.content.kind),
            agency_id: line.content.agency_id.raw(),
        }
    }
}

// Repo

#[async_trait]
impl Repo<Line> for PgDatabaseAutocommit {
    async fn get(&mut self, id: Id<Line>) -> Result<DatabaseEntry<Line>> {
        get(&self.pool, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Line>>> {
        get_all(&self.pool).await
    }

    async fn insert(&mut self, element: WithOrigin<Line>) -> Result<WithOrigin<WithId<Line>>> {
        insert(&self.pool, element).await
    }

    async fn put(&mut self, element: WithOrigin<WithId<Line>>) -> Result<WithOrigin<WithId<Line>>> {
        put(&self.pool, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Line>>,
    ) -> Result<WithOrigin<WithId<Line>>> {
        update(&self.pool, element).await
    }

    async fn exists(&mut self, id: Id<Line>) -> Result<bool> {
        exists(&self.pool, id).await
    }

    async fn exists_with_origin(&mut self, id: Id<Line>, origin: Id<Origin>) -> Result<bool> {
        exists_with_origin(&self.pool, id, origin).await
    }
}

#[async_trait]
impl<'a> Repo<Line> for PgDatabaseTransaction<'a> {
    async fn get(&mut self, id: Id<Line>) -> Result<DatabaseEntry<Line>> {
        get(&mut *self.tx, id).await
    }

    async fn get_all(&mut self) -> Result<Vec<DatabaseEntry<Line>>> {
        get_all(&mut *self.tx).await
    }

    async fn insert(&mut self, element: WithOrigin<Line>) -> Result<WithOrigin<WithId<Line>>> {
        insert(&mut *self.tx, element).await
    }

    async fn put(&mut self, element: WithOrigin<WithId<Line>>) -> Result<WithOrigin<WithId<Line>>> {
        put(&mut *self.tx, element).await
    }

    async fn update(
        &mut self,
        element: WithOrigin<WithId<Line>>,
    ) -> Result<WithOrigin<WithId<Line>>> {
        update(&mut *self.tx, element).await
    }

    async fn exists(&mut self, id: Id<Line>) -> Result<bool> {
        exists(&mut *self.tx, id).await
    }

    async fn exists_with_origin(&mut self, id: Id<Line>, origin: Id<Origin>) -> Result<bool> {
        exists_with_origin(&mut *self.tx, id, origin).await
    }
}

// Subject Repo

#[async_trait]
impl SubjectRepo<Line> for PgDatabaseAutocommit {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Line>>> {
        id_by_original_id(&self.pool, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Line>,
    ) -> Result<OriginalIdMapping<Line>> {
        put_original_id(&self.pool, origin, original_id, id).await
    }
}

#[async_trait]
impl<'a> SubjectRepo<Line> for PgDatabaseTransaction<'a> {
    async fn id_by_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
    ) -> Result<Option<Id<Line>>> {
        id_by_original_id(&mut *self.tx, origin, original_id).await
    }

    async fn put_original_id(
        &mut self,
        origin: Id<Origin>,
        original_id: String,
        id: Id<Line>,
    ) -> Result<OriginalIdMapping<Line>> {
        put_original_id(&mut *self.tx, origin, original_id, id).await
    }
}

// Line Repo

#[async_trait]
impl LineRepo for PgDatabaseAutocommit {
    async fn line_by_name_and_agency<S: Into<String> + Send>(
        &mut self,
        name: S,
        agency: &Id<Agency>,
    ) -> Result<Vec<DatabaseEntry<Line>>> {
        get_by_name_and_agency(&self.pool, name, agency).await
    }

    async fn get_by_stop_id(&mut self, stop_id: &Id<Stop>) -> Result<Vec<DatabaseEntry<Line>>> {
        // TODO: make underlying function take stop_id by ref.
        get_by_stop_id(&self.pool, stop_id.clone()).await
    }
}

#[async_trait]
impl<'a> LineRepo for PgDatabaseTransaction<'a> {
    async fn line_by_name_and_agency<S: Into<String> + Send>(
        &mut self,
        name: S,
        agency: &Id<Agency>,
    ) -> Result<Vec<DatabaseEntry<Line>>> {
        get_by_name_and_agency(&mut *self.tx, name, agency).await
    }

    async fn get_by_stop_id(&mut self, stop_id: &Id<Stop>) -> Result<Vec<DatabaseEntry<Line>>> {
        // TODO: make underlying function take stop_id by ref.
        get_by_stop_id(&mut *self.tx, stop_id.clone()).await
    }
}
