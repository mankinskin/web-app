use yew::{
    *,
    services::{
        fetch::{
            FetchService,
            FetchTask,
        },
    },
};
use common::{
    status_stack::*,
};
use plans::{
    task::*,
};
use crate::{
    task::*,
    remote_data::*,
};
use rql::{
    *
};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsCast;
use std::result::{Result};
use web_sys::{Request, Response};
use wasm_bindgen_futures::JsFuture;
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
    pub task_list: RemoteData<Vec<Task>>,
    pub task: RemoteData<Task>,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    status: StatusStack<(), String>,
}
async fn fetch_request<T>(request: Request, responder: Callback<RemoteResponse<T>>)
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug
{
    let window = web_sys::window().unwrap();
    JsFuture::from(window.fetch_with_request(&request))
        .then(|result| {
            let value = result.unwrap();
            assert!(value.is_instance_of::<Response>());
            let response: Response = value.dyn_into().unwrap();
            console!(log, format!("got response {:#?}", response));
            futures::future::ready(response)
            //match result {
            //    Ok(value) => {
            //        assert!(value.is_instance_of::<Response>());
            //        Ok(value.dyn_into().unwrap() as Response)
            //    },
            //    Err(e) => {Err(e)},
            //})
        })
        .then(move |response: Response| {
            let promise = response
                .json()
                .map_err(|e| anyhow!(format!("{:#?}", e)))
                .unwrap();
                    //.and_then(|promise| {
                    //console!(log, format!("Start blocking"));
            JsFuture::from(promise)
                    //console!(log, format!("Response Json {:#?}", res));
                    //console!(log, format!("Response body {:#?}", res));
                //})
        })
        .then(move |res: Result<JsValue, JsValue>| {
            futures::future::ready(res.unwrap())
        })
        .then(move |val: JsValue| {
            RemoteResponse::for_request(request, val)
        })
        .then(move |resp: RemoteResponse<T>| {
            //data.respond(request, resp)
            responder.emit(resp);
            futures::future::ready(())
        }).await
}
impl PageView {
    fn tasks_request(&mut self, req: RemoteRequest<Vec<Task>>) -> Request {
        console!(log, "task_list request");
        self.props.task_list.request(req.clone()).unwrap()
        //console!(log, format!("{:#?}", request));
    }
    fn tasks_responder(&mut self) -> Callback<RemoteResponse<Vec<Task>>> {
        self.link.callback(move |response: RemoteResponse<Vec<Task>>| {
            Msg::RemoteTasks(RemoteMsg::Response(response))
        })
    }
    fn task_request(&mut self, req: RemoteRequest<Task>) -> Request {
        console!(log, "task_list request");
        self.props.task.request(req.clone()).unwrap()
        //console!(log, format!("{:#?}", request));
    }
    fn task_responder(&mut self) -> Callback<RemoteResponse<Task>> {
        self.link.callback(move |response: RemoteResponse<Task>| {
            Msg::RemoteTask(RemoteMsg::Response(response))
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
                match self.props.task_list.clone().as_deref() {
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
                        //self.task_request(request);
                        wasm_bindgen_futures::spawn_local(
                            fetch_request(
                                self.task_request(request),
                                self.task_responder()
                            )
                        );
                    },
                    RemoteMsg::Response(response) => {
                        console!(log, format!("Got RemoteResponse {:#?}", response));
                        //self.props.task.update(response);
                    },
                }
            },
            Msg::RemoteTasks(msg) => {
                match msg {
                    RemoteMsg::Request(request) => {
                        wasm_bindgen_futures::spawn_local(
                            fetch_request(
                                self.tasks_request(request),
                                self.tasks_responder()
                            )
                        );
                    },
                    RemoteMsg::Response(response) => {
                        console!(log, format!("Got RemoteResponse {:#?}", response));
                        //self.props.task_list.update(response);
                    },
                }
            },
        }
        true
    }
}
