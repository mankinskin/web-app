use yew::{
    *,
};
use common::{
    status_stack::*,
    remote_data::*,
};
use plans::{
    task::*,
};
use crate::{
    task::*,
};
use rql::{
    *
};
use futures::{Future, FutureExt};

pub enum Msg {
    RemoteTask(RemoteMsg<Task>),
    RemoteTasks(RemoteMsg<Vec<Task>>),
}
impl From<RemoteMsg<Task>> for Msg {
    fn from(msg: RemoteMsg<Task>) -> Self {
        Msg::RemoteTask(msg)
    }
}
impl From<RemoteMsg<Vec<Task>>> for Msg {
    fn from(msg: RemoteMsg<Vec<Task>>) -> Self {
        Msg::RemoteTasks(msg)
    }
}
#[derive(Properties, Clone, Debug)]
pub struct PageData {
    pub tasks: RemoteData<Vec<Task>>,
    pub task: RemoteData<Task>,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    status: StatusStack<(), String>,
}

impl PageView {
    fn tasks_responder(&self) -> Callback<RemoteResponse<Vec<Task>>> {
        self.link.callback(move |response: RemoteResponse<Vec<Task>>| {
            Msg::RemoteTasks(RemoteMsg::Response(response))
        })
    }
    fn tasks_request(&self, request: RemoteRequest<Vec<Task>>) -> impl Future<Output=()> + 'static {
        let callback = self.tasks_responder().clone();
        self.props.tasks.fetch_request(request)
            .then(move |res: RemoteResponse<Vec<Task>>| {
                futures::future::ready(callback.emit(res))
            })
    }
    fn task_responder(&self) -> Callback<RemoteResponse<Task>> {
        self.link.callback(move |response: RemoteResponse<Task>| {
            Msg::RemoteTask(RemoteMsg::Response(response))
        })
    }
    fn task_request(&self, request: RemoteRequest<Task>) -> impl Future<Output=()> + 'static {
        let callback = self.task_responder().clone();
        self.props.task.fetch_request(request)
            .then(move |res: RemoteResponse<Task>| {
                futures::future::ready(callback.emit(res))
            })
    }
}
impl Component for PageView {
    type Message = Msg;
    type Properties = PageData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let s = Self {
            link,
            props,
            status: StatusStack::new(),
        };
        s.link.send_message(Msg::RemoteTasks(RemoteMsg::Request(RemoteRequest::Get(Id::new()))));
        //s.link.send_message(Msg::RemoteTask(RemoteMsg::Request(RemoteRequest::Get(Id::new()))));
        s
    }
    fn view(&self) -> Html {
        html! {
            <div class="page">{
                match self.props.tasks.clone().as_deref() {
                    Some(tasks) => {
                        html! {
                            {
                                for tasks.iter().enumerate().map(|(i, task)| {
                                    let props = TaskTreeRootProps::create_root(task.clone());
                                    html! {
                                        <div>
                                            <TaskRootView with props />
                                            //<button onclick={self.post_task_callback(i)}>{
                                            //    "Push all changes"
                                            //}</button>
                                        </div>
                                    }
                                })
                            }
                        }
                    },
                    None => {
                        let props = StatusStackData::from(self.status.clone());
                        html! {
                            <div style="display: contents; color: white;">
                                <StatusStackView<(), String> with props/>
                            </div>
                        }
                    }
                }
            }</div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RemoteTask(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            self.task_request(request)
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.props.task.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
            Msg::RemoteTasks(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            self.tasks_request(request)
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.props.tasks.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
        }
        true
    }
}
