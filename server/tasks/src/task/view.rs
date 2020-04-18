use super::{
    TaskData,
    message::Msg,
    TaskPreview,
};
use common::{
    expander::{ExpanderView},
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

#[derive(Debug, Clone)]
pub struct TaskView {
    props: TaskData,
    link: ComponentLink<Self>,
}
impl Child for TaskView { }
impl Parent<ExpanderView<TaskView>> for TaskView {
    fn child_callback(link: &ComponentLink<Self>, child_index: usize)  -> Callback<<ExpanderView<TaskView> as Component>::Message>{
        link.callback(move |msg| {
            //console!(log, format!("child {} callback", child_index));
            <Msg as ChildMessage<ExpanderView<TaskView>>>::child_message(child_index, msg)
        })
    }
}
impl TaskView {
    pub fn move_to_left_edge(event: ClickEvent) -> <Self as Component>::Message {
        //console!(log, "Got clicked!");
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
            //console!(log, "applying {}", cmd.clone());
            body.set_attribute("style", &cmd).unwrap();
        }
        Msg::Noop
    }
    pub fn focus_task(&self) -> Callback<ClickEvent> {
        //console!(log, "Creating click callback");
        self.link.callback(Self::move_to_left_edge)
    }
    //pub fn edit_description(&self) -> Callback<ClickEvent> {
    //    //console!(log, "Creating click callback");
    //    self.link.callback(|click_event: ClickEvent| {
    //        //console!(log, "Got clicked!");
    //        if let Some(target) = click_event.target() {
    //            let mut target: HtmlElement = HtmlElement::try_from(target).unwrap();
    //            console!(log, format!("inner text: {}", target.inner_text()));
    //            target.set_attribute("contentEditable", "true").expect("Failed to set attribute");
    //        }
    //        Msg::Noop
    //    })
    //}
    pub fn update_description(&self) -> Callback<yew::events::InputData> {
        //console!(log, "Creating click callback");
        self.link.callback(|input: yew::events::InputData| {
            Msg::UpdateDescription(input.value)
        })
    }
}
impl Component for TaskView {
    type Message = Msg;
    type Properties = TaskData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskView"));
        //console!(log, format!("{} children", props.children.len()));
        //console!(log, format!("{} callback", props.message_parent.is_some()));
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("rendering TaskView for {:#?}", self.props));
        //console!(log, format!("Rendering TaskView"));
        let mut props = self.props.clone();
        props.set_callbacks(&self.link);
        html! {
            <div class="task task-container">
                <div class="task-content">
                    <h1 class="task-title">{
                        self.props.task.title()
                    }</h1>
                    <div class="task-description-container">
                        <div class="task-description-title">{
                            "Descripion"
                        }</div>
                        <div class="task-description-text"
                            contentEditable="true"
                            oninput={self.update_description()}>{
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
                </div>
                <div class="task-children">
                    { // item
                        for props.children
                            .iter()
                            .cloned()
                            .map(|props| {
                                html! {
                                    <div class="task-tree-level" tabindex="0" onclick={self.focus_task()}>
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
        //console!(log, format!("Changing TaskView"));
        self.props = props;
        false
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        //console!(log, format!("Updating TaskView"));
        if let Some(message_parent) = &self.props.message_parent {
            //console!(log, format!("child TaskView"));
            message_parent.emit(msg.clone());
        } else {
            //console!(log, format!("root TaskView"));
        }
        self.props.update(msg.clone());
        match msg {
            Msg::ExpanderMessage(_, _) => true,
            _ => false,
        }
    }
}
impl Preview for TaskView {
    fn preview(props: <Self as Component>::Properties) -> Html {
        html! {
            <TaskPreview with props/>
        }
    }
}
