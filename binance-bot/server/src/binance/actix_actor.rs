
#[cfg(feature = "actix_server")]
use actix::{
    Actor,
    Context,
    Addr,
};
#[cfg(feature = "actix_server")]
use actix_web::ResponseError;

#[cfg(feature = "actix_server")]
impl ResponseError for Error {}
#[cfg(feature = "actix_server")]
impl Actor for Binance {
    type Context = Context<Self>;
}