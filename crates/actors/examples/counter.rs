use std::any::Any;

use actors::{
    actor::{Actor, SupervisionStrategy},
    actor_ref::ActorRef,
    handler::{Handler, Message},
    run,
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct Increment {
    pub value: i64,
}

impl Message for Increment {
    type Response = ();
}

#[derive(Clone)]
pub struct GetValue {}

impl Message for GetValue {
    type Response = i64;
}

pub struct Counter {
    pub count: i64,
}

impl Actor for Counter {
    fn on_fail(&mut self, _: Box<dyn Any + Send>) -> SupervisionStrategy {
        SupervisionStrategy::Restart
    }
}

#[async_trait]
impl Handler<Increment> for Counter {
    async fn handle(&mut self, message: Increment) {
        self.count -= message.value;
    }
}

#[async_trait]
impl Handler<GetValue> for Counter {
    async fn handle(&mut self, _: GetValue) -> i64 {
        self.count
    }
}

#[async_trait]
pub trait CounterRef {
    async fn increment(&self, value: i64);
    async fn get_value(&self) -> i64;
}

#[async_trait]
impl CounterRef for ActorRef<Counter> {
    async fn increment(&self, value: i64) {
        self.tell(Increment { value }).await.unwrap();
    }

    async fn get_value(&self) -> i64 {
        self.ask(GetValue {}).await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    let actor_ref = run(|| Counter { count: 0 });
    actor_ref.increment(1).await;
    actor_ref.increment(5).await;
    actor_ref.increment(-2).await;
    let value = actor_ref.get_value().await;
    println!("value: {}", value); // 4
}
