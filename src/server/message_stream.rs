#[allow(unused)]
use tracing::{
    debug,
    error,
};
use futures::{
    Stream,
    StreamExt,
    future::Future,
};
use crate::{
    Error,
};
use std::any::{
    TypeId,
    Any,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::stream::StreamMap;
use std::pin::Pin;
use std::marker::Unpin;

pub trait AnyMessage : Any + Send + Sync {}
impl<M: Any + Send + Sync> AnyMessage for M {}

pub trait AnyStream : Stream<Item=Arc<dyn Any + Send + Sync>> + Send + Sync + Unpin + 'static {}
impl<S: Stream<Item=Arc<dyn Any + Send + Sync>> + Send + Sync + Unpin + 'static> AnyStream for S {}

pub trait MessageStream<M> : Stream<Item=M> + Send + Sync + Unpin + Sized + 'static {}
impl<M, S: Stream<Item=M> + Send + Sync + Unpin + 'static + Sized> MessageStream<M> for S {}

pub trait AnyHandlerArg : Any + Sync + Send + 'static {}
impl<A: Any + Sync + Send + 'static> AnyHandlerArg for A {}

pub trait HandlerMessage : Any + 'static + Clone {}
impl<M: Any + 'static + Clone> HandlerMessage for M {}

pub trait AsyncAnyHandler : Fn(Arc<dyn Any + Send + Sync + 'static>) -> Pin<Box<dyn AsyncHandlerResult>> + Sync + Send + 'static {}
impl<H: Fn(Arc<dyn Any + Send + Sync + 'static>) -> Pin<Box<dyn AsyncHandlerResult>> + Sync + Send + 'static> AsyncAnyHandler for H {}

pub trait AsyncHandler<M: AsyncHandlerMessage, R: AsyncHandlerResult> : Fn(M) -> R + Sync + Send + 'static + Clone {}
impl<M: AsyncHandlerMessage, R: AsyncHandlerResult, H: Fn(M) -> R + Sync + Send + 'static + Clone> AsyncHandler<M, R> for H {}

pub trait AsyncHandlerResult : Future<Output=()> + Send + Sync + 'static {}
impl<T: Future<Output=()> + Send + Sync + 'static> AsyncHandlerResult for T {}

pub trait AsyncHandlerMessage : Any + Send + Sync + 'static + Clone {}
impl<M: Any + Send + Sync + 'static + Clone> AsyncHandlerMessage for M {}

pub struct EventTypeManager {
    streams: Vec<Box<dyn AnyStream>>,
    handlers: Vec<Arc<dyn AsyncAnyHandler>>,
}
impl EventTypeManager {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            handlers: Vec::new(),
        }
    }
    pub fn from_stream<M: AnyMessage>(stream: impl MessageStream<M>) -> Self {
        Self {
            streams: vec![Self::boxed_any_stream(stream)],
            handlers: Vec::new(),
        }
    }
    pub fn with_stream<M: AnyMessage>(mut self, other: impl MessageStream<M>) -> Self{
        let other = Self::boxed_any_stream(other);
        self.streams.push(other);
        self
    }
    fn boxed_any_stream<M: Any + Send + Sync>(stream: impl MessageStream<M>) -> Box<impl AnyStream> {
        Box::new(stream.map(|m| Arc::new(m) as Arc<dyn Any + Send + Sync>))
    }
    pub fn with_handler<M: AsyncHandlerMessage, R: AsyncHandlerResult>(&mut self, handler: impl AsyncHandler<M, R>) {
        self.handlers
            .push(Arc::new(move |a: Arc<dyn Any + Send + Sync + 'static>| {
                let a = Arc::downcast::<M>(a).expect("Downcast to `M` in Handler");
                let handler = handler.clone();
                Box::pin((async move |a: M| handler(a).await)((*a).clone()))
            }));
    }
}
impl Stream for EventTypeManager {
    type Item=();
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let Self { streams, handlers } = &mut*self;
        let mut stream = futures::stream::select_all(streams);
        let poll = Stream::poll_next(Pin::new(&mut stream), cx);
        if let Poll::Ready(Some(arc)) = poll {
            for handler in handlers.iter() {
                tokio::spawn((*handler)(arc.clone()));
            }
            return Poll::Ready(Some(()));
        }
        Poll::Pending
    }
}

//struct IdHandlers(TypeId, Vec<Arc<dyn AsyncAnyHandler>>);
//impl PartialEq for IdHandlers {
//    fn eq(&self, o: &Self) -> bool {
//        self.0 == o.0
//    }
//}
//impl Eq for IdHandlers {}
//impl std::hash::Hash for IdHandlers {
//    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
//        self.0.hash(hasher)
//    }
//}
impl Stream for EventManager {
    type Item=();
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.tys), cx).map(|o| o.map(|(_, r)| r))
    }
}
pub struct EventManager {
    tys: StreamMap<TypeId, EventTypeManager>
}
impl EventManager {
    pub fn new() -> Self {
        Self {
            tys: StreamMap::new(),
        }
    }
    pub fn stream<M: AnyMessage>(&mut self, stream: impl MessageStream<M>) {
        let id = TypeId::of::<M>();
        let new = if let Some(man) = self.tys.remove(&id) {
            man.with_stream(stream)
        } else {
            EventTypeManager::from_stream(stream)
        };
        self.tys.insert(id, new);
    }
    pub fn handler<M: AsyncHandlerMessage, R: AsyncHandlerResult>(&mut self, handler: impl AsyncHandler<M, R>) {
        let id = TypeId::of::<M>();
        //debug!("Adding handler to {:#?}", &id);

        // TODO add entry API to tokio::StreamMap
        if !self.tys.contains_key(&id) {
            self.tys.insert(id, EventTypeManager::new());
        }
        let keys: Vec<_> = self.tys.keys().cloned().collect();
        keys.into_iter().zip(self.tys.values_mut())
            .find(|(i, _)| *i == id)
            .unwrap()
            .1
            .with_handler(handler);
    }
}

pub struct Events;
impl Events {
    pub fn new() -> Self {
        Self
    }
    pub async fn stream<M: AnyMessage>(stream: impl MessageStream<M>) {
        events_mut().await.stream(stream)
    }
    pub async fn handler<M: AsyncHandlerMessage, R: AsyncHandlerResult>(handler: impl AsyncHandler<M, R>) {
        events_mut().await.handler(handler);
    }
    pub async fn listen() {
        StreamExt::collect::<Vec<_>>(Events).await;
    }
}
use lazy_static::lazy_static;
use async_std::sync::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use std::task::{
    Poll,
    Context,
};
lazy_static! {
    pub static ref EVENTS: Arc<RwLock<EventManager>> = Arc::new(RwLock::new(EventManager::new()));
}
async fn events() -> RwLockReadGuard<'static, EventManager> {
    EVENTS.read().await
}
async fn events_mut() -> RwLockWriteGuard<'static, EventManager> {
    EVENTS.write().await
}
fn try_events() -> Option<RwLockReadGuard<'static, EventManager>> {
    EVENTS.try_read()
}
fn try_events_mut() -> Option<RwLockWriteGuard<'static, EventManager>> {
    EVENTS.try_write()
}
impl Stream for Events {
    type Item=();
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if let Some(mut events) = try_events_mut() {
            Stream::poll_next(Pin::new(&mut *events), cx)
        } else {
            Poll::Pending
        }
    }
}
