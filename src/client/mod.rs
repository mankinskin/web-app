use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    Viewable,
    Init,
    auth::{
        *,
        self,
    },
};
use openlimits::{
    model::{
        Candle,
    },
};
use tracing::{
    debug,
};
pub mod chart;
use chart::{
    Chart,
};
pub mod websocket;
use websocket::{
    WebSocket,
};
pub mod router;
use router::Router;

fn init_tracing() {
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
}
fn get_host() -> Result<String, JsValue> {
    web_sys::window().unwrap().location().host()
}
#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_tracing();
    App::start("app",
        |url, orders| {
            let host = get_host().unwrap();
            debug!("Host: {}", host);
            Model {
                host: host.clone(),
                websocket: WebSocket::init(host, &mut orders.proxy(Msg::Websocket)),
                router: Router::init(url, &mut orders.proxy(Msg::Router)),
            }
        },
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
}
#[derive(Debug)]
pub struct Model {
    pub host: String,
    pub router: Router,
    pub websocket: WebSocket
}
#[derive(Clone, Debug)]
pub enum Msg {
    Websocket(websocket::Msg),
    Router(router::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        debug!("Root update");
        match msg {
            Msg::Router(msg) => {
                self.router.update(msg, &mut orders.proxy(Msg::Router));
            },
            Msg::Websocket(msg) => {
                self.websocket.update(msg, &mut orders.proxy(Msg::Websocket));
            },
        }
    }
}
impl Model {
    #[allow(unused)]
    fn mutation_observer(&self) {
        if let Some(node) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|document| document.get_element_by_id("graph-svg")) {
            log!("found node");
            let closure = wasm_bindgen::closure::Closure::new(|record: web_sys::MutationRecord| {
                log!("Mutation {}", record);
            });
            let function = js_sys::Function::from(closure.into_js_value());
            let observer = web_sys::MutationObserver::new(&function).unwrap();
            observer.observe(&node).unwrap();
        }
    }
    #[allow(unused)]
    async fn fetch_candles(url: &str) -> Result<Vec<Candle>, FetchError> {
        let url = format!("http://{}/api/price_history", url);
        seed::fetch::fetch(
            Request::new(url)
                .method(Method::Get)
            )
            .await?
            .check_status()?
            .json()
            .await
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Self::Msg> {
        //seed::log!("App redraw!");
        div![
            self.router.view().map_msg(Msg::Router),
        ]
    }
}
