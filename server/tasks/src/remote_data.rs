use yew::{
    *,
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
use web_sys::{Request, RequestInit, RequestMode, Response};
use std::result::{Result};
use wasm_bindgen_futures::JsFuture;
use futures::{FutureExt};
use wasm_bindgen::JsCast;

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
    pub async fn for_request(request: Request, res: Result<JsValue, Error>) -> Self {
        match request.method().as_str() {
            "POST" => {
                RemoteResponse::Post(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            "GET" => {
                RemoteResponse::Get(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            "DELETE" => {
                RemoteResponse::Delete(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
                )
            },
            "UPDATE" | _ => {
                RemoteResponse::Update(
                    res.and_then(|body|
                        body.into_serde()
                            .map_err(|e| anyhow!(format!("into_serde: {:#?}", e)))
                    )
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
    id: Option<Id<T>>,
    url: Url,
}
impl<T> std::ops::Deref for RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
pub async fn fetch_request<T>(request: Request, responder: Callback<RemoteResponse<T>>)
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    let window = web_sys::window().expect("web_sys window()");
    JsFuture::from(window.fetch_with_request(&request))
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
            RemoteResponse::for_request(
                request,
                res.map_err(|e| anyhow!(format!("JsFuture::from: {:#?}", e)))
            )
        })
        .then(move |resp: RemoteResponse<T>| {
            console!(log, "Got response 4");
            futures::future::ready(responder.emit(resp))
        }).await
}
impl<T> RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    pub fn try_new<S: ToString>(url: S) -> Result<Self, url::ParseError> {
        Url::parse(&url.to_string())
            .map(|url| Self::new(url))
    }
    pub fn new(url: Url) -> Self {
        Self {
            buffer: None,
            id: None,
            url,
        }
    }
    #[allow(unused)]
    pub fn id(&self) -> &Option<Id<T>> {
        &self.id
    }
    #[allow(unused)]
    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn fetch_request(&self, req: RemoteRequest<T>, responder: Callback<RemoteResponse<T>>) -> Result<(), Error> {
        console!(log, "task_list request");
        let request = self.request(req)?;
        wasm_bindgen_futures::spawn_local(
            fetch_request(request, responder)
        );
        Ok(())
    }
    fn default_request_init(method: &str) -> Result<RequestInit, Error> {
        Ok(RequestInit::new()
            .method(method)
            .mode(RequestMode::Cors)
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
    pub fn request(&self, msg: RemoteRequest<T>)
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
    pub fn respond(&mut self, msg: RemoteResponse<T>) -> Result<(), anyhow::Error> {
        match msg {
            RemoteResponse::Get(res) => {
                res.map_err(|e| anyhow!(e))
                   .map(|data| { self.buffer = Some(data); })
            },
            RemoteResponse::Post(res) => {
                res.map_err(|e| anyhow!(e))
                   .map(|id| { self.id = Some(id); })
            },
            RemoteResponse::Delete(res) => {
                res.map_err(|e| anyhow!(e))
            },
            RemoteResponse::Update(res) => {
                res.map_err(|e| anyhow!(e))
            },
        }
    }
}
