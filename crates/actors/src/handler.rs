use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::Actor;

#[async_trait]
pub trait Handler<M>: Actor
where
    M: Message,
{
    async fn handle(&mut self, message: M) -> M::Response;
}

pub trait Message: Clone + Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

#[async_trait]
pub trait MessageHandler<A: Actor>: Send + Sync {
    async fn handle(&mut self, actor: &mut A);
}

pub struct ActorMessage<M, A>
where
    M: Message,
    A: Actor,
{
    message: M,
    respond_to: Option<oneshot::Sender<M::Response>>,
    _phantom_actor: PhantomData<A>,
}

impl<M, A> ActorMessage<M, A>
where
    M: Message,
    A: Actor,
{
    pub fn new(message: M, respond_to: Option<oneshot::Sender<M::Response>>) -> Self {
        Self {
            message,
            respond_to,
            _phantom_actor: Default::default(),
        }
    }
}

#[async_trait]
impl<M, A> MessageHandler<A> for ActorMessage<M, A>
where
    M: Message,
    A: Handler<M>,
{
    async fn handle(&mut self, actor: &mut A) {
        let result = actor.handle(self.message.clone()).await;

        if let Some(respond_to) = self.respond_to.take() {
            respond_to
                .send(result)
                .unwrap_or_else(|_| log::error!("Can not respond to message!"));
        }
    }
}
