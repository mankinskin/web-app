use super::{
    Msg,
    ExpanderProps,
};
use yew::{
    *,
};
pub struct ExpanderView
{
    props: ExpanderProps,
    link: ComponentLink<Self>,
}
impl ExpanderView
{
    pub fn toggle_expand(&self) -> Callback<ClickEvent> {
        self.link.callback(|_| {
            console!(log, format!("ToggleExpand"));
            Msg::ToggleExpand
        })
    }
}
impl Component for ExpanderView
{
    type Message = Msg;
    type Properties = ExpanderProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //console!(log, format!("Creating ExpanderView"));
        Self {
            props,
            link,
        }
    }
    fn view(&self) -> Html {
        //console!(log, format!("Rendering ExpanderView"));
        html!{
            <div class="expander-container">
                <div class="expander-icon" onclick=&self.toggle_expand()>
                    <ion-icon
                        name={
                            if self.props.is_expanded() {
                                "caret-down-outline"
                            } else {
                                "caret-forward-outline"
                            }
                        }
                    ></ion-icon>
                </div>
                {
                    html!{
                        <div style="display: contents;">
                            <div class="expander-line"/>
                            <div class="expander-content">{
                                self.props.children.render()
                            }</div>
                        </div>
                    }
                }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        //console!(log, format!("Changing ExpanderView"));
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        //console!(log, format!("Updating ExpanderView"));
        self.props.parent_callback.emit(msg);
        false
    }
}
