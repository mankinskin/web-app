use super::{
    TaskTreeNodeProps,
    message::{
        *,
    },
    TaskView,
    TaskViewProps,
};
use components::{
    expander::{ExpanderView},
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};

#[derive(Debug, Clone)]
pub struct TaskNodeView {
    props: TaskTreeNodeProps,
    link: ComponentLink<Self>,
}
impl Component for TaskNodeView {
    type Message = NodeMsg;
    type Properties = TaskTreeNodeProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating TaskNodeView"));
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("Rendering TaskNodeView"));
        html! {
            <div class="task-tree-level" tabindex="0" onfocus={self.focus()}>
                <ExpanderView
                    data={self.props.data.expander_data.clone()}
                    parent_callback={<Self as IParent<ExpanderView>>::child_callback(self, 0)}
                    >{{
                    //console!(log, format!("Adding expander Node {:#?}", props));
                    let props = TaskViewProps::with_callback(
                        self.props.data.tree_data.clone(),
                        <Self as IParent<TaskView>>::child_callback(self, 0)
                        );
                    if self.props.data.expander_data.is_expanded() {
                        html!{
                            <TaskView with props/>
                        }
                    } else {
                        TaskView::preview(props)
                    }
                }}</ExpanderView>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing TaskNodeView"));
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.node_callback.emit(msg);
        false
    }
}
impl IParent<ExpanderView> for TaskNodeView {
    fn link(&self)  -> &ComponentLink<Self> {
        &self.link
    }
}
impl IParent<TaskView> for TaskNodeView {
    fn link(&self)  -> &ComponentLink<Self> {
        &self.link
    }
}
impl Focussable<FocusEvent> for TaskNodeView {
    fn focus(&self) -> Callback<FocusEvent> {
        //console!(log, "Creating focus callback");
        self.link.callback(|event: FocusEvent| {
            //console!(log, "Got focussed!");
            NodeMsg::Focussed(event)
        })
    }
}
