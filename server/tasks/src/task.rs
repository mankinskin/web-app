pub use plans::{
    *,
    task::*,
};
use yew::{
    *,
};

pub enum Msg {
    Update,
}

pub struct TaskView {
    task: Task,
    link: ComponentLink<Self>,
}

impl Component for TaskView {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut task = Task::new("Do work");
        Self {
            link,
            task,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="task-container">
                <h1 class="task-description">{self.task.description().clone()}</h1>
            </div>
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update => {
                true
            },
            _ => false
        }
    }
}
