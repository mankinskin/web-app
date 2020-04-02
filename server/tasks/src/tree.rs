use yew::{
    *,
};

#[derive(Clone, Debug)]
pub enum Msg {
    ToggleExpand,
    ChildMessage(usize, Box<Msg>),
}

#[derive(Properties, Clone, Debug)]
pub struct TreeData<P>
    where P: Properties
{
    pub element: P,
    pub expanded: bool,
    pub children: Vec<TreeData<P>>,
    pub message_parent: Option<Callback<Msg>>,
}

impl<P> TreeData<P>
    where P: Properties
{
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

pub struct TreeView<C>
    where C: Component,
          <C as Component>::Properties: std::fmt::Debug,
{
    props: TreeData<<C as Component>::Properties>,
    link: ComponentLink<Self>,
}

impl<C> TreeView<C>
    where C: Component,
          <C as Component>::Properties: std::fmt::Debug,
{
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

impl<C> Component for TreeView<C>
    where C: Component,
          <C as Component>::Properties: std::fmt::Debug,
{
    type Message = Msg;
    type Properties = TreeData<<C as Component>::Properties>;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        let props = self.props.element.clone();
        html!{
            <div class="tree-node" style="margin-left: 20px" >
                <div onclick=&self.toggle_expand()>
                    <span>{if self.props.expanded {
                        "V"
                    } else {
                        ">"
                    }}</span>
                    <C with props/>
                </div>
                {
                    if self.props.expanded {
                        html!{
                            <div>{
                                for self.props
                                        .children
                                        .iter()
                                        .cloned()
                                        .enumerate()
                                        .map(|(child_index,child)| html! {
                                            <TreeView<C>
                                                element={child.element}
                                                children={child.children}
                                                expanded={child.expanded}
                                                message_parent={Some(self.child_messenger(child_index))}
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
#[derive(Properties, Clone, Debug)]
pub struct StringProperty {
    string: String,
}
impl From<String> for StringProperty {
    fn from(string: String) -> Self {
        Self {
            string,
        }
    }
}
impl<'a> From<&'a str> for StringProperty {
    fn from(s: &'a str) -> Self {
        Self {
            string: s.to_string(),
        }
    }
}
impl ToString for StringProperty {
    fn to_string(&self) -> String {
        self.string.clone()
    }
}
#[derive(Debug)]
pub struct StringComponent {
    props: StringProperty,
    link: ComponentLink<Self>,
}
impl Component for StringComponent {
    type Message = ();
    type Properties = StringProperty;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        html! {
            <span>{self.props.to_string()}</span>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}
