extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate http;
extern crate anyhow;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate rql;
extern crate plans;
extern crate budget;
extern crate updatable;
extern crate database;

use plans::{
    credentials::*,
    user::*,
};
use seed::{
    *,
    prelude::*,
    fetch::*,
};
use futures::{
    Future,
};

struct Model {
    count: i32,
    what_we_count: String,
    login: login::Model,
    session: Option<UserSession>,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        Self {
            count: 0,
            what_we_count: "click".into(),
            login: login::Model::default(),
            session: None,
        }
    }
}

#[derive(Clone)]
enum Msg {
    Increment,
    Decrement,
    Login(login::Msg),
    LoginResponse(Result<UserSession, FailReason<UserSession>>),
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
fn login_request(credentials: Credentials)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/users/login")
        .method(Method::Post)
        .send_json(&credentials)
        .fetch_json_data(move |data_result: ResponseDataResult<UserSession>| {
            Msg::LoginResponse(data_result)
        })
}
/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.count += 1,
        Msg::Decrement => model.count -= 1,
        Msg::Login(msg) => {
            login::update(msg.clone(), &mut model.login);
            match msg {
                login::Msg::Login => {
                    orders.perform_cmd(login_request(model.login.credentials()));
                },
                _ => {}
            }
        },
        Msg::LoginResponse(res) => {
            match res {
                Ok(session) => {
                    seed::log!(session);
                    model.session = Some(session);
                },
                Err(_reason) => {},
            }
        },
    }
}
mod login {
    use seed::{
        *,
        prelude::*,
    };
    use plans::{
        credentials::*,
    };
    /// credential input component
    #[derive(Clone, Default)]
    pub struct Model {
        credentials: Credentials,
    }
    impl Model {
        pub fn credentials(&self) -> Credentials {
            self.credentials.clone()
        }
    }
    #[derive(Clone)]
    pub enum Msg {
        ChangeUsername(String),
        ChangePassword(String),
        Login,
    }
    pub fn update(msg: Msg, model: &mut Model) {
        match msg {
            Msg::ChangeUsername(u) => model.credentials.username = u,
            Msg::ChangePassword(p) => model.credentials.password = p,
            Msg::Login => {},
        }
    }
    pub fn view(model: &Model) -> Node<Msg> {
        div![
            p!["Username"],
            input![
                attrs!{
                    At::Placeholder => "Username",
                    At::Value => model.credentials.username,
                },
                input_ev(Ev::Input, Msg::ChangeUsername)
            ],
            p!["Password"],
            input![
                attrs!{
                    At::Type => "password",
                    At::Placeholder => "Password",
                    At::Value => model.credentials.password,
                },
                input_ev(Ev::Input, Msg::ChangePassword)
            ],
            button![simple_ev(Ev::Click, Msg::Login), "Login"],
        ]
    }

}

/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    let plural = if model.count == 1 {""} else {"s"};

    // Attrs, Style, Events, and children may be defined separately.
    let outer_style = style!{
            St::Display => "flex";
            St::FlexDirection => "column";
            St::TextAlign => "center"
    };

    div![ outer_style,
        h1![ "The Grand Total" ],
        div![
            style!{
                // Example of conditional logic in a style.
                St::Color => if model.count > 4 {"purple"} else {"gray"};
                St::Border => "2px solid #004422";
                St::Padding => unit!(20, px);
            },
            // We can use normal Rust code and comments in the view.
            h3![ format!("{} {}{} so far", model.count, model.what_we_count, plural) ],
            button![ simple_ev(Ev::Click, Msg::Increment), "+" ],
            button![ simple_ev(Ev::Click, Msg::Decrement), "-" ],

            // Optionally-displaying an element
            if model.count >= 10 { h2![ style!{St::Padding => px(50)}, "Nice!" ] } else { empty![] }
        ],
        login::view(&model.login)
            .map_msg(Msg::Login),
    ]
}


#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .build_and_start();
}
