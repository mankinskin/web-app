use crate::{
    binance::Binance,
    subscriptions::Subscriptions,
    shared::{
        PriceHistoryRequest,
    },
};
use async_std::{
    io::{
        prelude::BufReadExt,
        BufReader,
        Stdin,
    },
    sync::{
        Arc,
        RwLock,
    },
};
use clap::{
    App,
    Arg,
};
use futures_core::stream::Stream;
use std::{
    pin::Pin,
    task::Poll,
};
use tracing::{
    debug,
    error,
};

use lazy_static::lazy_static;
lazy_static! {
    pub static ref STDIN: Arc<RwLock<Stdin>> = Arc::new(RwLock::new(async_std::io::stdin()));
}
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
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
                    let id = Subscriptions::add_subscription(PriceHistoryRequest::from(symbol.to_string()))
                        .await
                        .map_err(|e| e.to_string())?;
                    //crate::server::interval::set(interval(Duration::from_secs(1)));
                    format!("Ok {}", id)
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
use actix_interop::{
    FutureInterop,
};
pub struct CommandLine;
impl CommandLine {
    pub async fn init() -> Addr<Self> {
        Self::create(|_| Self)
    }
}
impl Actor for CommandLine {
    type Context = Context<Self>;
   fn started(&mut self, ctx: &mut Context<Self>) {
       // add stream
       Self::add_stream(Self, ctx);
   }
}
impl StreamHandler<Msg> for CommandLine {
    fn handle(
        &mut self,
        msg: Msg,
        ctx: &mut Self::Context,
    ) {
        ctx.notify(msg);
    }
}
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
