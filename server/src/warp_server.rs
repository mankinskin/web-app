use crate::{
    CLIENT_PATH,
    database,
    keys,
    binance::PriceHistoryRequest,
    websocket,
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
use warp::reply::Reply;
use warp::Filter;
use riker::actors::*;
use async_std::{
    net::SocketAddr,
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};
use shared::{
    PriceSubscription,
};
use lazy_static::lazy_static;
lazy_static! {
    static ref ACTOR_SYS: Arc<RwLock<ActorSystem>> = Arc::new(RwLock::new(ActorSystem::new().unwrap()));
}
pub async fn actor_sys() -> RwLockReadGuard<'static, ActorSystem> {
    ACTOR_SYS.read().await
}
pub async fn actor_sys_mut() -> RwLockWriteGuard<'static, ActorSystem> {
    ACTOR_SYS.write().await
}
pub async fn run() -> std::io::Result<()> {
    let websocket =
        warp::path("wss")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            debug!("Websocket connection request");
            ws.on_upgrade(async move |ws: warp::ws::WebSocket| {
                websocket::websocket_session(ws).await
            })
        });
    let pkg_dir = warp::fs::dir(format!("{}/pkg", CLIENT_PATH.to_string()));
    let favicon = warp::path::path("favicon.ico").and(warp::fs::file(format!("{}/favicon.ico", CLIENT_PATH)));

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
    let price_history = warp::get()
        .and(warp::path!("price_history"))
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
        .and(warp::path!("login"))
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
        .and(warp::path!("register"))
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
    let post_subscription = warp::post()
        .and(warp::body::json())
        .and_then(async move |sub: PriceSubscription| {
            debug!("{:#?}", sub);
            Ok(String::new()) as Result<String, std::convert::Infallible>
        });
    let subscriptions = warp::path!("subscriptions")
        .and(post_subscription);
    
    let api = warp::path!("api" / ..)
        .and(price_history
            .or(login)
            .or(register)
            .or(subscriptions)
        );
    let index_file = warp::fs::file(format!("{}/index.html", CLIENT_PATH));
    let index_page = warp::path::end().and(index_file.clone());
    let pages = index_page
        .or(warp::path("subscriptions").and(index_file.clone()))
        .or(warp::path("login").and(index_file.clone()))
        .or(warp::path("register").and(index_file.clone()));
    let files = pages
        .or(favicon)
        .or(pkg_dir);
    let routes =
    warp::cookie::optional("session")
        .and(
            api
                .or(websocket)
                .or(files)
                .map(Reply::into_response)
        )
        .and_then(async move |session: Option<String>, reply: warp::reply::Response| {
            let set_new_session = if let Some(id) = session {
                debug!("Request for session {}", id);
                let session = crate::session::get_session(&id).await;
                if let Some(session) = session {
                    if session.is_stale() {
                        debug!("Session stale, recreating.");
                        true
                    } else {
                        false
                    }
                } else {
                    debug!("Invalid session. Creating new one.");
                    true
                }
            } else {
                debug!("Request with no session ID");
                true
            };
            Ok(if set_new_session {
                let session = crate::session::create_session().await;
                debug!("Setting header Set-Cookie {}", session.id);
                warp::reply::with_header(reply, "Set-Cookie", session.cookie_string()).into_response()
            } else {
                reply
            }) as Result<_, std::convert::Infallible>
        })
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
