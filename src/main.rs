extern crate reqwest;
extern crate telegram_bot;
extern crate serde;
extern crate serde_json;
extern crate openlimits;
extern crate tokio;
extern crate futures;
extern crate async_std;
extern crate futures_core;
extern crate async_tls;
extern crate rustls;
extern crate async_h1;
extern crate http_types;

use futures_core::{
    stream::{
        Stream,
    },
};
use futures::{
    StreamExt,
};
use telegram_bot::{
    *,
    Error as TelegramError,
    Update as TelegramUpdate,
};
use openlimits::{
    errors::{
        OpenLimitError as BinanceError,
    },
    shared::{
        Result as OpenLimitResult,
    },
    binance::Binance,
    exchange::Exchange,
    model::{
        GetPriceTickerRequest,
        Ticker,
    },
};

use std::{
    convert::{
        AsRef,
    },
    path::{
        Path,
    },
};
use async_std::{
    io::{
        BufReader,
        prelude::{
            BufReadExt,
        },
    },
    net::{
        TcpListener,
        Incoming,
        TcpStream,
        SocketAddr,
    },
};
use rustls::{
    ServerConfig,
    NoClientAuth,
};
use async_tls::{
    TlsAcceptor,
};
use std::{
    sync::{
        Arc,
    },
    pin::Pin,
    task::Poll,
};
use http_types::{
    Request,
    Response,
    StatusCode,
    Body,
    mime::{
        Mime,
    },
};

fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
#[derive(Clone, Debug)]
enum CommandContext {
    Message(Message),
    CommandLine(String),
}
impl From<Message> for CommandContext {
    fn from(m: Message) -> CommandContext {
        Self::Message(m)
    }
}
impl From<String> for CommandContext {
    fn from(m: String) -> CommandContext {
        Self::CommandLine(m)
    }
}
#[derive(Clone, Debug)]
enum Update {
    Telegram(TelegramUpdate),
    CommandLine(String),
    TcpStream(TcpStream),
}
#[derive(Debug)]
enum Error {
    Telegram(TelegramError),
    Binance(BinanceError),
    AsyncIO(async_std::io::Error),
    Http(http_types::Error),
}
impl From<TelegramError> for Error {
    fn from(err: TelegramError) -> Self {
        Self::Telegram(err)
    }
}
impl From<BinanceError> for Error {
    fn from(err: BinanceError) -> Self {
        Self::Binance(err)
    }
}
impl From<async_std::io::Error> for Error {
    fn from(err: async_std::io::Error) -> Self {
        Self::AsyncIO(err)
    }
}
impl From<http_types::Error> for Error {
    fn from(err: http_types::Error) -> Self {
        Self::Http(err)
    }
}
struct CommandStream<'a> {
    pub telegram_stream: UpdatesStream,
    pub stdin: async_std::io::Stdin,
    pub incoming: Incoming<'a>,
}
impl<'a> Stream for CommandStream<'a> {
    type Item = Result<Update, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let rself = self.get_mut();
        let stdin = BufReader::new(&mut rself.stdin);
        let mut lines = stdin.lines();
        let cli_poll = Stream::poll_next(Pin::new(&mut lines), cx);
        if cli_poll.is_ready() {
            return cli_poll.map(|opt|
                opt.map(|result|
                    result.map(|line| Update::CommandLine(line))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        let telegram_poll = Stream::poll_next(Pin::new(&mut rself.telegram_stream), cx);
        if telegram_poll.is_ready() {
            return telegram_poll.map(|opt|
                opt.map(|result|
                    result.map(|update| Update::Telegram(update))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        let incoming_poll = Stream::poll_next(Pin::new(&mut rself.incoming), cx);
        if incoming_poll.is_ready() {
            return incoming_poll.map(|opt|
                opt.map(|result|
                    result.map(|stream| Update::TcpStream(stream))
                          .map_err(|err| Error::from(err))
                )
            );
        }
        Poll::Pending
    }
}
fn setup_binance_market() -> Binance {
    let binance_api_key = read_key_file("keys/binance_api");
    let binance_secret_key = read_key_file("keys/binance_secret");

    Binance::with_credential(&binance_api_key, &binance_secret_key, false)
}
fn setup_telegram_api() -> Api {
    let telegram_key = read_key_file("keys/telegram");
    Api::new(telegram_key)
}
struct Context {
    binance: Binance,
    telegram: Api,
}
impl Context {
    pub fn new() -> Self {
        let binance = setup_binance_market();
        let telegram = setup_telegram_api();
        Self {
            binance,
            telegram,
        }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        let config = ServerConfig::new(NoClientAuth::new());
        let mut acceptor = TlsAcceptor::from(Arc::new(config));
        let addr = SocketAddr::from(([0,0,0,0], 8000));
        let listener = TcpListener::bind(addr).await?;

        let mut stream = CommandStream {
            telegram_stream: self.telegram.stream(),
            stdin: async_std::io::stdin(),
            incoming: listener.incoming(),
        };
        while let Some(result) = stream.next().await {
            match result {
                Ok(update) => match update {
                    Update::Telegram(update) => self.telegram_update(update).await?,
                    Update::CommandLine(text) => self.run_command(CommandContext::from(text)).await?,
                    Update::TcpStream(stream) => self.handle_connection(&mut acceptor, stream).await?,
                },
                Err(err) => println!("{:#?}", err),
            }
        }
        Ok(())
    }
    async fn handle_connection(&mut self, _acceptor: &mut TlsAcceptor, stream: TcpStream) -> Result<(), Error> {
        println!("starting new connection from {}", stream.peer_addr()?);
        let stream = stream.clone();
        async_std::task::spawn(async {
            if let Err(e) = async_h1::accept(stream, |req| async move {
                Self::handle_request(req).await
            })
            .await {
                eprintln!("{}", e);
            }
        });
        Ok(())
    }
    async fn file_response<P: AsRef<Path>>(path: P) -> Result<Response, http_types::Error> {
        let mut res = Response::new(StatusCode::Ok);
        let mime = path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "wasm" => Some("application/wasm".to_string()),
                _ => Mime::from_extension(ext)
                    .map(|mime| mime.to_string())
            })
            .unwrap_or("text/plain".to_string());
        res.insert_header("Content-Type", mime);
        res.set_body(Body::from_file(path).await?);
        Ok(res)
    }
    async fn handle_request(req: Request) -> Result<Response, http_types::Error> {
        let req_path = req.url().path();
        //let pkg_path = "/home/linusb/.woz/rubot/pkg/app";
        let pkg_path = "/home/linusb/git/binance-bot/client/pkg";
        let file_path = match req_path {
            path if path.is_empty() || path == "/" => "/index.html".to_string(),
            path => path.to_string(),
        };
        let file_path = async_std::path::PathBuf::from(format!("{}{}", pkg_path, file_path));
        println!("{}", file_path.to_string_lossy());
        Self::file_response(file_path).await
    }
    async fn get_price(&mut self, symbol: String) -> OpenLimitResult<Ticker> {
        self.binance.get_price_ticker(&GetPriceTickerRequest {
            symbol,
            ..Default::default()
        }).await
    }
    async fn run_command(&mut self, context: CommandContext) -> Result<(), Error> {
        Ok(match context {
            CommandContext::Message(message) => {
                match message.kind.clone() {
                    MessageKind::Text { data, .. } => {
                        // Print received text message to stdout.
                        println!("<{}>: {}", &message.from.first_name, data);
                        let btc_price = self.get_price(data).await;
            
                        self.telegram.send(message.text_reply(format!(
                            "{:#?}", btc_price,
                        )))
                        .await?;
                    },
                    _ => {},
                }
            },
            CommandContext::CommandLine(text) => {
                // Print received text message to stdout.
                let btc_price = self.get_price(text).await;
                println!("{:#?}", btc_price);
            },
        })
    }
    async fn telegram_update(&mut self, update: TelegramUpdate) -> Result<(), Error> {
        Ok(
            match update.kind {
                UpdateKind::Message(message) => {
                    self.run_command(CommandContext::from(message)).await?
                },
                UpdateKind::EditedMessage(_message) => {},
                UpdateKind::ChannelPost(_post) => { },
                UpdateKind::EditedChannelPost(_post) => { },
                UpdateKind::InlineQuery(_query) => { },
                UpdateKind::CallbackQuery(_query) => { },
                UpdateKind::Error(_error) => { },
                UpdateKind::Unknown => { },
            }
        )
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    Context::new().run().await
}
