use crate::{
    binance::{
        Binance,
        PriceHistoryRequest,
    },
    shared::{
        subscriptions::PriceSubscription,
    },
};
#[cfg(feature = "actix_server")]
use crate::{
    subscriptions::{
        SubscriptionsActor,
    },
};
use async_std::{
    io::{
        Stdin,
    },
    sync::{
        Arc,
        RwLock,
    },
};
#[cfg(feature = "actix_server")]
use async_std::{
    io::{
        prelude::BufReadExt,
        BufReader,
    },
};
use clap::{
    App,
    Arg,
};
#[cfg(feature = "actix_server")]
use futures_core::stream::Stream;
#[cfg(feature = "actix_server")]
use std::{
    pin::Pin,
    task::Poll,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
};

use lazy_static::lazy_static;
lazy_static! {
    pub static ref STDIN: Arc<RwLock<Stdin>> = Arc::new(RwLock::new(async_std::io::stdin()));
}
#[cfg_attr(feature = "actix_server", derive(Message))]
#[cfg_attr(feature = "actix_server", rtype(result = "()"))]
#[derive(Clone, Debug)]
pub enum Msg {
    Line(String),
}
#[derive(Clone, Debug)]
pub struct Error(String);
impl From<String> for Error {
    fn from(err: String) -> Self {
        Self(err)
    }
}
impl ToString for Error {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub async fn run_command(text: String) -> Result<String, Error> {
    debug!("Running command...");
    let mut args = vec![""];
    args.extend(text.split(" "));
    let app = App::new("")
        .subcommand(
            App::new("price")
                .arg(Arg::with_name("symbol").help("The Market Symbol to get the price of")),
        )
        .subcommand(
            App::new("history")
                .arg(Arg::with_name("symbol").help("The Market symbol to get the history of")),
        )
        .subcommand(
            App::new("watch").arg(Arg::with_name("symbol").help("The Market symbol to watch")),
        )
        .get_matches_from_safe(args);
    Ok(match app {
        Ok(app) => {
            if let Some(price_app) = app.subcommand_matches("price") {
                if let Some(symbol) = price_app.value_of("symbol") {
                    let price = Binance::get_symbol_price(symbol)
                        .await
                        .map_err(|e| e.to_string())?;
                    format!("{:#?}", price)
                } else {
                    price_app.usage().to_string()
                }
            } else if let Some(history_app) = app.subcommand_matches("history") {
                if let Some(symbol) = history_app.value_of("symbol") {
                    let price_history = 
                        Binance::get_symbol_price_history(PriceHistoryRequest {
                            market_pair: symbol.to_string().clone(),
                            interval: None,
                            paginator: None,
                        })
                        .await
                        .map_err(|e| e.to_string())?;
                    format!("{:#?}", price_history)
                } else {
                    history_app.usage().to_string()
                }
            } else if let Some(watch_app) = app.subcommand_matches("watch") {
                if let Some(symbol) = watch_app.value_of("symbol") {
                    start_subscription(PriceSubscription::from(symbol.to_string())).await?
                } else {
                    watch_app.usage().to_string()
                }
            } else {
                app.usage().to_string()
            }
        }
        Err(err) => format!("{}", err),
    })
}

#[cfg(feature = "actix_server")]
async fn start_subscription(subscription: PriceSubscription) -> Result<String, Error>{
    let id = SubscriptionsActor::add_subscription(subscription)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Ok {:#?}", id))
}
#[cfg(not(feature = "actix_server"))]
async fn start_subscription(_subscription: PriceSubscription) -> Result<String, Error>{
    unimplemented!()
}

#[cfg(feature = "actix_server")]
use actix::{
    Actor,
    Handler,
    Context,
    Addr,
    ResponseActFuture,
    StreamHandler,
    AsyncContext,
    Message,
};
#[cfg(feature = "actix_server")]
use actix_interop::{
    FutureInterop,
};
pub struct CommandLine;
impl CommandLine {
    #[cfg(feature = "actix_server")]
    pub async fn init() -> Addr<Self> {
        Self::create(|_| Self)
    }
}
#[cfg(feature = "actix_server")]
impl Actor for CommandLine {
    type Context = Context<Self>;
   fn started(&mut self, ctx: &mut Context<Self>) {
       // add stream
       Self::add_stream(Self, ctx);
   }
}
#[cfg(feature = "actix_server")]
impl StreamHandler<Msg> for CommandLine {
    fn handle(
        &mut self,
        msg: Msg,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
#[cfg(feature = "actix_server")]
impl Handler<Msg> for CommandLine {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        msg: Msg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        async move {
            match msg {
                Msg::Line(line) => match run_command(line).await {
                    Ok(text) => println!["{}", text],
                    Err(e) => error!("{:#?}", e),
                },
            }
        }.interop_actor_boxed(self)
    }
}
#[cfg(feature = "actix_server")]
impl Stream for CommandLine {
    type Item = Msg;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        if let Some(mut stdin) = STDIN.try_write() {
            let stdin = BufReader::new(&mut *stdin);
            let mut lines = stdin.lines();
            let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
            if cli_poll.is_ready() {
                //debug!("CLI poll ready");
                if let Poll::Ready(Some(result)) = cli_poll {
                    match result {
                        Ok(line) => { return Poll::Ready(Some(Msg::Line(line))); },
                        Err(e) => error!("{:#?}", e),
                    }
                }
            }
        }
        Poll::Pending
    }
}
