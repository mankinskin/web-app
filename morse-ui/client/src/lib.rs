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
    Button(ButtonMsg),
    Click,
    Release
}
struct Root {
    button: Button,
}
impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ButtonMsg| {
            match msg {
                ButtonMsg::Click => Some(Msg::Click),
                ButtonMsg::Release => Some(Msg::Release),
            }
        });
        Self {
            button: Button,
        }
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Button(msg) => {
                self.button.update(msg, &mut orders.proxy(Msg::Button));
            },
            Self::Msg::Click => debug!("Click!"),
            Self::Msg::Release => debug!("Release!"),
        }
    }
}
impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        div![
            "Hello World",
            self.button,
        ]
    }
}

#[derive(Debug, Clone)]
enum ButtonMsg {
    Click,
    Release,
}
struct Button;

impl UpdateEl<Msg> for &Button {
    fn update_el(self, el: &mut El<Msg>) {
        self.view().map_msg(Msg::Button).update_el(el)
    }
}
impl Component for Button {
    type Msg = ButtonMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match &msg {
            Self::Msg::Click => {},
            Self::Msg::Release => {},
        }
        orders.notify(msg);
    }
}
impl Viewable for Button {
    fn view(&self) -> Node<<Self as Component>::Msg> {
        button![
            "Click!",
            ev(Ev::MouseDown, |_| {
                Self::Msg::Click       
            }),
            ev(Ev::MouseLeave, |_| {
                Self::Msg::Release       
            }),
            ev(Ev::MouseUp, |_| {
                Self::Msg::Release       
            }),
        ]
    }
}
