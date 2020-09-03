use crate::{
    Update,
    Error,
    telegram,
    INTERVAL,
};
use telegram_bot::{
    UpdatesStream,
};
use futures_core::{
    stream::{
        Stream,
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
        SocketAddr,
    },
    sync::{
        Arc,
        RwLock,
    },
    stream::{
        Interval,
    },
};
use std::{
    pin::Pin,
    task::Poll,
};
use rustls::{
    ServerConfig,
    NoClientAuth,
};
use async_tls::{
    TlsAcceptor,
};

pub struct MessageStream {
    pub listener: Option<TcpListener>,
    pub stdin: async_std::io::Stdin,
    pub telegram_stream: Option<UpdatesStream>,
    pub interval: Arc<RwLock<Option<Interval>>>,
}
impl Stream for MessageStream {
    type Item = Result<Update, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let rself = self.get_mut();
        if let Some(mut interval) = rself.interval.try_write() {
            if let Some(interval) = &mut *interval {
                let interval_poll = Stream::poll_next(Pin::new(interval), cx);
                if interval_poll.is_ready() {
                    return Poll::Ready(Some(Ok(Update::Interval)));
                }
            }
        }
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
        if let Some(listener) = &mut rself.listener {
            let incoming_poll = Stream::poll_next(Pin::new(&mut listener.incoming()), cx);
            if incoming_poll.is_ready() {
                return incoming_poll.map(|opt|
                    opt.map(|result|
                        match result {
                            Ok(stream) => Ok(Update::TcpStream(stream)),
                            Err(err) => {
                                rself.listener = None;
                                Err(Error::from(err))
                            },
                        }
                    )
                );
            }
        }
        if let Some(telegram) = &mut rself.telegram_stream {
            let telegram_poll = Stream::poll_next(Pin::new(telegram), cx);
            if telegram_poll.is_ready() {
                return telegram_poll.map(|opt|
                    opt.map(|result|
                        match result {
                            Ok(update) => Ok(Update::Telegram(update)),
                            Err(err) => {
                                rself.telegram_stream = None;
                                Err(Error::from(err))
                            },
                        }
                    )
                );
            }
        }
        Poll::Pending
    }
}
impl MessageStream {
    pub async fn init() -> Result<Self, Error> {
        let config = ServerConfig::new(NoClientAuth::new());
        let mut _acceptor = TlsAcceptor::from(Arc::new(config));
        let addr = SocketAddr::from(([0,0,0,0], 8000));

        let listener = TcpListener::bind(addr).await?;
        Ok(MessageStream {
            listener: Some(listener),
            stdin: async_std::io::stdin(),
            telegram_stream: Some(telegram().await.stream()),
            interval: INTERVAL.clone(),
        })
    }
}
