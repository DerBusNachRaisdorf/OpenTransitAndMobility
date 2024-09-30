# Actor framework for `public_transport`

A very simple actor framework for `public_transport` inspired by
[Tiny Tokio Actor](https://github.com/fdeantoni/tiny-tokio-actor).

## Example

```rs
pub struct PingActor {}

impl Actor for PingActor {}

#[derive(Clone)]
struct Ping {}

impl Message for Ping {
    type Response = String;
}

#[async_trait]
impl Handler<Ping> for PingActor {
    async fn handle(&mut self, _: Ping) -> <Ping as Message>::Response {
        "pong".to_owned()
    }
}

#[derive(Clone)]
struct Echo {
    pub message: String,
}

impl Message for Echo {
    type Response = String;
}

#[async_trait]
impl Handler<Echo> for PingActor {
    async fn handle(&mut self, message: Echo) -> <Echo as Message>::Response {
        message.message
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let actor_ref = run(|| PingActor {});
    let pong = actor_ref.ask(Ping {}).await.unwrap();
    println!("ping: {}", pong);
    let echo = actor_ref
        .ask(Echo {
            message: "test".to_owned(),
        })
        .await
        .unwrap();
    println!("echo: {}", echo);
}
```
