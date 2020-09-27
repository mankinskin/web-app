use crate::{
    shared::{
        ServerMessage,
        ClientMessage,
    },
    Error,
};
use futures::{
    Stream,
    Sink,
    channel::mpsc::{
        Sender,
        Receiver,
    },
    task::{
        Poll,
        Context,
    },
};
#[allow(unused)]
use tracing::{
    debug,
    error,
};
use std::pin::Pin;

pub struct Connection {
    sender: Sender<ClientMessage>,
    receiver: Receiver<ServerMessage>,
}
impl Connection {
    pub fn new(sender: Sender<ClientMessage>, receiver: Receiver<ServerMessage>) -> Self {
        Self {
            sender,
            receiver,
        }
    }
    //pub async fn push_update(&mut self) -> Result<(), Error> {
    //    //debug!("Pushing updates");
    //    for subscription in self.subscriptions.clone().iter() {
    //        //debug!("Updating subscription {}", &subscription.market_pair);
    //        let history = subscription.latest_price_history().await?;
    //        self.send(ClientMessage::PriceHistory(history)).await?;
    //    }
    //    Ok(())
    //}
    //pub async fn receive_message(&mut self, msg: ServerMessage) -> Result<(), Error> {
    //    //debug!("Received websocket msg");
    //    //debug!("{:#?}", msg);
    //    let response = match msg {
    //        ServerMessage::SubscribePrice(market_pair) => {
    //            //debug!("Subscribing to market pair {}", &market_pair);
    //            crate::model().await.add_symbol(market_pair.clone()).await?;
    //            crate::server::interval::set(interval(Duration::from_secs(1)));    
    //            let subscription = PriceSubscription::from(market_pair);
    //            let response = ClientMessage::PriceHistory(subscription.latest_price_history().await?);
    //            self.subscriptions.push(subscription);
    //            Some(response)
    //        },
    //        _ => None,
    //    };
    //    if let Some(response) = response {
    //        self.send(response).await.map_err(Into::into)
    //    } else {
    //        Ok(())
    //    }
    //}
}
impl Stream for Connection {
    type Item = Result<ServerMessage, Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.receiver), cx)
            .map(|opt| opt.map(Ok))
    }
}
impl Sink<ClientMessage> for Connection {
    type Error = Error;
    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn start_send(mut self: Pin<&mut Self>, item: ClientMessage) -> Result<(), Self::Error> {
        Sink::start_send(Pin::new(&mut self.sender), item).map_err(Into::into)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_close(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
}
