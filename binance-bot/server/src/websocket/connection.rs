use shared::{
	ClientMessage,
	ServerMessage,
	WebsocketCommand,
};
use futures::{
	StreamExt,
	SinkExt,
	channel::mpsc::{
		Receiver,
		channel,
	},
	Stream,
	Sink,
};
use std::{
	fmt::{
		Debug,
	},
	convert::{
		TryInto,
	},
	result::Result,
};
use tide_websockets::{
    Message,
	WebSocketConnection,
};
use riker::actors::*;
use crate::websocket::{
    Connection,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
	warn,
};
#[derive(Debug)]
pub struct WebsocketPacket(Vec<u8>);
impl From<ServerMessage> for WebsocketPacket {
	fn from(msg: ServerMessage) -> Self {
		Self(serde_json::to_vec(&msg).expect("Failed to serialize ServerMessage to Vec<u8>."))
	}
}
impl TryInto<ClientMessage> for WebsocketPacket {
	type Error = serde_json::Error;
	fn try_into(self) -> Result<ClientMessage, Self::Error> {
		serde_json::from_slice(&self.0)
	}
}
pub async fn poll_messages<E, Rx>(connection: ActorRef<<Connection as Actor>::Msg>, mut rx: Rx)
	where E: ToString + Send + Debug,
		  Rx: Stream<Item=Result<WebsocketPacket, E>> + Send + 'static + Unpin,
{
	while let Some(msg) = rx.next().await {
		debug!("ClientMessage received: {:#?}", msg);
		// convert M to ClientMessage
		let res = msg
			.map_err(|e| e.to_string())
			.and_then(|m| m.try_into()
					  .map_err(|e| format!("Failed to parse ClientMessage: {}", e))
					  as Result<ClientMessage, String>)
			.map(|msg| WebsocketCommand::ClientMessage(msg));
		match res {
			Ok(msg) => {
				// forward messages to connection actor
				if let WebsocketCommand::Close = msg {
					// stop listener
					crate::actor_sys().await.stop(connection);
					break;
				} else {
					// handle message
					connection.tell(msg, None);
				}
			},
			Err(e) => error!("{}", e),
		}
	}
}
pub async fn send_messages<Tx>(receiver: Receiver<ServerMessage>, tx: Tx) -> Result<(), String>
	where Tx: Sink<WebsocketPacket> + Send + 'static,
		  <Tx as Sink<WebsocketPacket>>::Error: ToString,
{
	receiver
		.map(|msg: ServerMessage|
			Ok(WebsocketPacket::from(msg))
		)
		// send messages through websocket sink
		.forward(tx.sink_map_err(|e| e.to_string()))
		.await
}
pub async fn connection(ws: WebSocketConnection) {
	let (sink, stream) = ws.split();
	let rx = stream.map(|msg| msg.map(|msg| WebsocketPacket(msg.into_data())));
	let tx = sink.with(async move |msg: WebsocketPacket| {
		Ok(Message::from(msg.0)) as Result<_, tide_websockets::Error>
	});
	// connection lasts for the duration of this async fn
	debug!("Starting websocket connection");
	const CHANNEL_BUFFER_SIZE: usize = 100;
	let (sender, receiver) = channel(CHANNEL_BUFFER_SIZE);

	// create a connection actor with a ServerMessage sender
	let connection = Connection::create(sender).await.unwrap();
	let connection2 = connection.clone();
	// spawn listener for websocket stream
	let ws_listener = async_std::task::spawn(async move {
		poll_messages(connection2, rx).await
	});
	send_messages(receiver, tx).await
		.expect("Failed to forward connection messages to websocket!");
	//// wait for ServerMessages from connection actor
	ws_listener.await;
	//async_std::task::sleep(std::time::Duration::from_secs(100)).await;
	debug!("Closing websocket connection");
	crate::actor_sys().await.stop(connection.clone());
}

