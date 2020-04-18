use super::{
    TaskNodeData,
};
use yew::{
    *,
};
#[derive(Debug)]
pub struct TaskPreview {
    props: TaskNodeData,
    link: ComponentLink<Self>,
}
impl Component for TaskPreview {
    type Message = ();
    type Properties = TaskNodeData;
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
