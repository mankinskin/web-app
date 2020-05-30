use yew::{
    *,
};
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
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}
