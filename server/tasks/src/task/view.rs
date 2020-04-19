use super::{
    TaskData,
    TaskNodeData,
    message::{
        *,
        Msg,
    },
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
pub struct TaskNodeView {
    props: TaskNodeData,
    link: ComponentLink<Self>,
}
impl Child for TaskNodeView { }
impl Parent<ExpanderView<TaskNodeView>> for TaskNodeView {
    fn child_callback(&self, child_index: usize)  -> Callback<<ExpanderView<TaskNodeView> as Component>::Message>{
        self.link.callback(move |msg| {
            //console!(log, format!("child {} callback", child_index));
            <Msg as ChildMessage<ExpanderView<TaskNodeView>>>::child_message(child_index, msg)
        })
    }
    fn set_child_callbacks(&mut self) {
        self.props.children =
            self.props
                .children
                .iter()
                .cloned()
                .enumerate()
                .map(|(child_index, mut child_expander)| {
                    child_expander.set_parent_callback(self.child_callback(child_index));
                    child_expander
                })
                .collect()
    }
}



trait Focussable<E> {
    fn focus(&self) -> Callback<E>;
}
impl Focussable<ClickEvent> for TaskNodeView {
    fn focus(&self) -> Callback<ClickEvent> {
        //console!(log, "Creating click callback");
        self.link.callback(|event: ClickEvent| {
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
            Msg::Focussed
        })
    }
}
impl TaskNodeView {
    pub fn update_description(&self) -> Callback<yew::events::InputData> {
        //console!(log, "Creating click callback");
        self.link.callback(|input: yew::events::InputData| {
            Msg::TaskMessage(TaskMsg::UpdateDescription(input.value))
        })
    }
}
impl Component for TaskNodeView {
    type Message = Msg;
    type Properties = TaskNodeData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskNodeView"));
        //console!(log, format!("{} children", props.children.len()));
        //console!(log, format!("{} callback", props.parent_callback.is_some()));
        let mut s = Self {
            props,
            link,
        };
        s.set_child_callbacks();
        s
    }
    fn mounted(&mut self) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        //console!(log, format!("rendering TaskNodeView for {:#?}", self.props));
        //console!(log, format!("Rendering TaskNodeView"));
        let props = self.props.clone();
        html! {
            <div class="task task-container">
                <div class="task-content">
                    <h1 class="task-title">{
                        self.props.task.title()
                    }</h1>
                    <div class="task-description-container">
                        <div class="task-subsection-title">{
                            "Descripion"
                        }</div>
                        <div class="task-description-text"
                            contentEditable="true"
                            oninput={self.update_description()}
                        >
                        <div>{self.props.task.description()}</div>
                        </div>
                    </div>
                    <div class="task-assignees-container">
                        <div class="task-subsection-title">{
                            "Assignees"
                        }</div>
                        <div>{
                            for self.props.task
                                .assignees()
                                .iter()
                                .map(|assignee| html!{
                                    <div>{assignee.name()}</div>
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
                                    <div class="task-tree-level" tabindex="0" onclick={self.focus()}>
                                        <ExpanderView<TaskNodeView> with props />
                                    </div>
                                }
                            })
                    }
                </div>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing TaskNodeView"));
        self.props = props;
        self.set_child_callbacks();
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        console!(log, format!("Updating TaskNodeView"));
        console!(log, format!("{:#?}", msg));
        //self.props.update(msg.clone());
        self.props.parent_callback.emit(msg.clone());
        false
    }
}
impl Preview for TaskNodeView {
    fn preview(props: <Self as Component>::Properties) -> Html {
        html! {
            <TaskPreview with props/>
        }
    }
}
#[derive(Debug, Clone)]
pub struct TaskView {
    props: TaskData,
    child: TaskNodeData,
    link: ComponentLink<Self>,
}
impl Focussable<ClickEvent> for TaskView {
    fn focus(&self) -> Callback<ClickEvent> {
        //console!(log, "Creating click callback");
        self.link.callback(|event: ClickEvent| {
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
            Msg::Focussed
        })
    }
}
impl Parent<TaskNodeView> for TaskView {
    fn child_callback(&self, _child_index: usize)  -> Callback<<TaskNodeView as Component>::Message>{
        self.link.callback(move |msg| {
            //console!(log, format!("child {} callback", child_index));
            <Msg as ChildMessage<TaskNodeView>>::child_message(0, msg)
        })
    }
    fn set_child_callbacks(&mut self) {
        self.child.set_parent_callback(self.child_callback(0));
    }
}

impl Component for TaskView {
    type Message = Msg;
    type Properties = TaskData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskNodeView"));
        //console!(log, format!("{} children", props.children.len()));
        //console!(log, format!("{} callback", props.parent_callback.is_some()));

        let child = TaskNodeData::create(props.task.clone(), Callback::noop());
        let mut s = Self {
            props,
            child,
            link,
        };
        s.set_child_callbacks();
        s
    }
    fn mounted(&mut self) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        //console!(log, format!("rendering TaskNodeView for {:#?}", self.props));
        //console!(log, format!("Rendering TaskNodeView"));
        let props = self.child.clone();
        html! {
            <div class="task-tree-level" tabindex="0" onclick={self.focus()}>
                <TaskNodeView with props />
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing TaskNodeView"));
        self.child = TaskNodeData::create(props.task.clone(), Callback::noop());
        self.set_child_callbacks();
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        console!(log, format!("Updating TaskNodeView"));
        console!(log, format!("{:#?}", msg));
        self.props.task.update(msg.clone());
        if let Msg::ChildMessage(_, m) = msg.clone() {
            self.child.update(*m);
        }
        true
    }
}
