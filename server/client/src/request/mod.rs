use seed::{
    *,
    browser::service::fetch::{
        FetchObject,
    },
};
use futures::{
    Future,
};
use plans::{
    user::*,
    credentials::*,
};
use crate::{
    *,
};
use rql::{
    *,
};
use database::{
    *,
};
use std::result::Result;

pub fn fetch_all_users(session: UserSession)
    -> impl Future<Output = Result<users::Msg, users::Msg>>
{
    Request::new("http://localhost:8000/api/users")
        .header("authorization", &format!("{}", session.token))
        .method(Method::Get)
            .fetch_json_data(move |data_result: ResponseDataResult<Vec<Entry<User>>>| {
                users::Msg::FetchedUsers(data_result)
            })
}
pub fn fetch_user(id: Id<User>)
    -> impl Future<Output = Result<users::user::Msg, users::user::Msg>>
{
    Request::new(format!("http://localhost:8000/api/users/{}", id))
        .method(Method::Get)
        .fetch_json_data(move |data_result: ResponseDataResult<User>| {
            users::user::Msg::FetchedUser(data_result)
        })
}
pub fn validate_session_request(session: UserSession)
    -> impl Future<Output = Result<root::GMsg, root::GMsg>>
{
    Request::new("http://localhost:8000/api/token_valid")
        .header("authorization", &format!("{}", session.token))
        .method(Method::Get)
        .fetch(move |fetch_object: FetchObject<()>| {
            match fetch_object.response() {
                Ok(response) => {
                    if response.status.is_ok() {
                        GMsg::SetSession(session)
                    } else {
                        GMsg::EndSession
                    }
                },
                Err(_) => GMsg::EndSession,
            }
        })
}
pub fn registration_request(user: &User)
    -> impl Future<Output = Result<register::Msg, register::Msg>>
{
    Request::new("http://localhost:8000/users/register")
        .method(Method::Post)
        .send_json(user)
        .fetch_json_data(move |data_result: ResponseDataResult<UserSession>| {
            register::Msg::RegistrationResponse(data_result)
        })
}
pub fn login_request(credentials: &Credentials)
    -> impl Future<Output = Result<login::Msg, login::Msg>>
{
    Request::new("http://localhost:8000/users/login")
        .method(Method::Post)
        .send_json(credentials)
        .fetch_json_data(move |data_result: ResponseDataResult<UserSession>| {
            login::Msg::LoginResponse(data_result)
        })
}
