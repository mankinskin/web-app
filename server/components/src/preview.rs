use yew::{
    *,
};

pub trait Preview : Component {
    fn preview(props: <Self as Component>::Properties) -> Html;
}

