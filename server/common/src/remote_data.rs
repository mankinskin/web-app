pub use crate::fetch::*;
use rql::*;
use url::*;
use std::result::{Result};
use futures::{Future};

#[derive(Debug)]
pub enum RemoteMsg<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    Request(FetchMethod),
    Response(FetchResponse<T>)
}
#[derive(Clone, Debug)]
pub struct RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    data: Option<T>,
    id: Option<Id<T>>,
    url: Url,
}
impl<T> std::ops::Deref for RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T> RemoteData<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static
{
    pub fn try_new<S: ToString>(url: S) -> Result<Self, url::ParseError> {
        Url::parse(&url.to_string())
            .map(|url| Self::new(url))
    }
    pub fn new(url: Url) -> Self {
        Self {
            data: None,
            id: None,
            url,
        }
    }
    pub fn data(&self) -> &Option<T> {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut Option<T> {
        &mut self.data
    }
    pub fn set_data(&mut self, data: T) {
        self.data = Some(data);
    }
    pub fn id(&self) -> &Option<Id<T>> {
        &self.id
    }
    pub fn id_mut(&mut self) -> &mut Option<Id<T>> {
        &mut self.id
    }
    pub fn set_id(&mut self, id: Id<T>) {
        self.id = Some(id);
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn url_mut(&mut self) -> &mut Url {
        &mut self.url
    }
    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }
    pub fn fetch_request(&self, method: FetchMethod)
        -> Result<impl Future<Output=FetchResponse<T>> + 'static, anyhow::Error> {
        //console!(log, "task_list request");
        Fetch::send_request(
            self.url.clone(),
            self.request(method)
        )
    }
    pub fn post_request(&self) -> FetchRequest<T> {
        FetchRequest::Post(self.data.clone().unwrap())
    }
    pub fn get_request(&self) -> FetchRequest<T> {
        FetchRequest::Get(self.id.unwrap())
    }
    pub fn delete_request(&self) -> FetchRequest<T> {
        FetchRequest::Delete(self.id.unwrap())
    }
    pub fn update_request(&self) -> FetchRequest<T> {
        FetchRequest::Update(
            self.id.unwrap(),
            self.data.clone().unwrap()
        )
    }
    pub fn request(&self, method: FetchMethod) -> FetchRequest<T> {
        match method {
            FetchMethod::Get => {
                self.get_request()
            },
            FetchMethod::Post => {
                self.post_request()
            },
            FetchMethod::Delete => {
                self.delete_request()
            },
            FetchMethod::Update => {
                self.update_request()
            },
        }
    }
    pub fn respond(&mut self, msg: FetchResponse<T>) -> Result<(), anyhow::Error> {
        match msg {
            FetchResponse::Get(res) => {
                res.map_err(|e| anyhow!(e))
                   .map(|data| { self.data = Some(data); })
            },
            FetchResponse::Post(res) => {
                res.map_err(|e| anyhow!(e))
                   .map(|id| { self.id = Some(id); })
            },
            FetchResponse::Delete(res) => {
                res.map_err(|e| anyhow!(e))
            },
            FetchResponse::Update(res) => {
                res.map_err(|e| anyhow!(e))
            },
        }
    }
}
