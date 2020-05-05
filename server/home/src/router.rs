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
};
use common::{
    remote_data::*,
};

#[derive(Switch, Clone, Debug)]
pub enum ClientRoute {
    #[to = "/note"]
    Note,
    #[to = "/user"]
    User,
    #[to = "/"]
    Index,
}

impl ToString for ClientRoute {
    fn to_string(&self) -> String {
        match self {
            ClientRoute::Index => format!("/"),
            ClientRoute::User => format!("/user"),
            ClientRoute::Note => format!("/note"),
        }
    }
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(ClientRoute),
}

pub struct ClientRouter {
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}
impl ClientRouter {
    fn change_route(&self, route: ClientRoute) -> Callback<ClickEvent> {
        self.link.callback(move |_| {
            Msg::ChangeRoute(route.clone())
        })
    }
}

impl Component for ClientRouter {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        console!(log, format!("ClientRouter::create"));
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let callback = link.callback(|route| Msg::RouteChanged(route));
        //route_service.register_callback(callback);

        Self {
            route_service,
            route,
            link,
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
                </nav>
                <div>{
                    match ClientRoute::switch(self.route.clone()) {
                        Some(ClientRoute::Index) => html!{
                            <div>
                                <p>{"Index"}</p>
                                <a href="/tasks">{"Tasks"}</a>
                            </div>
                        },
                        Some(ClientRoute::User) => {
                            html!{ <UserProfileView user={RemoteData::try_new("http://0.0.0.0:8000/api/user").unwrap()}/> }
                        },
                        Some(ClientRoute::Note) => {
                            html!{ <NoteEditor note={RemoteData::try_new("http://0.0.0.0:8000/api/note").unwrap()}/> }
                        },
                        None => html!{ <p>{"404"}</p> },
                    }
                }</div>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => {
                console!(log, format!("RouteChanged({:#?})", route));
                self.route = route
            },
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
