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
    events::{
        FocusEvent,
    },
};
use stdweb::{
    unstable::{
        TryFrom,
    },
    web::{
        event::{
            IEvent,
        },
        EventTarget,
        IEventTarget,
        Element,
        IElement,
        HtmlElement,
        IHtmlElement,
    }
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
impl TaskView {
    pub fn move_to_left_edge(event: yew::events::ClickEvent) -> <Self as Component>::Message {
        console!(log, "Got clicked!");
        if let Some(target) = event.target() {
            let body: HtmlElement = stdweb::web::document()
                .body().expect("body not found");
            let mut target: HtmlElement = HtmlElement::try_from(target).unwrap();
            if !target.class_list().contains("task-container") {
                let elem = target.closest(".task-container").unwrap().unwrap();
                target = HtmlElement::try_from(elem).unwrap();
            }
            let target_rect = target.get_bounding_client_rect();
            let body_rect = body.get_bounding_client_rect();
            let offset_left = target_rect.get_x() - body_rect.get_x();
            let cmd = format!("margin-left: {}px", -offset_left.max(0.0));
            console!(log, "applying {}", cmd.clone());
            body.set_attribute("style", &cmd).unwrap();
        }
        ()
    }
    pub fn on_click(&self) -> Callback<yew::events::ClickEvent> {
        console!(log, "Creating click callback");
        self.link.callback(Self::move_to_left_edge)
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
            <div class="task task-container" tabindex="0" onclick={self.on_click()}>
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

