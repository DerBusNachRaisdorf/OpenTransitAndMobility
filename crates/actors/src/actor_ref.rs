use tokio::sync::oneshot;

use crate::{
    actor::{Actor, ActorError},
    handler::{ActorMessage, Handler, Message},
    mailbox::{BoundedMailbox, Mailbox},
};

#[derive(Clone)]
pub struct ActorRef<A: Actor> {
    sender: BoundedMailbox<A>,
}

impl<A: Actor> ActorRef<A> {
    pub(crate) fn new(sender: BoundedMailbox<A>) -> Self {
        Self { sender }
    }

    pub async fn tell<M>(&self, msg: M) -> Result<(), ActorError<A, BoundedMailbox<A>>>
    where
        M: Message,
        A: Handler<M>,
    {
        let message = ActorMessage::<M, A>::new(msg, None);
        self.sender
            .send(message)
            .await
            .map_err(|why| ActorError::<A, BoundedMailbox<A>>::SendError(why))
    }

    pub async fn ask<M>(&self, msg: M) -> Result<M::Response, ActorError<A, BoundedMailbox<A>>>
    where
        M: Message,
        A: Handler<M>,
    {
        let (response_tx, response_rx) = oneshot::channel();
        let message = ActorMessage::<M, A>::new(msg, Some(response_tx));
        self.sender
            .send(message)
            .await
            .map_err(|why| ActorError::<A, BoundedMailbox<A>>::SendError(why))?;
        response_rx
            .await
            .map_err(|why| ActorError::ReceiveAnswerError(why))
    }
}
