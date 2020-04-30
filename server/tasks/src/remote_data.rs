use yew::{
    format::{
        Json,
        Text,
    },
    services::fetch::{
        StatusCode,
    },
};
use anyhow::{
    Error
};
use rql::{
    *
};
use url::{
    *
};
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use std::result::{Result};
use wasm_bindgen_futures::JsFuture;
use futures::{Future, FutureExt};

#[derive(Debug)]
pub enum RemoteMsg<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    Request(RemoteRequest<T>),
    Response(RemoteResponse<T>)
}
#[derive(Debug, Clone)]
pub enum RemoteRequest<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    Post(T),
    Get(Id<T>),
    Update(T),
    Delete(Id<T>),
}
#[derive(Debug)]
pub enum RemoteResponse<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    Post(Result<Id<T>, Error>),
    Get(Result<T, Error>),
    Update(Result<(), Error>),
    Delete(Result<(), Error>),
}
impl<T> RemoteResponse<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug
{
    pub async fn for_request(request: Request, value: JsValue) -> Self {
        match request.method().as_str() {
            "POST" => {
                RemoteResponse::Post(
                    value.into_serde()
                        .map_err(|e| anyhow!(format!("{:#?}", e)))
                )
            },
            "GET" => {
                RemoteResponse::Get(
                    value.into_serde()
                        .map_err(|e| anyhow!(format!("{:#?}", e)))
                )
            },
            "DELETE" => {
                RemoteResponse::Delete(
                        Ok(())
                )
            },
            "UPDATE" | _ => {
                RemoteResponse::Update(
                        Ok(())
                )
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    buffer: Option<T>,
    reference: Id<T>,
    url: Url,
}
impl<T> std::ops::Deref for RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl<T> RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static
{
    pub fn try_new<S: ToString>(reference: Id<T>, url: S) -> Result<Self, url::ParseError> {
        Url::parse(&url.to_string()).map(|url|
            Self {
                buffer: None,
                reference,
                url,
            })
    }
    pub fn new(reference: Id<T>, url: Url) -> Self {
        Self {
            buffer: None,
            reference,
            url,
        }
    }
    pub fn id(&self) -> &Id<T> {
        &self.reference
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
    fn default_request_init(method: &str) -> Result<RequestInit, Error> {
        Ok(RequestInit::new()
            .method(method)
            .mode(RequestMode::Cors)
            //.headers(
            //    &JsValue::from_serde(&vec![
            //        "content-type", "application/json"
            //    ])
            //    .map_err(|e| anyhow!(format!("Request Error: Serde error: {:#?}", e)))?
            //)
            .clone()
        )
    }
    fn post_request(&self, data: &T) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &self.url.clone().to_string(),
            &Self::default_request_init("POST")?
                .body(Some(
                    &JsValue::from_serde(data)
                        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn get_request(&self, id: Id<T>) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &self.url.clone().to_string(),
            &Self::default_request_init("GET")?
                //.body(Some(
                //    &JsValue::from_serde(&id)
                //        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                //))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn delete_request(&self, id: Id<T>) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &self.url.clone().to_string(),
            &Self::default_request_init("DELETE")?
                .body(Some(
                    &JsValue::from_serde(&id)
                        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    pub fn request(&mut self, msg: RemoteRequest<T>)
        -> Result<Request, Error> {
        match msg {
            RemoteRequest::Get(id) => {
                self.get_request(id)
            },
            RemoteRequest::Post(data) => {
                self.post_request(&data)
            },
            RemoteRequest::Delete(id) => {
                self.delete_request(id)
            },
            RemoteRequest::Update(data) => {
                // TODO
                self.post_request(&data)
            },
        }
    }
}
