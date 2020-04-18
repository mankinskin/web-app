pub use plans::{
    *,
    task::*,
};
pub use common::{
    expander::{self, *},
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};
use stdweb::{
    unstable::{
        TryFrom,
    },
    web::{
        event::{
            IEvent,
        },
        IElement,
        HtmlElement,
        IHtmlElement,
    }
};
#[derive(Clone, Debug)]
pub enum Msg {
    ExpanderMessage(usize, Box<expander::Msg<TaskView>>),
    Noop,
}
impl ChildMessage<ExpanderView<TaskView>> for Msg {
    fn child_message(child_index: usize, msg: <ExpanderView<TaskView> as Component>::Message) -> Self {
        Msg::ExpanderMessage(child_index, Box::new(msg))
    }
}

#[derive(Properties, Clone, Debug)]
pub struct TaskData {
    pub task: Task,
    pub message_parent: Option<Callback<<TaskView as Component>::Message>>,
    pub children: Vec<ExpanderData<TaskView>>,
}
impl TaskData {
    pub fn from_task(task: Task) -> Self {
        let children = task.children
            .iter()
            .cloned()
            .enumerate()
            .map(|(_, child)|
                ExpanderData::<TaskView> {
                    element: TaskData::from_task(child),
                    expanded: false,
                    message_parent: Callback::noop(),
                }
            )
            .collect();
        Self {
            task,
            message_parent: None,
            children,
        }
    }
    pub fn set_callbacks(&mut self, link: &ComponentLink<TaskView>) {
        for (child_index, child_expander) in self.children.iter_mut().enumerate() {
            child_expander.set_parent_callback(<TaskView as Parent<ExpanderView<TaskView>>>::child_callback(link, child_index));
        }
    }

}
impl ChildProps<TaskView> for TaskData {
    fn set_parent_callback(&mut self, callback: Callback<<TaskView as Component>::Message>) {
        self.message_parent = Some(callback);
    }
    fn get_parent_callback(&self)-> Callback<<TaskView as Component>::Message> {
        self.message_parent.clone().unwrap_or(Callback::noop())
    }
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::ExpanderMessage(child_index, msg) => {
                console!(log, format!("ExpanderMessage {} {:#?}", child_index, msg));
                self.children[child_index].update(*msg);
            },
            Msg::Noop => {},
        }
    }
}
impl Child for TaskView {
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

#[derive(Debug, Clone)]
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
impl Parent<ExpanderView<TaskView>> for TaskView {
    fn child_callback(link: &ComponentLink<Self>, child_index: usize)  -> Callback<<ExpanderView<TaskView> as Component>::Message>{
        link.callback(move |msg| {
            //console!(log, format!("child {} callback", child_index));
            <Msg as ChildMessage<ExpanderView<TaskView>>>::child_message(child_index, msg)
        })
    }
}
impl TaskView {
    pub fn move_to_left_edge(event: yew::events::ClickEvent) -> <Self as Component>::Message {
        console!(log, "Got clicked!");
        if let Some(target) = event.target() {
            let body: HtmlElement = stdweb::web::document()
                .body().expect("body not found");
            let mut target: HtmlElement = HtmlElement::try_from(target).unwrap();
            if !target.class_list().contains("task-tree-level") {
                let elem = target.closest(".task-tree-level").unwrap().unwrap();
                target = HtmlElement::try_from(elem).unwrap();
            }
            let target_rect = target.get_bounding_client_rect();
            let body_rect = body.get_bounding_client_rect();
            //console!(log, "target {:#?}\nbody {:#?}", target_rect.clone(), body_rect.clone());
            let offset_left = target_rect.get_x() - body_rect.get_x();
            let cmd = format!("margin-left: {}px", -offset_left);
            console!(log, "applying {}", cmd.clone());
            body.set_attribute("style", &cmd).unwrap();
        }
        Msg::Noop
    }
    pub fn on_click(&self) -> Callback<yew::events::ClickEvent> {
        //console!(log, "Creating click callback");
        self.link.callback(Self::move_to_left_edge)
    }
}
impl Component for TaskView {
    type Message = Msg;
    type Properties = TaskData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        console!(log, format!("creating TaskView"));
        console!(log, format!("{} children", props.children.len()));
        console!(log, format!("{} callback", props.message_parent.is_some()));
        //if props.message_parent.is_none() {
        //}
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("rendering TaskView for {:#?}", self.props));
        console!(log, format!("Rendering TaskView"));
        let mut props = self.props.clone();
        props.set_callbacks(&self.link);
        html! {
            <div class="task task-container">
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
                        for props.children
                            .iter()
                            .cloned()
                            .map(|props| {
                                html! {
                                    <div class="task-tree-level" tabindex="0" onclick={self.on_click()}>
                                        <ExpanderView<TaskView> with props />
                                    </div>
                                }
                            })
                    }
                </div>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        console!(log, format!("Changing TaskView"));
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        console!(log, format!("Updating TaskView"));
        if let Some(message_parent) = &self.props.message_parent {
            console!(log, format!("child TaskView"));
            message_parent.emit(msg);
            false
        } else {
            console!(log, format!("root TaskView"));
            self.props.update(msg.clone());
            true
        }
    }
}
