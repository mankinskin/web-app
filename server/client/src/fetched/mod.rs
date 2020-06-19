use seed::{
    fetch::{
        Method,
        FetchError,
    },
    prelude::{
        Orders,
        JsValue,
    },
};
use crate::{
    root,

};
pub mod fetch;
pub use fetch::{
    FetchResponse,
    Fetch,
};
pub mod query;
pub use query::*;
pub mod status;
pub use status::*;

use serde::{
    Serialize,
    Deserialize,
};
#[derive(Clone)]
pub enum Msg<F> {
    Clear,
    Fetch(seed::fetch::Method),
    Response(fetch::FetchResponse<F>),
    Error(String),
}
#[derive(Clone)]
pub struct Fetched<F>
    where F: Clone + for<'de> Deserialize<'de> + Serialize + 'static,
{
    fetch: Fetch<F>,
    status: Status<F>,
}
impl<F> Fetched<F>
    where F: Clone + for<'de> Deserialize<'de> + Serialize + 'static,
{
    pub fn status_mut(&mut self) -> &mut Status<F> {
        &mut self.status
    }
    pub fn status(&self) -> &Status<F> {
        &self.status
    }
    pub fn query(&self) -> &Query<F> {
        &self.fetch.query
    }
    pub fn empty(url: url::Url, query: Query<F>) -> Self {
        Self {
            fetch: Fetch::new(url, query),
            status: Status::Empty,
        }
    }
    pub fn ready(url: url::Url, data: F, query: Query<F>) -> Self {
        Self {
            fetch: Fetch::new(url, query),
            status: Status::Ready(data),
        }
    }
    async fn send_request(&self, method: Method) -> Result<FetchResponse<F>, FetchError> {
        match method {
            Method::Get => {
                self.fetch.get_request().await
            },
            Method::Delete => {
                self.fetch.delete_request().await
            },
            Method::Post => {
                match self.status.clone().get_ready() {
                    Ok(f) => self.fetch.post_request(&f).await,
                    Err(e) => Err(e),
                }
            },
            Method::Put => {
                match self.status.clone().get_ready() {
                    Ok(f) => self.fetch.put_request(&f).await,
                    Err(e) => Err(e),
                }
            },
            _ => {Err(FetchError::RequestError(JsValue::from_str("Invalid Method")))},
        }
    }
    async fn fetch(self, method: seed::fetch::Method) -> Msg<F> {
        match self.send_request(method).await {
           Ok(t) => Msg::Response(t),
           Err(e) => Msg::Error(format!("{:?}", e)),
        }
    }
    fn process_response(&mut self, response: FetchResponse<F>) {
        match response {
            FetchResponse::Get(f) => {
                self.status = Status::Ready(f);
            },
            FetchResponse::Post(query) => {
                self.fetch.query = query;
            },
            FetchResponse::Put => {
            },
            FetchResponse::Delete => {
            },
        }
    }
    pub fn update(&mut self, msg: Msg<F>, orders: &mut impl Orders<Msg<F>, root::GMsg>) {
        match msg {
            Msg::Fetch(method) => {
                orders.perform_cmd(self.clone().fetch(method));
            },
            Msg::Response(resp) => {
                self.process_response(resp);
            },
            Msg::Error(e) => {
                self.status = Status::Failed(e);
            },
            Msg::Clear => {
                self.status = Status::Empty;
            },
        }
    }
}

