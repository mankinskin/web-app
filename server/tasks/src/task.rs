pub use plans::{
    *,
    task::*,
};
pub use common::{
    expander::*,
    preview::*,
};
use yew::{
    *,
};
#[derive(Properties, Clone, Debug)]
pub struct TaskData {
    pub task: Task,
}
impl From<Task> for TaskData {
    fn from(task: Task) -> Self {
        Self {
            task,
        }
    }
}
impl From<TaskData> for ExpanderData<TaskData> {
    fn from(data: TaskData) -> Self {
        Self {
            element: data,
            expanded: false,
            //message_parent: None,
        }
    }
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
                <h1 class="task-title">{
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
impl Preview for TaskView {
    fn preview(props: <Self as Component>::Properties) -> Html {
        html! {
            <TaskPreview with props/>
        }
    }
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
            <div class="task">
                <h1 class="task-title">{
                    self.props.task.title()
                }</h1>
                <div class="task-description-container">
                    <div>{
                        "Descripion:"
                    }</div>
                    <div>{
                        self.props.task.description()
                    }</div>
                </div>
                <div class="task-assignees-container">
                    <div>{
                        "Assignees:"
                    }</div>
                    <div>{
                        for self.props.task
                            .assignees()
                            .iter()
                            .map(|assignee| html!{
                                <div>{
                                    assignee
                                }</div>
                            })
                    }</div>
                </div>
                <div class="task-children">
                    { // item
                        for self.props.task.children
                        .iter()
                        .cloned()
                        .map(|task| {
                            let props = ExpanderData::from(TaskData::from(task));
                            html! {
                                <ExpanderView<Self> with props/>
                            }
                        })
                    }
                </div>
            </div>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}

