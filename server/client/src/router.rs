use yew::{
    *,
};
use yew_router::{
    route::Route,
    service::RouteService,
    Switch,
};
use crate::{
    user_profile::*,
    note::*,
    login::*,
    signup::*,
    tasks::*,
};
use url::{
    *,
};
use plans::{
    user::*,
};
use rql::{
    *,
};

#[derive(Switch, Clone, Debug)]
pub enum ClientRoute {
    #[to = "/tasks"]
    Tasks,
    #[to = "/note"]
    Note,
    #[to = "/user"]
    User,
    #[to = "/login"]
    Login,
    #[to = "/signup"]
    Signup,
    #[to = "/"]
    Index,
}

impl ToString for ClientRoute {
    fn to_string(&self) -> String {
        match self {
            ClientRoute::Index => format!("/"),
            ClientRoute::User => format!("/user"),
            ClientRoute::Note => format!("/note"),
            ClientRoute::Login => format!("/login"),
            ClientRoute::Signup => format!("/signup"),
            ClientRoute::Tasks => format!("/tasks"),
        }
    }
}


pub type AccessToken = String;

pub enum Msg {
    //RouteChanged(Route<()>),
    ChangeRoute(ClientRoute),
    SetUser(Id<User>),
    UnsetUser,
    SetToken(AccessToken),
    UnsetToken,
}

pub struct ClientRouter {
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
    user: Option<Id<User>>,
    token: Option<AccessToken>,
}
impl ClientRouter {
    fn change_route(&self, route: ClientRoute) -> Callback<ClickEvent> {
        self.link.callback(move |_| {
            Msg::ChangeRoute(route.clone())
        })
    }
    fn user_setter(&self) -> Callback<Id<User>> {
        self.link.callback(move |id| {
            Msg::SetUser(id)
        })
    }
    fn user_unsetter(&self) -> Callback<()> {
        self.link.callback(move |_| {
            Msg::UnsetUser
        })
    }
    pub fn set_user(&mut self, id: Id<User>) {
        self.user = Some(id);
    }
    pub fn unset_user(&mut self) {
        self.user = None;
    }
    pub fn get_current_user(&self) -> Option<Id<User>> {
        self.user
    }
    fn token_setter(&self) -> Callback<AccessToken> {
        self.link.callback(move |token| {
            Msg::SetToken(token)
        })
    }
    fn token_unsetter(&self) -> Callback<()> {
        self.link.callback(move |_| {
            Msg::UnsetToken
        })
    }
    pub fn set_token(&mut self, token: AccessToken) {
        self.token = Some(token);
    }
    pub fn unset_token(&mut self) {
        self.token = None;
    }
    pub fn get_current_token(&self) -> Option<AccessToken> {
        self.token.clone()
    }
}

impl Component for ClientRouter {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        console!(log, format!("ClientRouter::create"));
        let route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        //let callback = link.callback(|route| Msg::RouteChanged(route));
        //route_service.register_callback(callback);

        Self {
            route_service,
            route,
            link,
            user: None,
            token: None,
        }
    }
    fn view(&self) -> Html {
        console!(log, format!("view: self.route {:#?}", self.route));
        html! {
            <div>
                <nav class="menu">
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Index) > {"Index"} </button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::User) > {"User"} </button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Note) > {"Note"} </button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Login) > {"Log In"} </button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Signup) > {"Sign Up"} </button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Tasks) > {"Tasks"} </button>
                </nav>
                <div>{
                    match ClientRoute::switch(self.route.clone()) {
                        None => html!{ <p>{"404"}</p> },
                        Some(ClientRoute::Index) => html!{
                            <div>
                                <p>{"Index"}</p>
                                <a href="/tasks">{"Tasks"}</a>
                            </div>
                        },
                        Some(ClientRoute::Signup) => html!{
                            <Signup
                                signup={Url::parse("http://localhost:8000/signup").unwrap()}
                                user={None}
                            />
                        },
                        Some(ClientRoute::Login) => html!{
                            <Login
                                login={Url::parse("http://localhost:8000/login").unwrap()}
                                user_setter={self.user_setter()}
                                token_setter={self.token_setter()}
                            />
                        },
                        Some(ClientRoute::User) => html!{
                            <UserProfileView
                                user={Url::parse("http://localhost:8000/api/users").unwrap()}
                            />
                        },
                        Some(ClientRoute::Note) => html!{
                            <NoteEditor
                                note={Url::parse("http://localhost:8000/api/notes").unwrap()}
                            />
                        },
                        Some(ClientRoute::Tasks) => html!{
                            <TasksView
                                tasks={Url::parse("http://localhost:8000/api/tasks").unwrap()}
                            />
                        },
                    }
                }</div>
            </div>
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetUser(id) => {
                console!(log, format!("User set: {}", id));
                self.set_user(id);
            },
            Msg::UnsetUser => {
                console!(log, format!("User unset"));
                self.unset_user();
            },
            Msg::SetToken(token) => {
                console!(log, format!("Token set: {}", token));
                self.set_token(token);
            },
            Msg::UnsetToken => {
                console!(log, format!("User unset"));
                self.unset_token();
            },
            //Msg::RouteChanged(route) => {
            //    console!(log, format!("RouteChanged({:#?})", route));
            //    self.route = route
            //},
            Msg::ChangeRoute(route) => {
                console!(log, format!("ChangeRoute({:#?})", route));
                self.route_service.set_route(&route.to_string(), ());
                self.route = self.route_service.get_route();
                console!(log, format!("Changed route to {:#?}", self.route));
            }
        }
        true
    }
}
