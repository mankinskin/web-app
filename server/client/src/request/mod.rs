use seed::{
    browser::fetch::*,
};
use plans::{
    user::*,
    credentials::*,
};
use crate::{
    *,
};
use std::result::Result;

pub async fn validate_session_request(session: UserSession) -> Result<(), FetchError>
{
    fetch(
        Request::new("http://localhost:8000/api/token_valid")
            .header(Header::authorization(format!("{}", session.token)))
            .method(Method::Get)
    ).await?
    .check_status()
    .map(|_| ())
}
pub async fn registration_request(user: User)
    -> Result<UserSession, FetchError>
{
    fetch(
        Request::new("http://localhost:8000/users/register")
            .method(Method::Post)
            .json(&user)?
    ).await?
    .check_status()?
    .json()
    .await
}
pub async fn login_request(credentials: Credentials) -> Result<UserSession, FetchError>
{
    fetch(
        Request::new("http://localhost:8000/users/login")
            .method(Method::Post)
            .json(&credentials)?
    ).await?
    .check_status()?
    .json()
    .await
}
