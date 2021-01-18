use components::{
	Component,
	Init,
	Viewable,
};
use seed::{
	prelude::*,
	*,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
};
fn init_tracing() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	debug!("Tracing initialized.");
	debug!("Debug logs enabled.");
	info!("Info logs enabled.");
	trace!("Trace logs enabled.");
	error!("Error logs enabled.");
}
#[wasm_bindgen(start)]
pub fn render() {
    init_tracing();
    debug!("Starting App");
    App::start(
        "app",
        Root::init,
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
}
#[derive(Debug, Clone)]
enum Msg {
}
struct Root {
}
impl Init<Url> for Root {
    fn init(_url: Url, _orders: &mut impl Orders<Msg>) -> Self {
        Self {
        }
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Self::Msg>) {
        match msg {
        }
    }
}
impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        div![
            "Hello World"
        ]
    }
}
