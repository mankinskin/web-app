pub use crate::fetch::*;
use rql::*;
use url::*;
use std::result::{Result};
use futures::{Future, FutureExt};
use std::convert::TryFrom;
use yew::{
    *,
};

#[derive(Debug)]
pub enum RemoteMsg<T>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + 'static
{
    Request(FetchMethod),
    Response(FetchResponse<T>)
}
#[derive(Clone, Debug)]
pub struct RemoteRoute {
    url: Url,
}
impl RemoteRoute {
    pub fn new(url: Url) -> Self {
        Self {
            url,
        }
    }
}
impl From<Url> for RemoteRoute {
    fn from(url: Url) -> Self {
        Self::new(url)
    }
}
impl std::ops::Deref for RemoteRoute {
    type Target = Url;
    fn deref(&self) -> &Self::Target {
        &self.url
    }
}
#[derive(Clone, Debug)]
pub struct RemoteData<T, C>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static,
          C: Component,
{
    route: RemoteRoute,
    link: ComponentLink<C>,
    data: Option<T>,
    id: Option<Id<T>>,
}
impl<T, C> std::ops::Deref for RemoteData<T, C>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static,
          C: Component,
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T, C> RemoteData<T, C>
    where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + std::fmt::Debug + 'static,
          C: Component,
          <C as Component>::Message: From<RemoteMsg<T>>,
{
    pub fn new(route: RemoteRoute, link: ComponentLink<C>) -> Self {
        Self {
            route,
            link,
            data: None,
            id: None,
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
    pub fn route(&self) -> &RemoteRoute {
        &self.route
    }
    pub fn fetch_request(&self, method: FetchMethod)
        -> Result<impl Future<Output=()> + 'static, anyhow::Error> {
        //console!(log, "task_list request");
        let callback = self.responder().clone();
        Ok(
            Fetch::send_request(
                self.route.url.clone(),
                self.method_request(method)
            )?
            .then(move |res: FetchResponse<T>| {
                futures::future::ready(callback.emit(res))
            })
        )
    }
    pub fn responder(&self) -> Callback<FetchResponse<T>> {
        self.link.callback(move |response: FetchResponse<T>| {
            <C as Component>::Message::from(RemoteMsg::Response(response))
        })
    }
    fn post_request(&self) -> FetchRequest<T> {
        FetchRequest::Post(self.data.clone().unwrap())
    }
    fn get_request(&self) -> FetchRequest<T> {
        FetchRequest::Get(self.id)
    }
    fn delete_request(&self) -> FetchRequest<T> {
        FetchRequest::Delete(self.id.unwrap())
    }
    fn update_request(&self) -> FetchRequest<T> {
        FetchRequest::Update(
            self.id.unwrap(),
            self.data.clone().unwrap()
        )
    }
    fn method_request(&self, method: FetchMethod) -> FetchRequest<T> {
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
