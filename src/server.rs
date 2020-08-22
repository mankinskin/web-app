use async_std::{
    net::{
        TcpListener,
        SocketAddr,
        TcpStream,
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
    path::{
        Path,
    },
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
use crate::{
    Error,
};
pub struct TcpServer {
    _acceptor: TlsAcceptor,
    addr: SocketAddr,
}
impl TcpServer {
    pub async fn create_listener(&self) -> Result<TcpListener, Error> {
        Ok(TcpListener::bind(self.addr).await?)
    }
    pub fn new() -> Self {
        let config = ServerConfig::new(NoClientAuth::new());
        let mut _acceptor = TlsAcceptor::from(Arc::new(config));
        let addr = SocketAddr::from(([0,0,0,0], 8000));
        Self {
            _acceptor,
            addr,
        }
    }
    pub async fn handle_connection(&mut self, stream: TcpStream) -> Result<(), Error> {
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
        let pkg_path = "/home/linusb/git/binance-bot/client/pkg";
        let file_path = match req_path {
            path if path.is_empty() || path == "/" => "/index.html".to_string(),
            path => path.to_string(),
        };
        let file_path = async_std::path::PathBuf::from(format!("{}{}", pkg_path, file_path));
        println!("{}", file_path.to_string_lossy());
        Self::file_response(file_path).await
    }
}
