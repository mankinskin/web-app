use anyhow::Error;
use std::result::{Result};
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use rql::*;
use url::*;
use futures::{Future, FutureExt};

#[derive(Debug, Clone)]
pub enum FetchMethod {
    Post,
    Get,
    Update,
    Delete,
}
#[derive(Debug, Clone)]
pub enum FetchRequest<T> {
    Post(T),
    Get(Id<T>),
    Update(Id<T>, T),
    Delete(Id<T>),
}

impl<T> FetchRequest<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    pub fn build_request(self, url: Url) -> Result<Request, Error> {
        match self {
            Self::Get(id) => {
                Fetch::get_request(url, id)
            },
            Self::Post(data) => {
                Fetch::post_request(url, &data)
            },
            Self::Delete(id) => {
                Fetch::delete_request(url, id)
            },
            Self::Update(id, data) => {
                Fetch::update_request(url, id, &data)
            },
        }
    }
    pub fn method(&self) -> FetchMethod {
        match self {
            Self::Get(_) => {
                FetchMethod::Get
            },
            Self::Post(_) => {
                FetchMethod::Post
            },
            Self::Delete(_) => {
                FetchMethod::Delete
            },
            Self::Update(_, _) => {
                FetchMethod::Update
            },
        }
    }
}
#[derive(Debug)]
pub enum FetchResponse<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    Post(Result<Id<T>, Error>),
    Get(Result<T, Error>),
    Update(Result<(), Error>),
    Delete(Result<(), Error>),
}
impl<T> FetchResponse<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    pub async fn for_method(method: FetchMethod, res: Result<JsValue, Error>) -> Self {
        match method {
            FetchMethod::Post => {
                Self::Post(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            FetchMethod::Get => {
                Self::Get(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            FetchMethod::Delete => {
                Self::Delete(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            FetchMethod::Update => {
                Self::Update(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
        }
    }
}

pub struct Fetch<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    _ty: std::marker::PhantomData<T>,
}
impl<T> Fetch<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    pub fn send_request(url: Url, request: FetchRequest<T>) -> Result<impl Future<Output=FetchResponse<T>> + 'static, anyhow::Error>
    {
        let method = request.method();
        let request = request.build_request(url)?;
        let window = web_sys::window().expect("web_sys window()");
        Ok(JsFuture::from(window.fetch_with_request(&request))
            .then(|result| {
                console!(log, "Got response 1");
                let value = result.expect("got not value");
                assert!(value.is_instance_of::<Response>());
                let response: Response = value.dyn_into().expect("dyn_into Response failed!");
                futures::future::ready(response)
            })
            .then(move |response: Response| {
                console!(log, "Got response 2");
                let promise = response
                    .json()
                    .map_err(|e| anyhow!(format!("{:#?}", e)))
                    .expect("Response json()");
                JsFuture::from(promise)
            })
            .then(move |res: Result<JsValue, JsValue>| {
                console!(log, "Got response 3");
                FetchResponse::for_method(
                    method,
                    res.map_err(|e| anyhow!(format!("JsFuture::from: {:#?}", e)))
                )
            }))
    }
    fn default_request_init(method: &str) -> Result<RequestInit, Error> {
        let headers = Headers::new().unwrap();
        headers.append("content-type", "application/json").unwrap();
        Ok(RequestInit::new()
            .method(method)
            .mode(RequestMode::Cors)
            .headers(&headers)
            .clone()
        )
    }
    fn post_request(url: Url, data: &T) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &url.to_string(),
            &Self::default_request_init("POST")?
                .body(Some(
                    &JsValue::from_serde(data)
                        .map(|v| { console!(log, format!("Post: {:#?}", v)); v })
                        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn get_request(url: Url, id: Id<T>) -> Result<Request, Error> {
        let url = url.to_string() + "/" + &id.to_string();
        Request::new_with_str_and_init(
            &url.to_string(),
            &Self::default_request_init("GET")?
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn delete_request(url: Url, id: Id<T>) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &url.to_string(),
            &Self::default_request_init("DELETE")?
                .body(Some(
                    &JsValue::from_serde(&id)
                        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn update_request(url: Url, id: Id<T>, data: &T) -> Result<Request, Error> {
        Request::new_with_str_and_init(
            &url.to_string(),
            &Self::default_request_init("UPDATE")?
                .body(Some(
                    &JsValue::from_serde(data)
                        .map(|v| { console!(log, format!("Update: {:#?}", v)); v })
                        .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
}
