use crate::{
    subscriptions::SubscriptionsActor,
    websocket,
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
    convert::TryInto,
};
#[cfg(feature = "warp_server")]
use {
    warp::{
        ws::WebSocket,
    },
};

#[actor(ClientMessage)]
#[derive(Debug)]
pub struct Session {
    sender: Sender<ServerMessage>,
    subscriptions: Option<ActorRef<<SubscriptionsActor as Actor>::Msg>>,
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
    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: ClientMessage, _sender: RkSender) {
        debug!("Received {:#?}", msg);
        match msg {
            ClientMessage::Close => ctx.stop(ctx.myself()),
            _ => {}
        }
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

pub async fn create_session_actor(sender: Sender<ServerMessage>) -> Result<ActorRef<<Session as Actor>::Msg>, CreateError> {
    let id = websocket::new_session_id();
    crate::actor_sys().await.actor_of_args::<Session, _>(&format!("session_{}", id), sender)
}

#[cfg(feature = "warp_server")]
pub async fn websocket_session(ws: WebSocket) {
    // session lasts for the duration of this async fn
    debug!("Open websocket connection");
    let (tx, mut rx): (SplitSink<WebSocket, warp::ws::Message>, _) = ws.split();
    const CHANNEL_BUFFER_SIZE: usize = 100;
    let (sender, receiver) = channel(CHANNEL_BUFFER_SIZE);

    // create a session actor with a ServerMessage sender
    let session = create_session_actor(sender).await.unwrap();

    // spawn listener for websocket stream
    let session2 = session.clone();
    let ws_listener = tokio::spawn(async move {
        let session = session2;
        while let Some(msg) = rx.next().await {
            match msg
                // convert warp::Message to ClientMessage
                .map_err(|e| e.to_string())
                .and_then(|msg| msg.try_into() as Result<ClientMessage, String>) {
                Ok(msg) => {
                    // forward messages to session actor
                    if let ClientMessage::Close = msg {
                        // stop listener
                        crate::actor_sys().await.stop(session);
                        break;
                    } else {
                        session.tell(msg, None);
                    }
                },
                Err(e) => error!("{}", e),
            }
            
        }
    });
    // wait for ServerMessages from session actor
    receiver
        .map(|msg: ServerMessage|
            serde_json::to_string(&msg)
                .map(|s| warp::ws::Message::text(s))
                .map_err(|e| format!("{}", e))
        )
        // send messages through websocket sink
        .forward(tx.sink_map_err(|e| e.to_string()))
        .await
        .expect("Failed to forward session messages to websocket!");
    ws_listener.await.expect("Failed to join websocket listener thread!");
    crate::actor_sys().await.stop(session.clone());
    debug!("closing websocket connection");
}

