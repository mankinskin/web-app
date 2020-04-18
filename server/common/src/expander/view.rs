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
    fn child_callback(link: &ComponentLink<Self>, _: usize)  -> Callback<<C as Component>::Message>{
        //console!(log, format!("creating expander callback"));
        link.callback(move |msg| {
            <Msg<C> as ChildMessage<C>>::child_message(0, msg)
        })
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
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("{:#?}\n-------------------\n", self.props));
        //console!(log, format!("Rendering ExpanderView"));
        let mut props = self.props.element.clone();
        props.set_parent_callback(<Self as Parent<C>>::child_callback(&self.link, 0));
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
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        //console!(log, format!("Updating ExpanderView"));
        //self.props.update(msg.clone());
        self.props.parent_callback.emit(msg);
        false
    }
}
