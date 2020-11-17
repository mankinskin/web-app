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
use riker::actors::*;
use futures::{
    StreamExt,
    SinkExt,
    channel::mpsc::{
        Sender,
        channel,
    },
};
use futures_util::{
    stream::SplitSink,
};
use riker::actors::{
    Sender as RkSender
};
use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
    convert::TryInto,
};
#[cfg(feature = "warp_server")]
use {
    warp::{
        ws::WebSocket,
    },
};
use lazy_static::lazy_static;
lazy_static! {
    static ref SESSION_COUNT: AtomicUsize = AtomicUsize::new(0);
}

pub fn new_session_id() -> usize {
    SESSION_COUNT.fetch_add(1, Ordering::Relaxed)
}

pub async fn create_session_actor(sender: Sender<ServerMessage>) -> Result<ActorRef<<Session as Actor>::Msg>, CreateError> {
    let id = new_session_id();
    crate::actor_sys().await.actor_of_args::<Session, _>(&format!("session_{}", id), sender)
}

#[cfg(feature = "warp_server")]
pub async fn websocket_session(ws: WebSocket) {
    // the session lasts for the duration of this async fn
    debug!("Open websocket connection");
    let (tx, mut rx): (SplitSink<WebSocket, warp::ws::Message>, _) = ws.split();
    let (sender, receiver) = channel(100);
    let session = create_session_actor(sender).await.unwrap();

    let session2 = session.clone();
    let ws_listener = tokio::spawn(async move {
        let session = session2;
        while let Some(msg) = rx.next().await {
            msg
                .map_err(|e| e.to_string())
                .and_then(|msg| msg.try_into() as Result<ClientMessage, String>)
                .map(|msg: ClientMessage| session.tell(msg, None))
                .unwrap_or_else(|e| error!("{}", e));
        }
    });
    receiver
        .map(|msg: ServerMessage|
            serde_json::to_string(&msg)
                .map(|s| warp::ws::Message::text(s))
                .map_err(|e| format!("{}", e))
        )
        .forward(tx.sink_map_err(|e| e.to_string()))
        .await
        .expect("Failed to forward session messages to websocket!");
    
    ws_listener.await.expect("Failed to join websocket listener thread!");
    crate::actor_sys().await.stop(session.clone());
    debug!("closing websocket connection");
}

#[derive(Debug, Clone)]
pub struct Error(String);
impl<E: ToString> From<E> for Error {
    fn from(s: E) -> Self {
        Self(s.to_string())
    }
}

#[actor(ClientMessage)]
#[derive(Debug)]
pub struct Session {
    sender: Sender<ServerMessage>,
    subscriptions: Option<ActorRef<<SubscriptionsActor as Actor>::Msg>>,
    //init_subscriptions: Option<SpawnHandle>,
}
impl Actor for Session {
    type Msg = SessionMsg;
    fn pre_start(&mut self, _ctx: &Context<Self::Msg>) {
        debug!("Starting session actor");
    }
    fn post_stop(&mut self) {
        debug!("Stopped session actor");
    }
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: RkSender) {
        self.receive(ctx, msg, sender);
    }
}
impl Receive<ClientMessage> for Session {
    type Msg = SessionMsg;
    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ClientMessage, _sender: RkSender) {
        debug!("Received {:#?}", msg);
    }
}
impl ActorFactoryArgs<Sender<ServerMessage>> for Session {
    fn create_args(sender: Sender<ServerMessage>) -> Self {
        info!("Creating Session");
        Self {
            sender,
            subscriptions: None,
        }
    }
}
#[cfg(feature = "actix_server")]
mod actix_session_actor {
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
}
