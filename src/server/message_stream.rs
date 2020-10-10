#[allow(unused)]
use tracing::{
    debug,
    error,
};
// for parallel_stream compatibility
use futures_core::{
    stream::Stream,
};
use parallel_stream::ParallelStream;
use futures::{
    StreamExt,
    future::Future,
};
use crate::{
    Error,
};
pub struct MessageStream<M: Send> {
    streams: Vec<Box<dyn MsgStream<M>>>,
}
pub trait MsgStream<M> : Stream<Item=Result<M, Error>> + Send + Sync + Unpin + 'static {}
impl<M, S: Stream<Item=Result<M, Error>> + Send + Sync + Unpin + 'static> MsgStream<M> for S {}

pub trait Handler<M, R: HandlerResult> : Fn(M) -> R + Sync + Send + Clone + 'static {}
impl<M, R: HandlerResult, T: Fn(M) -> R + Sync + Send + Clone + 'static> Handler<M, R> for T {}

pub trait HandlerResult : Future<Output=Result<(), Error>> + Send {}
impl<T: Future<Output=Result<(), Error>> + Send> HandlerResult for T {}

pub trait HandlerMessage : Send + 'static {}
impl<T: Send + 'static> HandlerMessage for T {}

impl<M: HandlerMessage> MessageStream<M> {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
        }
    }
    pub fn with_stream<I: Into<M>>(mut self, stream: impl MsgStream<I>) -> Self {
        self.streams.push(Box::new(stream.map(|r| r.map(Into::into))));
        self
    }
    pub async fn spawn_handlers<R: HandlerResult>(self, handler: impl Handler<M, R>) -> Result<(), Error> {
        let mut stream = parallel_stream::from_stream(
            tokio::stream::iter(self.streams.into_iter()).flatten()
        );
        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    let handler = handler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handler(message).await {
                            error!("{:#?}", e);
                        }
                    });
                },
                Err(e) => error!("{:#?}", e),
            }
        }
        Ok(())
    }
}
