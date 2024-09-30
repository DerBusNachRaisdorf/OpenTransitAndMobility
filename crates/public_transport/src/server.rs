use model::{origin::Origin, WithId};
use utility::id::Id;

use crate::{
    client::Client,
    collector::{self, Collector, CollectorInstance},
    database::{CollectorRepo, Database, DatabaseOperations},
    RequestResult,
};

pub struct Server<D>
where
    D: Database + Send + Sync + Sized + 'static,
{
    database: D,
}

impl<D> Server<D>
where
    D: Database,
{
    pub fn new(database: D) -> Self {
        Self { database }
    }

    pub fn client<S: Into<String>>(&self, id: S) -> Client<D> {
        Client::new(id, self.database.clone())
    }

    pub async fn origin<S: Into<String>>(
        &self,
        name: S,
        priority: i32,
    ) -> RequestResult<Id<Origin>> {
        let name: String = name.into();
        let id = Id::new(name.to_lowercase().replace(' ', "-").replace('.', "-"));
        self.database
            .auto()
            .put_origin(WithId::new(id.clone(), Origin { name, priority }))
            .await?;
        Ok(id)
    }

    pub async fn collector<C, F>(
        &self,
        id: &Id<CollectorInstance<C>>,
        origin: &Id<Origin>,
        factory: F,
    ) where
        C: Collector + Send + 'static,
        <C as Collector>::Error: Send,
        F: 'static + Send + Fn(C::State) -> C,
    {
        let client = self.client(origin.clone().raw());
        collector::run(factory, client, id.clone()).await;
    }

    pub async fn collectors<C: Collector + 'static>(&self) -> RequestResult<()>
    where
        C: Collector + Send + 'static,
        <C as Collector>::Error: Send,
    {
        let instances = self.database.auto().collectors::<C>().await?;
        for instance in instances {
            self.collector(&instance.id, &instance.content.origin, C::from_state)
                .await
        }
        Ok(())
    }
}
