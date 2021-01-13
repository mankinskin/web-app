use crate::{
	binance::{
		binance,
		BinanceActor,
		PriceHistoryRequest,
	},
	telegram::TelegramActor,
	*,
};
use app_model::{
	auth::{
		login,
		register,
		Credentials,
	},
	user::User,
};
use async_std::net::SocketAddr;
use chrono::Utc;
use shared::{
	PriceSubscription,
};
use tide::{
	Body,
	Endpoint,
	Middleware,
	Request,
	Response,
};
use tide_rustls::TlsListener;
use tide_tracing::TraceMiddleware;
use tide_websockets::{
	WebSocket,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
	warn,
};

macro_rules! client_file {
	($path:expr) => {
		|_: tide::Request<()>| {
			async move {
				let body = tide::Body::from_file(format!("{}/{}", CLIENT_PATH, $path)).await?;
				Ok(tide::Response::from(body))
			}
		}
	};
}
macro_rules! index {
	() => {
		client_file!("index.html")
	};
}
fn wss_middleware() -> impl Middleware<()> + Endpoint<()> {
	WebSocket::new(async move |_, ws| {
		websocket::connection(ws).await;
		Ok(())
	})
}
async fn wss_handler(request: Request<()>) -> tide::Result {
	wss_middleware().call(request).await
}
fn root() -> std::io::Result<tide::Server<()>> {
	let mut root = tide::new();
	root.at("/").get(client_file!("index.html"));
	root.at("/favicon.ico").get(client_file!("favicon.ico"));
	root.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH))?;
	Ok(root)
}
#[async_trait::async_trait]
trait ServeSession<'db, DB>
    where DB: Database<'db, User> + 'db,
{
    type Api;
    type Response;
    type Request;
    fn serve(api: &mut Self::Api) -> std::io::Result<()>;
    async fn login_handler(mut req: Self::Request) -> Self::Response;
    async fn logout_handler(mut req: Self::Request) -> Self::Response;
    async fn registration_handler(mut req: Self::Request) -> Self::Response;
}
#[async_trait::async_trait]
impl<DB> ServeSession<'static, DB> for TideServer
    where DB: Database<'static, User> + 'static,
{
    type Api = tide::Server<()>;
    type Response = tide::Result;
    type Request = Request<()>;
    fn serve(api: &mut Self::Api) -> std::io::Result<()> {
	    api.at("/login").post(<Self as ServeSession<'static, DB>>::login_handler);
	    api.at("/register").post(<Self as ServeSession<'static, DB>>::registration_handler);
	    api.at("/logout").post(<Self as ServeSession<'static, DB>>::logout_handler);
        Ok(())
    }
    async fn login_handler(mut req: Self::Request) -> Self::Response {
    	let credentials: Credentials = req.body_json().await?;
    	match login::<DB>(credentials).await {
    		Ok(session) => {
    			req.session_mut()
    				.insert("session", session)
    				.map(|_| Response::new(200))
    				.map_err(|e| tide::Error::from_str(500, e.to_string()))
    		}
    		Err(e) => Err(e),
    	}
    }
    async fn logout_handler(mut req: Self::Request) -> Self::Response {
    	req.session_mut().remove("session");
    	Ok(Response::new(200))
    }
    async fn registration_handler(mut req: Self::Request) -> Self::Response {
    	let user: User = req.body_json().await?;
    	match register::<database::Schema>(user).await {
    		Ok(_session) => Ok(Response::new(200)),
    		Err(e) => Err(tide::Error::from_str(500, e.to_string())),
    	}
    }
}
#[async_trait::async_trait]
trait ServeTable<'db, T, DB>
    where T: TableRoutable + DatabaseTable<'db, DB> + 'db,
          DB: Database<'db, T> + 'db,
{
    type Api;
    type Response;
    type Request;
    fn serve(api: &mut Self::Api) -> std::io::Result<()>;
    async fn post_handler(req: Self::Request) -> Self::Response;
    async fn get_handler(req: Self::Request) -> Self::Response;
    async fn get_list_handler(req: Self::Request) -> Self::Response;
    async fn delete_handler(req: Self::Request) -> Self::Response;
}
use database::Schema;
use database_table::{
    Database,
    DatabaseTable,
    TableRoutable,
};
use enum_paths::AsPath;
use std::fmt::Debug;

struct TideServer;

impl TideServer {
    pub fn serve<T>(api: &mut tide::Server<()>) -> std::io::Result<()> {
	    Ok(<TideServer as ServeTable<'_, PriceSubscription, Schema>>::serve(api)?)
    }
    pub fn auth(api: &mut tide::Server<()>) -> std::io::Result<()> {
	    Ok(<TideServer as ServeSession<'_, Schema>>::serve(api)?)
    }
}

#[async_trait::async_trait]
impl<T, DB> ServeTable<'static, T, DB> for TideServer
    where T: TableRoutable + DatabaseTable<'static, DB> + Debug + 'static,
          DB: Database<'static, T> + 'static,
{
    type Api = tide::Server<()>;
    type Response = tide::Result<Body>;
    type Request = Request<()>;

    fn serve(api: &mut Self::Api) -> std::io::Result<()> {
        let route = T::table_route().as_path();
    	api.at(&format!("/{}", route))
    		.get(<Self as ServeTable<'static, T, DB>>::get_list_handler)
    		.post(<Self as ServeTable<'static, T, DB>>::post_handler)
    		.delete(<Self as ServeTable<'static, T, DB>>::delete_handler);
    	api.at(&format!("/{}/:id", route))
            .get(<Self as ServeTable<'static, T, DB>>::get_handler);
    	Ok(())
    }
    async fn post_handler(mut req: Self::Request) -> Self::Response {
    	let s: T = req.body_json().await?;
    	let id = T::insert(s);
    	let body = Body::from_json(&id)?;
    	debug!("{:#?}", body);
    	Ok(body)
    }
    async fn get_handler(req: Self::Request) -> Self::Response {
    	let id: rql::Id<T> = req.param("id")?.parse()?;
    	let r = T::get(id);
    	Ok(Body::from_json(&r)?)
    }
    async fn get_list_handler(_req: Self::Request) -> Self::Response {
    	debug!("Get subscription list handler");
    	let list = T::get_all();
    	debug!("Result: {:?}", list);
    	Ok(Body::from_json(&list)?)
    }
    async fn delete_handler(req: Self::Request) -> Self::Response {
    	let id: rql::Id<T> = req.param("id")?.parse()?;
    	let r = T::delete(id);
    	Ok(Body::from_json(&r)?)
    }
}

fn api() -> std::io::Result<tide::Server<()>> {
	let mut api = tide::new();
    TideServer::auth(&mut api)?;
    TideServer::serve::<PriceSubscription>(&mut api)?;
	api.at("/price_history").nest(price_api()?);
	Ok(api)
}
async fn price_history_handler(_: Request<()>) -> tide::Result<Body> {
	match binance()
		.await
		.get_symbol_price_history(PriceHistoryRequest {
			market_pair: "SOLBTC".into(),
			interval: Some(openlimits::model::Interval::OneHour),
			paginator: None,
		})
		.await
	{
		Ok(data) => Ok(Body::from_json(&data)?),
		Err(e) => Err(tide::Error::from_str(500, e.to_string())),
	}
}
fn price_api() -> std::io::Result<tide::Server<()>> {
	let mut api = tide::new();
	api.at("/").get(price_history_handler);
	Ok(api)
}

fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
	tide::sessions::SessionMiddleware::new(
		tide::sessions::MemoryStore::new(),
		session::generate_secret().as_bytes(),
	)
	.with_cookie_name("session")
	.with_session_ttl(Some(std::time::Duration::from_secs(
		session::EXPIRATION_SECS as u64,
	)))
}

pub async fn run() -> std::io::Result<()> {
	let _telegram_actor = actor_sys_mut()
		.await
		.actor_of::<TelegramActor>("telegram-actor")
		.unwrap();
	let _binance_actor = actor_sys_mut()
		.await
		.actor_of::<BinanceActor>("binance-actor")
		.unwrap();

	let mut server = tide::new();
	server.with(TraceMiddleware::new());
	server.with(session_middleware());
	server.with(tide::utils::Before(async move |mut request: Request<()>| {
		let session = request.session_mut();
		if let Some(expiry) = session.expiry() {
			// time since expiry or (negative) until
			let dt = (Utc::now() - *expiry).num_seconds();
			if dt >= session::STALE_SECS as i64 {
				// expired and stale
				session.destroy()
			} else if dt >= 0 {
				// expired and not stale
				session.regenerate()
			}
		}
		request
	}));
	server.at("/").nest(root()?);
	server.at("/api").nest(api()?);
	server.at("/subscriptions").get(index!());
	server.at("/login").get(index!());
	server.at("/register").get(index!());
	server.at("/wss").get(wss_handler);

	let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
	server
		.listen(
			TlsListener::build()
				.addrs(addr)
				.cert(keys::to_key_path("tls.crt"))
				.key(keys::to_key_path("tls.key")),
		)
		.await?;
	Ok(())
}
