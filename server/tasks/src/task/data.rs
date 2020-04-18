use super::{
    TaskNodeView,
    message::Msg,
};
use plans::{
    task::*,
};
use common::{
    expander::{ExpanderData},
    parent_child::*,
};
use yew::{
    *,
};
#[derive(Properties, Clone, Debug)]
pub struct TaskNodeData {
    pub task: Task,
    pub parent_callback: Option<Callback<<TaskNodeView as Component>::Message>>,
    pub children: Vec<ExpanderData<TaskNodeView>>,
}
impl From<Task> for TaskNodeData {
    fn from(task: Task) -> Self {
        let children = task.children()
            .iter()
            .cloned()
            .enumerate()
            .map(|(_, child)|
                ExpanderData::<TaskNodeView> {
                    element: TaskNodeData::from(child),
                    expanded: false,
                    parent_callback: Callback::noop(),
                }
            )
            .collect();
        Self {
            task,
            children,
            parent_callback: None,
        }
    }
}
impl ChildProps<TaskNodeView> for TaskNodeData {
    fn set_parent_callback(&mut self, callback: Callback<<TaskNodeView as Component>::Message>) {
        self.parent_callback = Some(callback);
    }
    fn get_parent_callback(&self)-> Callback<<TaskNodeView as Component>::Message> {
        self.parent_callback.clone().unwrap_or(Callback::noop())
    }
}
impl UpdateProp<TaskNodeView> for TaskNodeData {
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
#[derive(Properties, Clone, Debug)]
pub struct TaskData {
    pub task: Task,
}
impl From<Task> for TaskData {
    fn from(task: Task) -> Self {
        Self {
            task,
        }
    }
}
