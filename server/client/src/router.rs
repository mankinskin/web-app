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
    root::*,
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


pub enum Msg {
    //RouteChanged(Route<()>),
    ChangeRoute(ClientRoute),
}

#[derive(Properties, Clone, Debug)]
pub struct ClientRouterProps {
    pub session_setter: Callback<UserSession>,
    pub session: Option<UserSession>,
}
pub struct ClientRouter {
    props: ClientRouterProps,
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}
impl ClientRouter {
    fn session_setter(&self) -> Callback<UserSession> {
        self.props.session_setter.clone()
    }
    fn get_session(&self) -> Option<UserSession> {
        self.props.session.clone()
    }
    fn change_route(&self, route: ClientRoute) -> Callback<ClickEvent> {
        self.link.callback(move |_| {
            Msg::ChangeRoute(route.clone())
        })
    }
}

impl Component for ClientRouter {
    type Message = Msg;
    type Properties = ClientRouterProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        console!(log, format!("ClientRouter::create"));
        let route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        //let callback = link.callback(|route| Msg::RouteChanged(route));
        //route_service.register_callback(callback);

        Self {
            route_service,
            route,
            link,
            props
        }
    }
    fn view(&self) -> Html {
        console!(log, format!("view: self.route {:#?}", self.route));
        html! {
            <div>
                <nav class="menu">
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::Index) >
                        {"Index"}
                    </button>
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::User) >
                        {"User"}
                    </button>
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::Note) >
                        {"Note"}
                    </button>
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::Login) >
                        {"Log In"}
                    </button>
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::Signup) >
                        {"Sign Up"}
                    </button>
                    <button
                        class="router-navigation-button"
                        onclick=&self.change_route(ClientRoute::Tasks) >
                        {"Tasks"}
                    </button>
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
                                session_setter={self.session_setter()}
                            />
                        },
                        Some(ClientRoute::User) => {
                            if let Some(session) = self.get_session() {
                                html!{
                                    <UserProfileView
                                        user_url={Url::parse("http://localhost:8000/api/users").unwrap()}
                                        session={session.clone()}
                                    />
                                }
                            } else {
                                html!{
                                    <div>{"Please log in"}</div>
                                }
                            }
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
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
