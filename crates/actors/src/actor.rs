use core::fmt;
use std::any::Any;

use tokio::sync::oneshot;

use crate::mailbox::Mailbox;

#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    Restart,
    Resume,
    Stop,
}

pub trait Actor: Send + Sync + 'static {
    /// Called when a handler on the actor panics. The return value represents the
    /// supervision strategy used to handle the panic.
    /// NOTE: If this method panics, the actor can not recover from the panic.
    #[allow(unused_variables)]
    fn on_fail(&mut self, error: Box<dyn Any + Send>) -> SupervisionStrategy {
        SupervisionStrategy::Restart
    }
}

pub enum ActorError<A, M>
where
    A: Actor,
    M: Mailbox<A>,
{
    SendError(M::Error),
    ReceiveAnswerError(oneshot::error::RecvError),
}

impl<A, M> fmt::Debug for ActorError<A, M>
where
    A: Actor,
    M: Mailbox<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SendError(why) => write!(f, "SendError: {:?}", why),
            Self::ReceiveAnswerError(why) => write!(f, "ReceiveError: {:?}", why),
        }
    }
}
