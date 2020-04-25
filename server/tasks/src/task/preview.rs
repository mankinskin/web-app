use super::{
    TaskViewProps,
};
use yew::{
    *,
};
#[derive(Debug)]
pub struct TaskPreview {
    props: TaskViewProps,
    link: ComponentLink<Self>,
}
impl Component for TaskPreview {
    type Message = ();
    type Properties = TaskViewProps;
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
                    self.props.data.task.title()
                }</h1>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}
