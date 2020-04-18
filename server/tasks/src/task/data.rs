use super::{
    TaskView,
    message::Msg,
};
use plans::{
    task::*,
};
use common::{
    expander::{ExpanderData, ExpanderView},
    parent_child::*,
};
use yew::{
    *,
};

#[derive(Properties, Clone, Debug)]
pub struct TaskData {
    pub task: Task,
    pub message_parent: Option<Callback<<TaskView as Component>::Message>>,
    pub children: Vec<ExpanderData<TaskView>>,
}
impl TaskData {
    pub fn from_task(task: Task) -> Self {
        let children = task.children()
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
                //console!(log, format!("ExpanderMessage {} {:#?}", child_index, msg));
                self.children[child_index].update(*msg);
            },
            Msg::UpdateDescription(value) => {
                self.task.update_description(value.clone());
            }
            Msg::Noop => {},
        }
    }
}
