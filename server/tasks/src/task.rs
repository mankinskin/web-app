pub use plans::{
    *,
};
use yew::{
    *,
    Properties,
    Renderable,
};

#[derive(Clone, Debug)]
pub enum Msg {
    ToggleExpand,
    ChildMessage(usize, Box<Msg>),
}

#[derive(Properties, Clone, Debug)]
pub struct TreeData {
    pub text: String,
    pub expanded: bool,
    pub children: Vec<TreeData>,
    pub message_parent: Option<Callback<Msg>>,
}

impl TreeData {
    fn update(&mut self, msg: Msg) {
        match msg.clone() {
            Msg::ToggleExpand => {
                self.expanded = !self.expanded;
            },
            Msg::ChildMessage(child_index, child_msg) => {
                self.children[child_index].update(*child_msg);
            },
        }
    }
}

pub struct TreeView {
    props: TreeData,
    link: ComponentLink<Self>,
}

impl TreeView {
    pub fn toggle_expand(&self) -> Callback<ClickEvent> {
        self.link.callback(|_| {
            Msg::ToggleExpand
        })
    }
    pub fn child_messenger(&self, child_index: usize)  -> Callback<Msg>{
        self.link.callback(move |msg| {
            Msg::ChildMessage(child_index, Box::new(msg))
        })
    }
}

impl Component for TreeView {
    type Message = Msg;
    type Properties = TreeData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="tree-node" style="margin-left: 20px" >
                <div onclick=&self.toggle_expand()>
                    <span>{if self.props.expanded {
                        "V"
                    } else {
                        ">"
                    }}</span>
                    <span>{self.props.text.clone()}</span>
                </div>
                {
                    if self.props.expanded {
                        html!{
                            <div>{
                                for self.props.children.iter().cloned().enumerate().map(|(i,child)| html!{
                                    <TreeView
                                    text={child.text}
                                    children={child.children}
                                    expanded={child.expanded}
                                    message_parent={Some(self.child_messenger(i))}
                                    />
                                })
                            }</div>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(message_parent) = &self.props.message_parent {
            // self is child
            message_parent.emit(msg.clone());
            false
        } else {
            // self is root
            self.props.update(msg);
            true
        }
    }
}
