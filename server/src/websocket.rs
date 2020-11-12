use crate::{
    subscriptions::SubscriptionsActor,
};
use shared::{
    ClientMessage,
    ServerMessage,
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
    SpawnHandle,
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
#[derive(Debug, Clone)]
pub struct Error(String);
impl<E: ToString> From<E> for Error {
    fn from(s: E) -> Self {
        Self(s.to_string())
    }
}
#[derive(Debug)]
pub struct Session {
    id: Option<Addr<Self>>,
    subscriptions: Option<Addr<SubscriptionsActor>>,
    init_subscriptions: Option<SpawnHandle>,
}
impl Session {
    pub fn new() -> Self {
        Self {
            id: None,
            subscriptions: None,
            init_subscriptions: None,
        }
    }
}
impl Actor for Session {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!["Started Websocket Session"];
        let addr = ctx.address();
        self.id = Some(addr.clone());

        self.init_subscriptions = Some(ctx.spawn(async move {
            let addr = SubscriptionsActor::init(addr).await;
            with_ctx::<Self, _, _>(|act, _ctx| {
                act.subscriptions = Some(addr);
            });
        }.interop_actor_boxed(self)));
    }
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
                            debug!("Closing websocket connection {:?} ({:#?}{})",
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
        let subscriptions = self.subscriptions.clone().expect("Subscriptions not available");
        async move {
            match msg {
                ClientMessage::Subscriptions(req) => {
                    let response = subscriptions.send(req).await.expect("Send failed");
                    if let Some(response) = response {
                        with_ctx::<Self, _, _>(|_act, ctx| {
                            ctx.notify(ServerMessage::Subscriptions(response));
                        })
                    };
                },
                _ => {},
            };
            None
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
        info!("Sending response");
        ctx.text(resp);
        MessageResult(())
    }
}
