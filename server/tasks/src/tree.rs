use yew::{
    *,
};
use crate::task::{
    *,
};

#[derive(Clone, Debug)]
pub enum Msg {
    ToggleExpand,
    SetParentMessenger(Callback<Self>),
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
                console!(log, format!("Toggle"));
                self.expanded = !self.expanded;
            },
            Msg::SetParentMessenger(callback) => {
                console!(log, format!("Toggle"));
                self.message_parent = Some(callback);
            },
            Msg::ChildMessage(child_index, child_msg) => {
                self.children[child_index].update(*child_msg);
            },
        }
    }
}

pub struct TreeView<C>
    where C: Component + Preview,
          <C as Component>::Properties: std::fmt::Debug,
{
    props: TreeData<<C as Component>::Properties>,
    link: ComponentLink<Self>,
}

impl<C> TreeView<C>
    where C: Component + Preview,
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
    pub fn has_children(&self) -> bool {
        !self.props.children.is_empty()
    }
}

impl<C> Component for TreeView<C>
    where C: Component + Preview,
          <C as Component>::Properties: std::fmt::Debug,
{
    type Message = Msg;
    type Properties = TreeData<<C as Component>::Properties>;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Self {
            props,
            link,
        };
        s.props.children = s.props.children
            .iter()
            .cloned()
            .enumerate()
            .map(|(child_index, mut child)| {
                let callback = s.child_messenger(child_index);
                child.message_parent = Some(callback.clone());
                callback.emit(Msg::SetParentMessenger(callback.clone()));
                child
            })
            .collect();
        s
    }
    fn view(&self) -> Html {
        console!(log, format!("{:#?}\n-------------------\n", self.props));
        let props = self.props.element.clone();
        html!{
            <table class="tree-node">
                <col class="tree-icon-column"/>
                <col/>
                <tr class="tree-header" onclick=&self.toggle_expand()>
                    { // icon
                            html!{
                                <th class="tree-icon">
                                <ion-icon
                                    name={
                                    if self.props.expanded {
                                        "caret-down-outline"
                                    } else {
                                        "caret-forward-outline"
                                    }
                                }></ion-icon>
                                </th>
                            }
                    }
                    <th class="tree-element">{
                        C::preview(props.clone())
                    }</th>
                </tr>
                { // element
                    if self.props.expanded {
                        html!{
                            <tr>
                                {
                                        html!{
                                            <td class="tree-line-cell">
                                                <div class="tree-line"/>
                                            </td>
                                        }
                                }
                                <td class="tree-element">
                                    <C with props/>
                                </td>
                            </tr>
                        }
                    } else { html!{} }
                }
                { // element
                    if self.props.expanded && self.has_children() {
                        html!{
                            for self.props
                            .children
                            .iter()
                            .cloned()
                            .map(|props| html! {
                                <tr>
                                    <td class="tree-line-cell">
                                        <div class="tree-line"/>
                                    </td>
                                    <td>
                                        <TreeView<C> with props/>
                                    </td>
                                </tr>
                            })
                        }
                    } else { html!{} }
                }
            </table>
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
