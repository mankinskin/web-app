pub use plans::{
    *,
    task::*,
};
pub use crate::tree::{
    *,
};
use yew::{
    *,
};
#[derive(Properties, Clone, Debug)]
pub struct TaskData {
    task: Task,
}
impl From<Task> for TaskData {
    fn from(task: Task) -> Self {
        Self {
            task,
        }
    }
}
//impl From<TaskData> for TreeData<TaskData> {
//    fn from(t: TaskData) -> Self {
//        Self {
//            element: t,
//            expanded: false,
//            message_parent: None,
//            children: Vec::new(),
//        }
//    }
//}
pub trait Preview : Component {
    fn preview(props: <Self as Component>::Properties) -> Html;
}

#[derive(Debug)]
pub struct TaskPreview {
    props: TaskData,
    link: ComponentLink<Self>,
}
impl Component for TaskPreview {
    type Message = ();
    type Properties = TaskData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        html! {
            <div class="task task-preview">
                <h1>{
                    self.props.task.title()
                }</h1>
            </div>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

#[derive(Debug)]
pub struct TaskView {
    props: TaskData,
    link: ComponentLink<Self>,
}
impl Component for TaskView {
    type Message = ();
    type Properties = TaskData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        html! {
            <div class="task task-content">
                <div>{
                    "Descripion:"
                }</div>
                <div>{
                    self.props.task.description()
                }</div>
            </div>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Preview for TaskView {
    fn preview(props: <Self as Component>::Properties) -> Html {
        html! {
            <TaskPreview with props/>
        }
    }
}

#[derive(Properties, Clone, Debug)]
pub struct TaskTreeData {
    pub task: TaskData,
    pub children: Vec<TaskTreeData>,
}
impl From<TaskTreeData> for TreeData<TaskData> {
    fn from(data: TaskTreeData) -> Self {
        Self {
            element: data.task,
            expanded: false,
            message_parent: None,
            children: data.children.iter().map(|c| Self::from(c.clone())).collect(),
        }
    }
}
#[derive(Debug)]
pub struct TaskTreeView {
    props: TaskTreeData,
    link: ComponentLink<Self>,
}
impl Component for TaskTreeView {
    type Message = ();
    type Properties = TaskTreeData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        let props = TreeData::from(self.props.clone());
        html! {
            <div class="task-tree">
                <TreeView<TaskView> with props/>
            </div>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Preview for TaskTreeView {
    fn preview(props: <Self as Component>::Properties) -> Html {
        let props = props.task;
        html! {
            <TaskPreview with props/>
        }
    }
}
