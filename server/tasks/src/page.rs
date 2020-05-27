use yew::{
    *,
};
use common::{
    status_stack::*,
    fetch::*,
    database::*,
};
use plans::{
    task::*,
};
use url::{
    *,
};
use crate::{
    task::*,
};

pub enum Msg {
    Task(FetchResponse<Task>),
    Tasks(FetchResponse<Vec<Entry<Task>>>),
}
impl From<FetchResponse<Task>> for Msg {
    fn from(msg: FetchResponse<Task>) -> Self {
        Msg::Task(msg)
    }
}
impl From<FetchResponse<Vec<Entry<Task>>>> for Msg {
    fn from(msg: FetchResponse<Vec<Entry<Task>>>) -> Self {
        Msg::Tasks(msg)
    }
}
#[derive(Properties, Clone, Debug)]
pub struct PageData {
    pub tasks: Url,
    pub task: Url,
}
pub struct PageView {
    props: PageData,
    link: ComponentLink<Self>,
    status: StatusStack<(), String>,
    task: Option<Task>,
    tasks: Option<Vec<Entry<Task>>>,
}

impl PageView {
}
impl Component for PageView {
    type Message = Msg;
    type Properties = PageData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let s = Self {
            link,
            props,
            status: StatusStack::new(),
            task: None,
            tasks: None,
        };
        Fetch::get(s.props.task.clone())
            .responder(s.link.callback(|response| {
                Msg::Task(response)
            }))
            .send()
            .expect("Fetch request failed");

        Fetch::get(s.props.tasks.clone())
            .responder(s.link.callback(|response| {
                Msg::Tasks(response)
            }))
            .send()
            .expect("Fetch request failed");
        s
    }
    fn view(&self) -> Html {
        html! {
            <div class="page">{
                match self.tasks.clone() {
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
            Msg::Task(res) => {
                match res.into_inner() {
                    Ok(task) => self.task = Some(task),
                    Err(e) => console!(log, format!("{:#?}", e)),
                }
            },
            Msg::Tasks(res) => {
                match res.into_inner() {
                    Ok(tasks) => self.tasks = Some(tasks),
                    Err(e) => console!(log, format!("{:#?}", e)),
                }
            },
        }
        true
    }
}
