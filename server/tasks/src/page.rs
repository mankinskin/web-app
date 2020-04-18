use yew::{
    *,
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
pub enum Msg {
    GetTask,
    GetTaskStatus(Result<(), String>),
    GetTaskResponse(Result<Task, String>),
}
#[derive(Properties, Clone, Debug)]
pub struct PageData {
    pub task: Option<Task>,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
    status: StatusStack<(), String>,
}
impl PageView {
    //fn post_note_callback(&self) -> Callback<ClickEvent> {
    //    self.link.callback(|_: ClickEvent| {
    //        Msg::PostNote
    //    })
    //}
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
}
impl Component for PageView {
    type Message = Msg;
    type Properties = PageData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        if props.task.is_none() {
            link.send_message(Msg::GetTask);
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
                match self.props.task.clone() {
                    Some(task) => {
                        let props = TaskData::from_task(task);
                        html! {
                            <TaskView with props />
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
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetTask => {
                self.status.clear();
                let status = self.get_task();
                self.link.send_message(Msg::GetTaskStatus(status));
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
                        self.props.task = Some(task);
                        self.status.push(Ok(()))
                    },
                    Err(err) => self.status.push(Err(err)),
                }
                true
            },
        }
    }
}
