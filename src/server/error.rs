use crate::{
    server::{
        telegram,
        model,
    },
};

use openlimits::{
    errors::{
        OpenLimitError,
    },
};

#[derive(Debug)]
pub enum Error {
    Telegram(telegram::Error),
    OpenLimits(OpenLimitError),
    AsyncIO(async_std::io::Error),
    Clap(clap::Error),
    Model(model::Error),
    Tokio(tokio::task::JoinError),
    SerdeJson(serde_json::Error),
    WebSocket(String),
    Warp(warp::Error),
    Mpsc(futures::channel::mpsc::SendError),
    Multiple(Vec<Error>),
}

impl<E: Into<Error>> From<Vec<E>> for Error {
    fn from(errors: Vec<E>) -> Self {
        Self::Multiple(errors.into_iter().map(Into::into).collect())
    }
}
impl From<futures::channel::mpsc::SendError> for Error {
    fn from(err: futures::channel::mpsc::SendError) -> Self {
        Self::Mpsc(err)
    }
}
impl From<warp::Error> for Error {
    fn from(err: warp::Error) -> Self {
        Self::Warp(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}
impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        Self::Clap(err)
    }
}
impl From<telegram::Error> for Error {
    fn from(err: telegram::Error) -> Self {
        Self::Telegram(err)
    }
}
impl From<OpenLimitError> for Error {
    fn from(err: OpenLimitError) -> Self {
        Self::OpenLimits(err)
    }
}
impl From<async_std::io::Error> for Error {
    fn from(err: async_std::io::Error) -> Self {
        Self::AsyncIO(err)
    }
}
impl From<model::Error> for Error {
    fn from(err: model::Error) -> Self {
        Self::Model(err)
    }
}
impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Tokio(err)
    }
}
