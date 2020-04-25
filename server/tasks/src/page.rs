use yew::{
    *,
    Callback,
    services::{
        fetch::{
            *,
            FetchTask,
        },
    },
    format::{
        Nothing,
        Json,
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
};
use anyhow::{
    Error
};
use stdweb::web::{
    *,
    html_element::{TextAreaElement},
};
use stdweb::unstable::TryInto;
pub enum Msg {
    GetTask,
    GetTaskStatus(Result<(), String>),
    GetTaskResponse(Result<Task, String>),

    PostTask(usize),
    PostTaskStatus(Result<(), String>),
    PostTaskResponse(Result<(), String>),

    GetTasks,
    GetTasksResponse(Result<Vec<Task>, String>),
    GetTasksStatus(Result<(), String>),
}
#[derive(Properties, Clone, Debug)]
pub struct PageData {
    pub tasks: Option<Vec<Task>>,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
    status: StatusStack<(), String>,
}
impl PageView {
    fn get_tasks(&mut self) -> Result<(), String> {
        let req = Request::get("/api/tasks")
            .header("Content-Type", "application/json")
            .body(Nothing)
            .unwrap();
        let callback = self.link.callback(|response: Response<Json<Result<Vec<Task>, Error>>>| {
            let (meta, Json(result)) = response.into_parts();
            if meta.status.is_success() {
                Msg::GetTasksResponse(result.map_err(|e| e.to_string()))
            } else {
                Msg::GetTaskResponse(Err(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status)))
                )
            }
        });
        let fetch_task = self.fetch_service.fetch(req, callback);
        match fetch_task {
            Ok(fetch_task) => {
                self.fetch_task = Some(fetch_task);
                Ok(())
            },
            Err(err) => {
                Err(err.to_string())
            },
        }
    }
    fn get_task(&mut self) -> Result<(), String> {
        let req = Request::get("/api/task")
            .header("Content-Type", "application/json")
            .body(Nothing)
            .unwrap();
        let callback = self.link.callback(|response: Response<Json<Result<Task, Error>>>| {
            let (meta, Json(result)) = response.into_parts();
            if meta.status.is_success() {
                Msg::GetTaskResponse(result.map_err(|e| e.to_string()))
            } else {
                Msg::GetTaskResponse(Err(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status)))
                )
            }
        });
        let fetch_task = self.fetch_service.fetch(req, callback);
        match fetch_task {
            Ok(fetch_task) => {
                self.fetch_task = Some(fetch_task);
                Ok(())
            },
            Err(err) => {
                Err(err.to_string())
            },
        }
    }
    fn post_task_callback(&self, index: usize) -> Callback<ClickEvent> {
        self.link.callback(move |_| {
            Msg::PostTask(index)
        })
    }
    fn post_task(&mut self, task: &Task) -> Result<(), String> {
        let json = serde_json::to_string(task).unwrap();
        let req = Request::post("/api/task")
            .header("Content-Type", "application/json")
            .body(Ok(json))
            .unwrap();
        let callback = self.link.callback(|response: Response<Nothing>| {
            let (meta, Nothing) = response.into_parts();
            if meta.status.is_success() {
                Msg::PostTaskStatus(Ok(()))
            } else {
                Msg::PostTaskStatus(Err(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status)))
                )
            }
        });
        let ft = self.fetch_service.fetch(req, callback);
        match ft {
            Ok(t) => {
                self.fetch_task = Some(t);
                Ok(())
            },
            Err(err) => {
                Err(err.to_string())
            },
        }
    }
}
struct RemoteData<T>
    where T: serde::Serialize
{
    buffer: Option<T>,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
    status: StatusStack<(), String>,
}
impl<T> RemoteData<T>
    where T: serde::Serialize
{
    fn post(&mut self, data: &T) -> Result<(), String> {
        let json = serde_json::to_string(data).unwrap();
        let req = Request::post("/api/task")
            .header("Content-Type", "application/json")
            .body(Ok(json))
            .unwrap();

        let callback = Callback::noop().reform(|response: Response<Nothing>| {
            let (meta, Nothing) = response.into_parts();
            if meta.status.is_success() {
                Msg::PostTaskStatus(Ok(()))
            } else {
                Msg::PostTaskStatus(Err(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status)))
                )
            }
        });

        let ft = self.fetch_service.fetch(req, callback);
        match ft {
            Ok(t) => {
                self.fetch_task = Some(t);
                Ok(())
            },
            Err(err) => {
                Err(err.to_string())
            },
        }
    }
}
impl Component for PageView {
    type Message = Msg;
    type Properties = PageData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        if props.tasks.is_none() {
            link.send_message(Msg::GetTasks);
        }
        Self {
            link,
            props,
            fetch_service: FetchService::new(),
            fetch_task: None,
            status: StatusStack::new(),
        }
    }
    fn view(&self) -> Html {
        html! {
            <div class="page">{
                match self.props.tasks.clone() {
                    Some(tasks) => {
                        html! {
                            {
                                for tasks.iter().enumerate().map(|(i, task)| {
                                    let props = TaskTreeRootProps::create_root(task.clone());
                                    html! {
                                        <div>
                                            <TaskRootView with props />
                                            <button onclick={self.post_task_callback(i)}>{
                                                "Push all changes"
                                            }</button>
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
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetTask => {
                self.status.clear();
                let status = self.get_task();
                self.link.send_message(Msg::GetTaskStatus(status));
                true
            },
            Msg::GetTasks => {
                self.status.clear();
                let status = self.get_tasks();
                self.link.send_message(Msg::GetTasksStatus(status));
                true
            },
            Msg::GetTasksStatus(status) => {
                match status {
                    Ok(_) => {
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
            Msg::GetTasksResponse(status) => {
                match status {
                    Ok(tasks) => {
                        self.props.tasks = Some(tasks);
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
            Msg::GetTaskStatus(status) => {
                match status {
                    Ok(_) => {
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
            Msg::GetTaskResponse(status) => {
                match status {
                    Ok(task) => {
                        self.props.tasks = self.props.tasks.clone()
                            .map(|mut tasks| {tasks.push(task.clone()); tasks })
                            .or_else(|| Some(vec![task.clone()]));
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
            Msg::PostTask(i) => {
                let status =
                    match self.props.tasks.clone() {
                        Some(tasks) => self.post_task(&tasks[i]),
                        None => Err("No tasks to post!".into())
                    };
                self.link.send_message(Msg::PostTaskStatus(status));
                true
            }
            Msg::PostTaskStatus(status) => {
                match status {
                    Ok(_) => {
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
            Msg::PostTaskResponse(status) => {
                match status {
                    Ok(_) => {
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
        }
    }
}
