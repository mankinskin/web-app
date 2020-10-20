use crate::{
    subscriptions::Subscriptions,
    shared::{
        ClientMessage,
        ServerMessage,
    },
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};
use actix::{
    Actor,
    StreamHandler,
    Handler,
    AsyncContext,
    Addr,
    MessageResult,
    ResponseActFuture,
};
use actix_web_actors::ws::{
    self,
    WebsocketContext,
    ProtocolError,
};
use actix_interop::{
    FutureInterop,
    with_ctx,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use lazy_static::lazy_static;
lazy_static! {
    static ref SESSION_COUNT: AtomicUsize = AtomicUsize::new(0);
}
#[derive(Debug, Clone)]
pub struct Error(String);
impl<E: ToString> From<E> for Error {
    fn from(s: E) -> Self {
        Self(s.to_string())
    }
}
pub struct Session {
    id: usize,
    subscriptions: Addr<Subscriptions>,
}
impl Session {
    pub fn new(subscriptions: Addr<Subscriptions>) -> Self {
        Self {
            id: Self::new_id(),
            subscriptions,
        }
    }
    fn new_id() -> usize {
        SESSION_COUNT.fetch_add(1, Ordering::Relaxed)
    }
}
impl Actor for Session {
    type Context = WebsocketContext<Self>;
}
impl StreamHandler<Result<ws::Message, ProtocolError>> for Session {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(msg) =>
                match msg {
                    ws::Message::Text(string) => {
                        match serde_json::from_str(&string) as Result<ClientMessage, _> {
                            Ok(msg) => {
                                ctx.notify(msg);
                            },
                            Err(err) => error!("{}", err),
                        }
                    },
                    ws::Message::Binary(_bytes) => {
                        error!("Unable to handle ClientMessage as bytes.")
                    },
                    ws::Message::Continuation(_item) => {},
                    ws::Message::Ping(_bytes) => {},
                    ws::Message::Pong(_bytes) => {},
                    ws::Message::Close(reason) => {
                        if let Some(reason) = reason {
                            debug!("Closing websocket connection {} ({:#?}{})",
                                self.id,
                                reason.code,
                                match &reason.description {
                                    Some(desc) => format!(": {}", desc),
                                    None => String::new(),
                                }
                            );
                        }
                    },
                    _ => {}
                },
            Err(err) => error!("{}", err),
        }
    }
}
impl Handler<ClientMessage> for Session {
    type Result = ResponseActFuture<Self, Option<ServerMessage>>;
    fn handle(
        &mut self,
        msg: ClientMessage,
        _: &mut Self::Context,
    ) -> Self::Result
    {
        let subscriptions = self.subscriptions.clone();
        async move {
            match msg {
                ClientMessage::Subscriptions(msg) => {
                    let response = subscriptions.send(msg).await.expect("Send failed");
                    if let Some(response) = response {
                        with_ctx::<Self, _, _>(|_act, ctx| {
                            ctx.notify(ServerMessage::Subscriptions(response));
                        })
                    };
                    None
                },
                _ => None,
            }
        }.interop_actor_boxed(self)
    }
}
impl Handler<ServerMessage> for Session {
    type Result = MessageResult<ServerMessage>;
    fn handle(
        &mut self,
        msg: ServerMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result
    {
        let resp = serde_json::to_string(&msg).unwrap();
        info!("Sending response {}", &resp);
        ctx.text(resp);
        MessageResult(())
    }
}
