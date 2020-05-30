use yew::{
    *,
};
use std::fmt::{Debug, Display};

#[derive(Clone, Debug)]
pub struct StatusStack<S, E>
    where S: Clone,
          E: Clone + Display,
{
    stack: Vec<Result<S, E>>,
}
use std::ops::{Deref};
impl<S, E> Deref for StatusStack<S, E>
    where S: Clone,
          E: Clone + Display,
{
    type Target = Vec<Result<S, E>>;
    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}
impl<S, E> StatusStack<S, E>
    where S: Clone,
          E: Clone + Display,
{
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }
    pub fn push(&mut self, status: Result<S, E>) {
        self.stack.push(status);
    }
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

#[derive(Properties, Clone, Debug)]
pub struct StatusStackData<S, E>
    where S: Clone,
          E: Clone + Display,
{
    pub stack: StatusStack<S, E>,
}

impl<S, E> From<StatusStack<S, E>> for StatusStackData<S, E>
    where S: Clone,
          E: Clone + Display,
{
    fn from(stack: StatusStack<S, E>) -> Self {
        Self {
            stack,
        }
    }
}

pub struct StatusStackView<S, E>
    where S: Clone + 'static,
          E: Clone + Display + 'static,
{
    props: StatusStackData<S, E>,
}
impl<'a, S, E> Component for StatusStackView<S, E>
    where S: Clone + 'static,
          E: Clone + Display + 'static,
{
    type Message = ();
    type Properties = StatusStackData<S, E>;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            props,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="submit-status">{
                if self.props.stack.is_empty() {
                    html!{}
                } else {
                    if self.props.stack.iter().all(|status| status.is_ok()) {
                        html!{
                           <div>{ "Success" }</div>
                        }
                    } else {
                        html!{{
                            for self.props.stack.iter().map(|status| html!{
                                <div>{ match status {
                                    Ok(_) => "Success".to_string(),
                                    Err(err) => format!("Error: {}", err),
                                }}</div>
                            })
                        }}
                    }
                }
            }</div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}
