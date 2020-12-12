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
    Stream,
    Sink,
};
use riker::actors::{
    Sender as RkSender
};
use std::convert::{
    TryFrom,
};
use serde::{
    Serialize,
    Deserialize,
};

#[actor(ClientMessage)]
#[derive(Debug)]
pub struct Connection {
    sender: Sender<ServerMessage>,
    subscriptions: Option<ActorRef<<SubscriptionsActor as Actor>::Msg>>,
}
impl Actor for Connection {
    type Msg = ConnectionMsg;
    fn pre_start(&mut self, _ctx: &Context<Self::Msg>) {
        debug!("Starting connection actor");
    }
    fn post_stop(&mut self) {
        debug!("Stopped connection actor");
    }
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: RkSender) {
        self.receive(ctx, msg, sender);
    }
}
impl Receive<ClientMessage> for Connection {
    type Msg = ConnectionMsg;
    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: ClientMessage, _sender: RkSender) {
        debug!("Received {:#?}", msg);
        match msg {
            ClientMessage::Close => ctx.stop(ctx.myself()),
            _ => {}
        }
    }
}
impl ActorFactoryArgs<Sender<ServerMessage>> for Connection {
    fn create_args(sender: Sender<ServerMessage>) -> Self {
        info!("Creating Connection");
        Self {
            sender,
            subscriptions: None,
        }
    }
}

pub async fn create_connection_actor(sender: Sender<ServerMessage>) -> Result<ActorRef<<Connection as Actor>::Msg>, CreateError> {
    let id = websocket::new_connection_id();
    crate::actor_sys().await.actor_of_args::<Connection, _>(&format!("connection_{}", id), sender)
}
pub async fn connection<E, M, Rx, Tx>(mut rx: Rx, tx: Tx)
    where E: ToString + Send,
          M: Serialize + for<'de> Deserialize<'de> + Send,
          Rx: Stream<Item=Result<M, E>> + Send + 'static + Unpin,
          Tx: Sink<M> + Send + 'static,
          <Tx as Sink<M>>::Error: ToString,
{
    // connection lasts for the duration of this async fn
    debug!("Open websocket connection");
    const CHANNEL_BUFFER_SIZE: usize = 100;
    let (sender, receiver) = channel(CHANNEL_BUFFER_SIZE);

    // create a connection actor with a ServerMessage sender
    let connection = create_connection_actor(sender).await.unwrap();

    // spawn listener for websocket stream
    let connection2 = connection.clone();
    let ws_listener = tokio::spawn(async move {
        let connection = connection2;
        while let Some(msg) = rx.next().await {
            // convert M to ClientMessage
            match msg
                .map_err(|e| e.to_string())
                .and_then(|msg| serde_json::to_string(&msg).map_err(|e| e.to_string()))
                .and_then(|msg| ClientMessage::try_from(msg).map_err(|e| e.to_string())) {
                Ok(msg) => {
                    // forward messages to connection actor
                    if let ClientMessage::Close = msg {
                        // stop listener
                        crate::actor_sys().await.stop(connection);
                        break;
                    } else {
                        connection.tell(msg, None);
                    }
                },
                Err(e) => error!("{}", e),
            }
            
        }
    });
    // wait for ServerMessages from connection actor
    receiver
        .map(|msg: ServerMessage|
            serde_json::to_string(&msg)
                .map_err(|e| e.to_string())
                .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
        )
        // send messages through websocket sink
        .forward(tx.sink_map_err(|e| e.to_string()))
        .await
        .expect("Failed to forward connection messages to websocket!");
    ws_listener.await.expect("Failed to join websocket listener thread!");
    crate::actor_sys().await.stop(connection.clone());
    debug!("closing websocket connection");
}

