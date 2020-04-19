use super::{
    TaskView,
    TaskNodeView,
    message::{
        Msg,
    },
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
    pub parent_callback: Callback<<TaskNodeView as Component>::Message>,
    pub children: Vec<ExpanderData<TaskNodeView>>,
}
impl TaskNodeData {
    pub fn create(task: Task, parent_callback: Callback<<TaskNodeView as Component>::Message>) -> Self {
        let children = task.children()
            .iter()
            .cloned()
            .enumerate()
            .map(|(_, child)|
                ExpanderData::<TaskNodeView> {
                    element: TaskNodeData::create(child, Callback::noop()),
                    expanded: false,
                    parent_callback: Callback::noop(),
                }
            )
            .collect();
        Self {
            task,
            children,
            parent_callback,
        }
    }
}
impl ChildProps<TaskNodeView> for TaskNodeData {
    fn set_parent_callback(&mut self, callback: Callback<<TaskNodeView as Component>::Message>) {
        self.parent_callback = callback;
    }
    fn get_parent_callback(&self)-> Callback<<TaskNodeView as Component>::Message> {
        self.parent_callback.clone()
    }
}
impl UpdateProp<TaskNodeView> for TaskNodeData {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::ChildMessage(child_index, msg) => {
                console!(log, format!("ChildMessage {} {:#?}", child_index, msg));
                match *msg {
                    Msg::ExpanderMessage(m) => {
                        console!(log, format!("ExpanderMessage {:#?}", m));
                        self.children[child_index].update(*m);
                    },

                    _ => {}
                }
            },
            Msg::ExpanderMessage(msg) => {
                console!(log, format!("ExpanderMessage {:#?}", msg));
            },
            Msg::UpdateDescription(_) => {
            },
            Msg::Focussed => {},
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
impl UpdateProp<TaskView> for Task {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::UpdateDescription(value) => {
                console!(log, "Updating Task Description");
                self.update_description(value.clone());
            },
            Msg::ChildMessage(child_index, msg) => {
                self.children_mut()[child_index].update(*msg);
            },
            _ => {}
        }
    }
}
