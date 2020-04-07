use yew::{
    *,
};
use yew_router::{
    *,
    route::Route,
    service::RouteService,
    Switch,
};
use crate::{
    tree::*,
    task::*,
};

#[derive(Switch, Clone, Debug)]
pub enum ClientRoute {
    #[to = "/tasks/tools"]
    Tools,
    #[to = "/tasks"]
    Index,
}

impl ToString for ClientRoute {
    fn to_string(&self) -> String {
        match self {
            ClientRoute::Index => format!("/tasks"),
            ClientRoute::Tools => format!("/tasks/tools"),
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
        route_service.register_callback(callback);

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
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Index)>{
                        "Index"
                    }</button>
                    <button class="router-navigation-button" onclick=&self.change_route(ClientRoute::Tools)>{
                        "Tools"
                    }</button>
                </nav>
                <div>{
                    match ClientRoute::switch(self.route.clone()) {
                        Some(ClientRoute::Index) => html! {
                            <div>
                                <TaskTreeView
                                task={TaskData::from(Task {
                                    title: "Root".into(),
                                    description: "This is the top level task.".into(),
                                    assignees: vec!["Heinz".into(), "Kunigunde".into(), "Andreas".into()],
                                })}
                                children={vec![
                                    TaskTreeData {
                                        task: Task {
                                            title: "First Item".into(),
                                            description: "This is the first sub task.".into(),
                                            assignees: vec!["Heinz".into(), "Kunigunde".into()],
                                        }.into(),
                                        children: vec![
                                            TaskTreeData {
                                                task: Task {
                                                    title: "Second Level".into(),
                                                    description: "This is a sub task of a sub task.".into(),
                                                    assignees: vec!["Heinz".into(), "Kunigunde".into()],
                                                }.into(),
                                                children: Vec::new(),
                                            }
                                        ],
                                    },
                                    TaskTreeData {
                                        task: Task {
                                            title: "Another Sub Task".into(),
                                            description: "This sub task has many children.".into(),
                                            assignees: vec!["Günter".into(), "Siegbert".into(), "Manfred".into(), "Georg".into()],
                                        }.into(),
                                        children: vec![
                                            TaskTreeData {
                                                task: Task {
                                                    title: "Task 1.2.1".into(),
                                                    description: "Child 1.".into(),
                                                    assignees: vec!["Günter".into()],
                                                }.into(),
                                                children: Vec::new(),
                                            },
                                            TaskTreeData {
                                                task: Task {
                                                    title: "Task 1.2.2".into(),
                                                    description: "Child 2.".into(),
                                                    assignees: vec!["Siegbert".into()],
                                                }.into(),
                                                children: Vec::new(),
                                            },
                                            TaskTreeData {
                                                task: Task {
                                                    title: "Task 1.2.3".into(),
                                                    description: "Child 3.".into(),
                                                    assignees: vec!["Manfred".into(), "Georg".into()],
                                                }.into(),
                                                children: Vec::new(),
                                            }

                                        ],
                                    }
                                ]}
                                />
                                <a href="/">{"Home"}</a>
                            </div>
                        },
                        Some(ClientRoute::Tools) => html! {
                            <p>{
                                "Tools"
                            }</p>
                        },
                        None => html! {
                            <p>{"404"}</p> },
                    }
                }</div>
            </div>
        }
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