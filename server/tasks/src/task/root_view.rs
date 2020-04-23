use super::{
    TaskViewProps,
    TaskTreeRootProps,
    message::{
        *,
    },
    TaskView,
};
use common::{
    parent_child::*,
};
use yew::{
    *,
};

#[derive(Debug, Clone)]
pub struct TaskRootView {
    props: TaskTreeRootProps,
    link: ComponentLink<Self>,
}
impl Component for TaskRootView {
    type Message = RootMsg;
    type Properties = TaskTreeRootProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskRootView"));
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("Rendering TaskRootView"));
        let props = TaskViewProps::with_callback(self.props.data.clone(), self.child_callback(0));
        html! {
            <div class="task-tree-level" tabindex="0" onfocus={self.focus()}>
                <TaskView with props />
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing TaskRootView"));
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.update(msg);
        true
    }
}
impl Focussable<FocusEvent> for TaskRootView {
    fn focus(&self) -> Callback<FocusEvent> {
        //console!(log, "Creating focus callback");
        self.link.callback(|event: FocusEvent| {
            //console!(log, "Got focussed!");
            RootMsg::Focussed(event)
        })
    }
}
impl IParent<TaskView> for TaskRootView {
    fn link(&self)  -> &ComponentLink<Self> {
        &self.link
    }
}
