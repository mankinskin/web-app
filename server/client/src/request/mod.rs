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
