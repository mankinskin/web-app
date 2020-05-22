use yew::{
    *,
};
use common::{
    status_stack::*,
    remote_data::*,
    database::*,
};
use plans::{
    task::*,
};
use crate::{
    task::*,
};
use futures::{Future, FutureExt};
use std::result::{Result};

pub enum Msg {
    RemoteTask(RemoteMsg<Task>),
    RemoteTasks(RemoteMsg<Vec<Entry<Task>>>),
}
impl From<RemoteMsg<Task>> for Msg {
    fn from(msg: RemoteMsg<Task>) -> Self {
        Msg::RemoteTask(msg)
    }
}
impl From<RemoteMsg<Vec<Entry<Task>>>> for Msg {
    fn from(msg: RemoteMsg<Vec<Entry<Task>>>) -> Self {
        Msg::RemoteTasks(msg)
    }
}
#[derive(Properties, Clone, Debug)]
pub struct PageData {
    pub tasks: RemoteRoute,
    pub task: RemoteRoute,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    status: StatusStack<(), String>,
    tasks: RemoteData<Vec<Entry<Task>>, Self>,
    task: RemoteData<Task, Self>,
}

impl PageView {
}
impl Component for PageView {
    type Message = Msg;
    type Properties = PageData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let task = RemoteData::new(props.task.clone(), link.clone());
        let tasks = RemoteData::new(props.tasks.clone(), link.clone());
        let s = Self {
            link,
            props,
            status: StatusStack::new(),
            task,
            tasks,
        };
        s.link.send_message(Msg::RemoteTasks(RemoteMsg::Request(FetchMethod::Get)));
        //s.link.send_message(Msg::RemoteTask(RemoteMsg::Request(FetchMethod::Get)));
        s
    }
    fn view(&self) -> Html {
        html! {
            <div class="page">{
                match self.tasks.clone().as_deref() {
                    Some(tasks) => {
                        html! {
                            {
                                for tasks.iter().enumerate().map(|(i, task)| {
                                    let props = TaskTreeRootProps::create_root(task.data().clone());
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
                            self.task.fetch_request(request)
                                .expect("Failed to make request")
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.task.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
            Msg::RemoteTasks(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            self.tasks.fetch_request(request)
                                .expect("Failed to make request")
                        );
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.tasks.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
        }
        true
    }
}
