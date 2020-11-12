use crate::{
    CLIENT_PATH,
    database,
    keys,
    binance::PriceHistoryRequest,
};
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    warn,
    trace,
};
use app_model::{
    user::User,
    auth::{
        login,
        register,
        Credentials,
    },
};
use async_std::net::SocketAddr;
use warp::reply::Reply;
use warp::Filter;

pub async fn run() -> std::io::Result<()> {
    //let websocket = warp::path("wss")
    //    .and(warp::ws())
    //    .map(move |ws: warp::ws::Ws| {
    //        debug!("Websocket connection request");
    //        ws.on_upgrade(websocket::connection)
    //    });
    let price_history = warp::get()
        .and(warp::path!("api" / "price_history"))
        .and_then(|| {
            async {
                crate::binance::Binance::
                    get_symbol_price_history(PriceHistoryRequest {
                        market_pair: "SOLBTC".into(),
                        interval: Some(openlimits::model::Interval::OneHour),
                        paginator: None,
                    })
                    .await
                    .map(|data| serde_json::to_string(&data).unwrap())
                    .map_err(|_err| warp::reject::not_found())
            }
        });
    let login = warp::post()
        .and(warp::path!("api" / "login"))
        .and(warp::body::json())
        .and_then(|credentials: Credentials| {
            async {
                Ok(match login::<database::Schema>(credentials).await {
                    Ok(session) => warp::reply::json(&session).into_response(),
                    Err(_status) => warp::reply::with_status("",
                        warp::http::StatusCode::from_u16(500).unwrap()
                        ).into_response(),
                }) as Result<warp::reply::Response, core::convert::Infallible>
            }
        });
    let register = warp::post()
        .and(warp::path!("api" / "register"))
        .and(warp::body::json())
        .and_then(|user: User| {
            async {
                Ok(match register::<database::Schema>(user).await {
                    Ok(session) => warp::reply::json(&session).into_response(),
                    Err(_status) => warp::reply::with_status("",
                        warp::http::StatusCode::from_u16(500).unwrap()
                    ).into_response(),
                }) as Result<warp::reply::Response, core::convert::Infallible>
            }
        });
    let api = price_history.or(login).or(register);
    let pkg_dir = warp::fs::dir(format!("{}/pkg", CLIENT_PATH.to_string()));
    let logger = warp::log::custom(|info| {
        debug!(
            "request from {:?}: {} {} {}ms {}",
            info.remote_addr(),
            info.method(),
            info.path(),
            info.elapsed().as_millis(),
            info.status(),
        )
    });
    let routes = api
        .or(pkg_dir)
        .or(warp::fs::file(format!("{}/pkg/index.html", CLIENT_PATH)))
        .with(logger);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let server = warp::serve(routes)
        .tls()
        .cert_path(keys::to_key_path("tls.crt"))
        .key_path(keys::to_key_path("tls.key"));
    info!("Starting Server");
    server.run(addr).await;
    Ok(())
}
