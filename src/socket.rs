use std::{
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
pub struct TcpSocket;
impl TcpSocket {
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
    async fn handle_file_request(path: String) -> Result<Response, http_types::Error> {
        let pkg_path = "/home/linusb/git/binance-bot/pkg";
        let file_path = match &path {
            path if path.is_empty() || path == "/" => "/index.html".to_string(),
            path => path.to_string(),
        };
        let file_path = async_std::path::PathBuf::from(format!("{}{}", pkg_path, file_path));
        println!("{}", file_path.to_string_lossy());
        Self::file_response(file_path).await
    }
    async fn get_price_history_response() -> Result<Response, http_types::Error> {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(
            Body::from_json(
                &crate::binance()
                    .await
                    .get_symbol_price_history(crate::binance::PriceHistoryRequest {
                        market_pair: "SOLBTC".to_string(),
                        interval: None,
                        paginator: None,
                    })
                    .await
                    .map_err(|e|
                        http_types::Error::from_str(
                            StatusCode::InternalServerError,
                            format!("{:?}", e)
                        )
                    )?
            )?
        );
        Ok(res)
    }
    async fn handle_api_request(path: String) -> Result<Response, http_types::Error> {
        let path = path.strip_prefix("/api").unwrap();
        match path {
            "/price_history" => Self::get_price_history_response().await,
            _ => Err(http_types::Error::from_str(http_types::StatusCode::NotFound, "Invalid api call").into()),
        }
    }
    pub async fn handle_request(req: Request) -> Result<Response, http_types::Error> {
        let req_path = req.url().path();
        match req_path {
            path if path.starts_with("/api") => {
                Self::handle_api_request(path.to_string()).await
            },
            path => Self::handle_file_request(path.to_string()).await,
        }
    }
}
