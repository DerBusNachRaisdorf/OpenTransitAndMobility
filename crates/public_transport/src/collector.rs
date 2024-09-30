use futures::FutureExt;
use model::origin::Origin;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use std::panic::AssertUnwindSafe;
use std::time::Duration;
use tokio::time::{self, sleep};
use utility::id::{HasId, Id};

use async_trait::async_trait;
use chrono::{DateTime, Local};

use crate::{
    client::Client,
    database::{CollectorRepo, Database},
};

pub struct CollectorInstance<C: Collector> {
    pub origin: Id<Origin>,
    pub is_active: bool,
    pub state: C::State,
}

impl<C> HasId for CollectorInstance<C>
where
    C: Collector,
{
    type IdType = i32;
}

#[derive(Clone)]
pub enum Continuation {
    ContinueAfter(Duration),
    ContinueAt(DateTime<Local>),
    Continue,
    Restart,
    Exit,
}

#[derive(Clone)]
pub enum SupervisionStrategy {
    Restart,
    Resume,
    Stop,
}

#[async_trait]
pub trait Collector {
    type Error: Debug;
    type State: Debug
        + Clone
        + Serialize
        + for<'a> Deserialize<'a>
        + Send
        + Sync
        + Unpin
        + 'static;

    /// This string should uniquely identify the collector. It is used to associate
    /// collector settings stored in the database with a collector implementation.
    /// If two collectors appear to have the same value for this,
    /// it will cause a crash.
    /// This string must also never change.
    fn unique_id() -> &'static str;

    /// Creates a new instance of the collector from a given state.
    /// Usually, this state is loaded from the database.
    fn from_state(state: Self::State) -> Self;

    /// This method is regularly called and supposed to gahter data and push
    /// it to the database.
    async fn run<D: Database>(
        &mut self,
        client: &Client<D>,
        state: Self::State,
    ) -> Result<(Continuation, Self::State), Self::Error>;

    /// Specifies how long to wait between calls to the `run` method.
    fn tick(&self) -> Option<Duration> {
        Some(Duration::from_secs(10))
    }

    /// Defines a backoff function, used to progressively increase the waiting
    /// time when consecutive failures happen.
    fn backoff(&self, last_backoff: Duration) -> Duration {
        last_backoff + self.tick().unwrap_or(Duration::from_secs(10))
    }

    /// Specifies the behavior if the collector returns an error.
    fn on_error(&self, _error: Self::Error) -> SupervisionStrategy {
        SupervisionStrategy::Resume
    }

    /// Specifies the behavior if the collector panics.
    fn on_panic(&self, _error: Box<dyn Any + Send>) -> SupervisionStrategy {
        SupervisionStrategy::Restart
    }
}

pub struct CollectorRef;

async fn run_persistent<'a, D, C>(
    id: Id<CollectorInstance<C>>,
    collector: &mut C,
    client: &'a Client<D>,
) -> Result<Continuation, C::Error>
where
    D: Database,
    C: Collector + 'static,
{
    let data = client.database.auto().get_collector(&id).await.unwrap();
    if !data.is_active {
        return Ok(Continuation::Exit);
    }
    let result = collector.run(client, data.state).await;
    match result {
        Ok((continuation, new_state)) => {
            client
                .database
                .auto()
                .set_collector_state(&id, new_state)
                .await
                .unwrap();
            Ok(continuation)
        }
        Err(why) => Err(why),
    }
}

pub async fn run<D, C, F>(
    factory: F,
    client: Client<D>,
    id: Id<CollectorInstance<C>>,
) -> CollectorRef
where
    D: Database,
    C: Collector + Send + 'static,
    <C as Collector>::Error: Send,
    F: 'static + Send + Fn(C::State) -> C,
{
    let instance = client.database.auto().get_collector(&id).await.unwrap(); // TODO!
    let mut collector = factory(instance.state);

    // run actor
    tokio::spawn(async move {
        let mut interval = collector.tick().map(|tick| time::interval(tick));
        let mut backoff = collector.tick().unwrap_or(Duration::from_secs(10));
        loop {
            // run
            let result =
                AssertUnwindSafe(run_persistent(id, &mut collector, &client))
                    .catch_unwind()
                    .await;
            // check for errors
            let mut result = match result {
                Ok(Ok(continuation)) => Ok(continuation),
                Ok(Err(why)) => {
                    eprintln!("collector failed: {:?}", why);
                    Err(collector.on_error(why))
                }
                Err(why) => {
                    eprintln!("collector paniced: {:?}", why);
                    Err(collector.on_panic(why))
                }
            };
            // continue
            if let Ok(continuation) = result.clone() {
                match continuation {
                    Continuation::ContinueAfter(_) => {
                        todo!();
                    }
                    Continuation::ContinueAt(_) => {
                        todo!();
                    }
                    Continuation::Continue => {
                        if let Some(tick) = &mut interval {
                            tick.tick().await;
                        }
                    }
                    Continuation::Restart => {
                        match client.database.auto().get_collector(&id).await {
                            Ok(value) => {
                                collector = factory(value.state);
                                if let Some(tick) = &mut interval {
                                    tick.tick().await;
                                }
                            }
                            Err(why) => {
                                result = Err(collector.on_panic(Box::new(why)))
                            }
                        }
                    }
                    Continuation::Exit => {
                        break;
                    }
                }
                backoff = collector.tick().unwrap_or(Duration::from_secs(10));
            }

            while let Err(strategy) = result.clone() {
                match strategy {
                    SupervisionStrategy::Restart => {
                        match client.database.auto().get_collector(&id).await {
                            Ok(value) => {
                                collector = factory(value.state);
                            }
                            Err(why) => {
                                result = Err(collector.on_panic(Box::new(why)))
                            }
                        }
                    }
                    SupervisionStrategy::Resume => {
                        break;
                    }
                    SupervisionStrategy::Stop => {
                        return;
                    }
                }
                backoff = collector.backoff(backoff);
                sleep(backoff).await;
            }
        }
    });

    CollectorRef {}
}
