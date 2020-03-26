use yew::{
    *,
    virtual_dom::*,
};
use yew_router::{
    *,
    route::Route,
    service::RouteService,
    Switch,
};

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/"]
    Root,
    #[to = "/budget"]
    Budget,
}

pub struct Router {
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}
pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
}

impl Router {
    fn change_route(&self, route: AppRoute) -> Callback<ClickEvent> {
        self.link.callback(move |_| {
            Msg::ChangeRoute(route.clone())
        })
    }
}

impl Component for Router {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let callback = link.callback(Msg::RouteChanged);
        route_service.register_callback(callback);

        Self {
            route_service,
            route,
            link,
        }
    }
    fn view(&self) -> Html {
        html! {
            <div>
                <nav class="menu",>
                    //<button onclick=&self.change_route(AppRoute::Root)> {"Root"} </button>
                    <button onclick=&self.change_route(AppRoute::Budget)> {"Budget"} </button>
                </nav>
                <div>
                {
                    match AppRoute::switch(self.route.clone()) {
                        Some(AppRoute::Root) => html!{ <p>{"Root"}</p> },
                        Some(AppRoute::Budget) => html!{ <p>{"Budget"}</p> },
                        None => VNode::from("404")
                    }
                }
                </div>
            </div>
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
            Msg::ChangeRoute(route) => {
                // This might be derived in the future
                let route_string = match route {
                    AppRoute::Root => format!("/"),
                    AppRoute::Budget => format!("/budget"),
                };
                self.route_service.set_route(&route_string, ());
                self.route = Route {
                    route: route_string,
                    state: (),
                };
            }
        }
        true
    }
}
