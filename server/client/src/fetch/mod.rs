use serde::{
    Serialize,
    Deserialize,
};
use seed::{
    fetch::{
        Header,
        Method,
        FetchError,
    },
};
use url::*;
use crate::{
    root,
};
use rql::{
    Id,
};
pub mod query;
pub use query::*;
pub mod status;
pub use status::*;

#[derive(Clone)]
pub enum Response<T> {
    Get(T),
    Post(Id<T>),
    Delete,
    Put,
}

#[derive(Clone)]
pub enum Request<T>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    Get(Query<T>),
    Post(T),
    Put(Query<T>, T),
    Delete(Query<T>),
}
#[derive(Clone)]
pub enum Msg<T>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    Request(Request<T>),
    Response(Response<T>),
    Error(String),
}

fn new_request<'a>(url: Url) -> seed::fetch::Request<'a> {
    let mut req = seed::fetch::Request::new(url.to_string());
    if let Some(session) = root::get_session() {
        req = req.header(Header::authorization(format!("{}", session.token)));
    }
    req
}
pub async fn fetch<T>(url: Url, request: Request<T>) -> Msg<T>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    let res = match request {
        Request::Get(query) => {
            get_request(url, query).await
        },
        Request::Delete(query) => {
            delete_request(url, query).await
        },
        Request::Post(data) => {
            post_request(url, &data).await
        },
        Request::Put(query, data) => {
            put_request(url, query, &data).await
        },
    };
    match res {
        Ok(response) => Msg::Response(response),
        Err(e) => Msg::Error(format!("{:?}", e)),
    }
}
fn query_request<'a, T>(url: Url, query: Query<T>) -> seed::fetch::Request<'a>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    let mut url = url;
    match query {
        Query::Id(id) => {
            url.path_segments_mut().unwrap().push(&id.to_string().clone());
        },
        _ => {}
    }
    new_request(url)
}
async fn delete_request<T>(url: Url, query: Query<T>) -> Result<Response<T>, FetchError>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    seed::fetch::fetch(query_request(url, query).method(Method::Delete))
        .await?
        .check_status()
        .map(|_| Response::Delete)
}
async fn get_request<T>(url: Url, query: Query<T>) -> Result<Response<T>, FetchError>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    seed::fetch::fetch(query_request(url, query).method(Method::Get))
        .await?
        .check_status()?
        .json()
        .await
        .map(|t| Response::Get(t))
}
async fn put_request<T>(url: Url, query: Query<T>, t: &T) -> Result<Response<T>, FetchError>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    seed::fetch::fetch(
        query_request(url, query)
            .method(Method::Put)
            .json(t)?
        )
        .await?
        .check_status()
        .map(|_| Response::Put)
}
async fn post_request<T>(url: Url, t: &T) -> Result<Response<T>, FetchError>
    where T: 'static + for<'de> Deserialize<'de> + Serialize
{
    seed::fetch::fetch(
        new_request(url)
            .method(Method::Post)
            .json(t)?
        )
        .await?
        .check_status()?
        .json()
        .await
        .map(|t| Response::Post(t))
}
