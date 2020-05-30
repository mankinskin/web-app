use super::{
    TaskView,
    TaskNodeView,
    message::{
        *,
    },
};
use plans::{
    task::*,
};
use components::{
    expander::{
        *,
    },
    parent_child::*,
};
use yew::{
    *,
};
use stdweb::{
    web::{
        event::*,
        HtmlElement,
        IHtmlElement,
        IElement,
    },
    unstable::{
        TryFrom,
    },
};
#[derive(Properties, Clone, Debug)]
pub struct TaskProps {
    pub task: Task,
}
impl From<Task> for TaskProps {
    fn from(task: Task) -> Self {
        Self {
            task,
        }
    }
}
#[derive(Properties, Clone, Debug)]
pub struct TaskTreeData {
    pub task: Task,
    pub children: Vec<TaskTreeNodeData>,
}
impl From<Task> for TaskTreeData {
    fn from(task: Task) -> Self {
        let children =
            task.children()
            .iter()
            .cloned()
            .map(|child| TaskTreeNodeData::from(child))
            .collect();
        Self {
            task,
            children,
        }
    }
}
#[derive(Properties, Clone, Debug)]
pub struct TaskTreeRootProps {
    pub data: TaskTreeData,
}
impl TaskTreeRootProps {
    pub fn create_root(task: Task) -> Self {
        Self {
            data: TaskTreeData::from(task),
        }
    }
}
#[derive(Clone, Debug)]
pub struct TaskTreeNodeData {
    pub tree_data: TaskTreeData,
    pub expander_data: ExpanderData,
}
impl From<Task> for TaskTreeNodeData {
    fn from(task: Task) -> Self {
        Self {
            tree_data: TaskTreeData::from(task),
            expander_data: ExpanderData::from(true),
        }
    }
}
#[derive(Properties, Clone, Debug)]
pub struct TaskViewProps {
    pub data: TaskTreeData,
    pub parent_callback: Callback<<TaskView as Component>::Message>,
}
impl TaskViewProps {
    pub fn with_callback(
        data: TaskTreeData,
        parent_callback: Callback<<TaskView as Component>::Message>) -> Self {
        Self {
            data,
            parent_callback,
        }
    }
}
#[derive(Properties, Clone, Debug)]
pub struct TaskTreeNodeProps {
    pub data: TaskTreeNodeData,
    pub node_callback: Callback<<TaskNodeView as Component>::Message>,
}
impl TaskTreeNodeProps {
    pub fn with_callback(
        data: TaskTreeNodeData,
        node_callback: Callback<<TaskNodeView as Component>::Message>
        ) -> Self {
        Self {
            data,
            node_callback,
        }
    }
}
fn set_margin_to_focussed_element_left(event: FocusEvent) {
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
        let body_cmd = format!("margin-left: {}px", -offset_left);
        //console!(log, "applying {}", cmd.clone());
        body.set_attribute("style", &body_cmd).unwrap();
    }
}
impl MessageUpdate<RootMsg> for TaskTreeRootProps {
    fn update(&mut self, msg: RootMsg) {
        console!(log, format!("Updating TaskTreeRootProps"));
        console!(log, format!("{:#?}", msg));
        match msg {
            RootMsg::Passthrough(m) => {
                self.data.update(m);
            },
            RootMsg::Focussed(event) => {
                set_margin_to_focussed_element_left(event);
            },
        }
    }
}
impl MessageUpdate<TaskMsg> for TaskTreeData {
    fn update(&mut self, msg: TaskMsg) {
        console!(log, format!("Updating TaskTreeData"));
        console!(log, format!("{:#?}", msg));
        match msg.clone() {
            TaskMsg::UpdateDescription(value) => {
                self.task.update_description(value);
            },
            TaskMsg::ChildMessage(i, childmsg) => {
                self.children[i].update(*childmsg);
            },
        }
    }
}
impl MessageUpdate<NodeMsg> for TaskTreeNodeData {
    fn update(&mut self, msg: NodeMsg) {
        console!(log, format!("Updating TaskTreeNodeData"));
        console!(log, format!("{:#?}", msg));
        match msg.clone() {
            NodeMsg::ExpanderMessage(m) => {
                self.expander_data.update(*m);
            },
            NodeMsg::Passthrough(m) => {
                self.tree_data.update(m);
            },
            NodeMsg::Focussed(event) => {
                //set_margin_to_focussed_element_left(event);
            },
        }
    }
}
