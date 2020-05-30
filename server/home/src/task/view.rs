use super::{
    TaskViewProps,
    TaskTreeNodeProps,
    message::{
        *,
    },
    TaskPreview,
    TaskNodeView,
};
use components::{
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};
#[derive(Debug, Clone)]
pub struct TaskView {
    props: TaskViewProps,
    link: ComponentLink<Self>,
}
impl Component for TaskView {
    type Message = TaskMsg;
    type Properties = TaskViewProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskView"));

        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("Rendering TaskView"));
        let task = self.props.data.task.clone();
        html! {
            <div class="task task-container">
                <div class="task-content">
                    <h1 class="task-title">{
                        task.title()
                    }</h1>
                    <div class="task-description-container">
                        <div class="task-subsection-title">{
                            "Descripion"
                        }</div>
                        <div class="task-description-text"
                            contentEditable="true"
                            oninput={self.update_description()}
                        >
                        <div>{task.description()}</div>
                        </div>
                    </div>
                    <div class="task-assignees-container">
                        <div class="task-subsection-title">{
                            "Assignees"
                        }</div>
                        <div>{
                            for task
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
                        for self.props.data.children
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(i, child)| {
                                let props = TaskTreeNodeProps::with_callback(child, self.child_callback(i));
                                html!{
                                    <TaskNodeView with props />
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
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.parent_callback.emit(msg);
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
impl IParent<TaskNodeView> for TaskView {
    fn link(&self)  -> &ComponentLink<Self> {
        &self.link
    }
}
impl TaskView {
    pub fn update_description(&self) -> Callback<yew::events::InputData> {
        //console!(log, "Creating click callback");
        self.link.callback(|input: yew::events::InputData| {
            TaskMsg::UpdateDescription(input.value)
        })
    }
}
