use crate::{
    Error,
};
use async_std::{
    stream::Interval,
    sync::{
        Arc,
        RwLock,
    },
};
use futures_core::stream::Stream;
use lazy_static::lazy_static;
use std::{
    pin::Pin,
    task::Poll,
};
lazy_static! {
    pub static ref INTERVAL: Arc<RwLock<Option<Interval>>> = Arc::new(RwLock::new(None));
}
pub fn set(new: Interval) {
    crate::server::interval::INTERVAL
        .try_write()
        .unwrap()
        .get_or_insert(new);
}
pub struct IntervalStream;
pub enum Msg {
    Tick,
}
impl Stream for IntervalStream {
    type Item = Result<Msg, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        if let Some(mut interval) = INTERVAL.try_write() {
            if let Some(interval) = &mut *interval {
                return Stream::poll_next(Pin::new(interval), cx)
                    .map(|opt| opt.map(|_| Ok(Msg::Tick)));
            }
        }
        Poll::Pending
    }
}
