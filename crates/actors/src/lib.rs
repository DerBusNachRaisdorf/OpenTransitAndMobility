use std::panic::AssertUnwindSafe;

use actor::{Actor, SupervisionStrategy};
use actor_ref::ActorRef;
use futures::FutureExt;
use mailbox::{bounded_mailbox, MailboxReceiver};

pub mod actor;
pub mod actor_ref;
pub mod handler;
pub mod mailbox;

/// Creates and runs an actor. If the actor panics, it is either restared, resumed
/// or stoped acording to the behavior specified by `Actor::on_fail()`.
pub fn run<A, F>(actor_factory: F) -> ActorRef<A>
where
    A: Actor,
    F: 'static + Send + Fn() -> A,
{
    let (tx, mut rx) = bounded_mailbox(32);
    let mut actor = actor_factory();
    let actor_ref = ActorRef::new(tx);

    // run actor
    tokio::spawn(async move {
        while let Some(mut message) = rx.recv().await {
            // handle message
            let result = AssertUnwindSafe(message.handle(&mut actor))
                .catch_unwind()
                .await;
            // handler paniced?
            if let Err(why) = result {
                log::error!("actor paniced: {:?}", why);
                match actor.on_fail(why) {
                    SupervisionStrategy::Restart => {
                        actor = actor_factory();
                    }
                    SupervisionStrategy::Resume => {}
                    SupervisionStrategy::Stop => {
                        break;
                    }
                };
            }
        }
    });

    actor_ref
}

/// Run an actor without supervision. This is not recommended.
pub fn run_unsupervised<A: Actor>(mut actor: A) -> ActorRef<A> {
    let (tx, mut rx) = bounded_mailbox(32);
    let actor_ref = ActorRef::new(tx);

    // run actor
    tokio::spawn(async move {
        while let Some(mut message) = rx.recv().await {
            message.handle(&mut actor).await;
        }
    });

    actor_ref
}
