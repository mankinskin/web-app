use serde::{
    Serialize,
    Deserialize,
};
use seed::{
    fetch::{
        Request,
        Header,
        Method,
        FetchError,
        fetch,
    },
};
use url::*;
use crate::{
    root,
    fetched::{
        query::*,
    },
};
#[derive(Clone)]
pub enum FetchResponse<T> {
    Get(T),
    Post(Query<T>),
    Delete,
    Put,
}
#[derive(Clone)]
pub struct Fetch<T> {
    pub url: Url,
    pub query: Query<T>,
}
impl<T> Fetch<T>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    pub fn new(url: Url, query: Query<T>) -> Self {
        Self {
            url,
            query,
        }
    }
    fn request(&self) -> Request {
        let mut url = self.url.clone();
        match self.query {
            Query::Id(id) => {
                url.path_segments_mut().unwrap().push(&id.to_string().clone());
            },
            _ => {}
        }
        let mut req = Request::new(url.to_string());
        if let Some(session) = root::get_session() {
            req = req.header(Header::authorization(format!("{}", session.token)));
        }
        req
    }
    pub async fn delete_request(&self) -> Result<FetchResponse<T>, FetchError> {
        fetch(
            self.request()
            .method(Method::Delete)
            )
            .await?
            .check_status()
            .map(|_| FetchResponse::Delete)
    }
    pub async fn get_request(&self) -> Result<FetchResponse<T>, FetchError> {
        fetch(
            self.request()
            .method(Method::Get)
            )
            .await?
            .check_status()?
            .json()
            .await
            .map(|t| FetchResponse::Get(t))
    }
    pub async fn put_request(&self, t: &T) -> Result<FetchResponse<T>, FetchError> {
        let mut req = self.request()
            .method(Method::Put);
        req = req.json(t)?;  // serialize from T
        fetch(req)
            .await?
            .check_status()
            .map(|_| FetchResponse::Put)
    }
    pub async fn post_request(&self, t: &T) -> Result<FetchResponse<T>, FetchError> {
        let mut req = self.request()
            .method(Method::Post);
        req = req.json(t)?;  // serialize from T
        fetch(req)
            .await?
            .check_status()?
            .json()
            .await
            .map(|t| FetchResponse::Post(t))
    }
}
