use super::{
    Msg,
    ExpanderData,
};
use crate::{
    preview::*,
    parent_child::*,
};
use yew::{
    *,
};
pub struct ExpanderView<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    props: ExpanderData<C>,
    link: ComponentLink<Self>,
}
impl<C> ExpanderView<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    pub fn toggle_expand(&self) -> Callback<ClickEvent> {
        self.link.callback(|_| {
            console!(log, format!("ToggleExpand"));
            Msg::<C>::ToggleExpand
        })
    }
}
impl<C> Child for ExpanderView<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + Clone + ChildProps<C>,
          <C as Component>::Message: std::fmt::Debug + Clone,
{ }
impl<C> Parent<C> for ExpanderView<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug  + ChildProps<C> + Clone,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    fn child_callback(&self, _: usize)  -> Callback<<C as Component>::Message>{
        //console!(log, format!("creating expander callback"));
        self.link.callback(move |msg| {
            <Msg<C> as ChildMessage<C>>::child_message(0, msg)
        })
    }
    fn set_child_callbacks(&mut self) {
        self.props.element.set_parent_callback(self.child_callback(0));
    }
}
impl<C> Component for ExpanderView<C>
    where C: Component + Preview + Child + Clone,
          <C as Component>::Properties: std::fmt::Debug + ChildProps<C> + Clone,
          <C as Component>::Message: std::fmt::Debug + Clone,
{
    type Message = Msg<C>;
    type Properties = ExpanderData<C>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("creating Expander"));
        let mut s = Self {
            props,
            link,
        };
        s.set_child_callbacks();
        s
    }
    fn view(&self) -> Html {
        //console!(log, format!("{:#?}\n-------------------\n", self.props));
        //console!(log, format!("Rendering ExpanderView"));
        let props = self.props.element.clone();
        html!{
            <div class="expander-container">
                <div class="expander-icon" onclick=&self.toggle_expand()>
                    <ion-icon
                        name={
                            if self.props.expanded {
                                "caret-down-outline"
                            } else {
                                "caret-forward-outline"
                            }
                        }
                    ></ion-icon>
                </div>
                {
                    if !self.props.expanded {
                        html!{
                            <div class="expander-content">{
                                C::preview(props.clone())
                            }</div>
                        }
                    } else {
                        html!{
                            <div style="display: contents;">
                                <div class="expander-line"/>
                                <div class="expander-content">
                                    <C with props/>
                                </div>
                            </div>
                        }
                    }
                }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing Expander"));
        self.props = props;
        self.set_child_callbacks();
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        //console!(log, format!("Updating ExpanderView"));
        //self.props.update(msg.clone());
        self.props.parent_callback.emit(msg);
        false
    }
}
