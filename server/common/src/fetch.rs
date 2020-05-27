use anyhow::Error;
use std::result::{Result};
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use url::*;
use futures::{FutureExt};
use yew::{Callback};

#[derive(Debug, Clone)]
pub enum FetchMethod {
    Post,
    Get,
    Update,
    Delete,
}
#[derive(Debug)]
pub struct FetchResponse<T>
    where T: for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    response: Result<T, Error>,
}
impl<T> FetchResponse<T>
    where T: for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    pub async fn build(res: Result<JsValue, Error>) -> Self {
        let res = res.and_then(|body|
            body.into_serde()
                .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
        );
        Self { response: res }
    }
    pub fn is_success(&self) -> bool {
        self.response.is_ok()
    }
    pub fn into_inner(self) -> Result<T, Error> {
        self.response
    }
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
#[derive(Debug, Clone)]
pub enum FetchRequest<T>
    where T: serde::Serialize + Clone + std::fmt::Debug + 'static
{
    Get,
    Delete,
    Post(T),
    Update(T),
}

impl<T> FetchRequest<T>
    where T: serde::Serialize + Clone + std::fmt::Debug + 'static
{
    fn without_body(method: &str, url: Url) -> Result<Request, Error> {
        Request::new_with_str_and_init(
                &url.to_string(),
                &default_request_init(method)?
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn with_body(method: &str, url: Url, body: &T) -> Result<Request, Error> {
        Request::new_with_str_and_init(
                &url.to_string(),
                &default_request_init(method)?
                    .body(Some(
                        &JsValue::from_str(&serde_json::to_string(body)
                            .map(|v| { console!(log, format!("{}: {:#?}", method, v)); v })
                            .map_err(|e| anyhow!(format!("Serde error: {:#?}", e)))?
                        )
                    ))
            )
            .map_err(|e| anyhow!(format!("Request Error: {:#?}", e)))
    }
    fn get_request(url: Url) -> Result<Request, Error> {
        Self::without_body("GET", url)
    }
    fn delete_request(url: Url) -> Result<Request, Error> {
        Self::without_body("DELETE", url)
    }
    fn post_request(url: Url, body: &T) -> Result<Request, Error> {
        Self::with_body("POST", url, body)
    }
    fn update_request(url: Url, body: &T) -> Result<Request, Error> {
        Self::with_body("UPDATE", url, body)
    }
    pub fn build_request(self, url: Url) -> Result<Request, Error> {
        match self {
            Self::Get => {
                Self::get_request(url)
            },
            Self::Delete => {
                Self::delete_request(url)
            },
            Self::Post(data) => {
                Self::post_request(url, &data)
            },
            Self::Update(data) => {
                Self::update_request(url, &data)
            },
        }
    }
    pub fn method(&self) -> FetchMethod {
        match self {
            Self::Get => {
                FetchMethod::Get
            },
            Self::Delete => {
                FetchMethod::Delete
            },
            Self::Post(_) => {
                FetchMethod::Post
            },
            Self::Update(_) => {
                FetchMethod::Update
            },
        }
    }
}
pub struct Fetch<Req, Res>
    where Req: serde::Serialize + Clone + std::fmt::Debug + 'static,
          Res: for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    url: Url,
    request: Option<FetchRequest<Req>>,
    callback: Option<Callback<FetchResponse<Res>>>,
}
impl<Res> Fetch<(), Res>
    where Res: for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    pub fn get(url: Url) -> Self {
        Self {
            request: Some(FetchRequest::Get),
            ..Self::new(url)
        }
    }
    pub fn delete(url: Url) -> Self {
        Self {
            request: Some(FetchRequest::Delete),
            ..Self::new(url)
        }
    }
}
impl<Req, Res> Fetch<Req, Res>
    where Req: serde::Serialize + Clone + std::fmt::Debug + 'static,
          Res: for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    pub fn new(url: Url) -> Self {
        Self {
            url,
            request: None,
            callback: None,
        }
    }
    pub fn post(url: Url, data: Req) -> Self {
        Self {
            request: Some(FetchRequest::Post(data)),
            ..Self::new(url)
        }
    }
    pub fn update(url: Url, data: Req) -> Self {
        Self {
            request: Some(FetchRequest::Update(data)),
            ..Self::new(url)
        }
    }
    pub fn responder(self, callback: Callback<FetchResponse<Res>>) -> Self {
        Self {
            callback: Some(callback),
            ..self
        }
    }
    pub fn send(self) -> Result<(), Error> {
        let url = self.url;
        let request = self
            .request
            .ok_or(anyhow!("FetchRequest not defined"))?
            .build_request(url.clone())?;
        let responder = self
            .callback
            .ok_or(anyhow!("Responder not defined"))?;
        let window = web_sys::window().expect("web_sys window()");
        Ok(
            wasm_bindgen_futures::spawn_local(
                JsFuture::from(window.fetch_with_request(&request))
                    .then(|result| {
                        //console!(log, "Got response 1");
                        let value = result.expect("got not value");
                        assert!(value.is_instance_of::<Response>());
                        let response: Response = value.dyn_into().expect("dyn_into Response failed!");
                        futures::future::ready(response)
                    })
                    .then(move |response: Response| {
                        //console!(log, "Got response 2");
                        let promise = response
                            .json()
                            .map_err(|e| anyhow!(format!("{:#?}", e)))
                            .expect("Response json()");
                        JsFuture::from(promise)
                    })
                    .then(move |res: Result<JsValue, JsValue>| {
                        //console!(log, "Got response 3");
                        FetchResponse::build(
                            res.map_err(|e| anyhow!(format!("JsFuture::from: {:#?}", e)))
                        )
                    })
                    .then(move |res: FetchResponse<Res>| {
                        futures::future::ready(responder.emit(res))
                    })
            )
        )
    }
}
