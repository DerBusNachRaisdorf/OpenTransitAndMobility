use std::fmt::Debug;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{handler::MessageHandler, Actor};

#[async_trait]
pub trait Mailbox<A>
where
    A: Actor,
{
    type Error: Debug;

    async fn send<M>(&self, message: M) -> Result<(), Self::Error>
    where
        M: MessageHandler<A> + 'static;
}

#[async_trait]
pub trait MailboxReceiver<A>
where
    A: Actor,
{
    async fn recv(&mut self) -> Option<Box<dyn MessageHandler<A>>>
    where
        A: Actor;
}

#[derive(Clone)]
pub struct BoundedMailbox<A>(mpsc::Sender<Box<dyn MessageHandler<A>>>);

#[async_trait]
impl<A> Mailbox<A> for BoundedMailbox<A>
where
    A: Actor,
{
    type Error = mpsc::error::SendError<Box<dyn MessageHandler<A>>>;

    async fn send<M>(&self, message: M) -> Result<(), Self::Error>
    where
        M: MessageHandler<A> + 'static,
    {
        self.0.send(Box::new(message)).await?;
        Ok(())
    }
}

pub struct BoundedMailboxReceiver<A>(mpsc::Receiver<Box<dyn MessageHandler<A>>>);

#[async_trait]
impl<A> MailboxReceiver<A> for BoundedMailboxReceiver<A>
where
    A: Actor,
{
    async fn recv(&mut self) -> Option<Box<dyn MessageHandler<A>>>
    where
        A: Actor,
    {
        self.0.recv().await.map(|message| message)
    }
}

pub fn bounded_mailbox<A>(buffer: usize) -> (BoundedMailbox<A>, BoundedMailboxReceiver<A>)
where
    A: Actor,
{
    let (tx, rx) = mpsc::channel(buffer);
    (BoundedMailbox(tx), BoundedMailboxReceiver(rx))
}
