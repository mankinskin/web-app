#[cfg(not(target_arch = "wasm32"))]
pub mod jwt;
#[cfg(not(target_arch = "wasm32"))]
use http::status::StatusCode;
#[cfg(not(target_arch = "wasm32"))]
use jwt::*;

pub mod credentials;
use crate::user::*;
use credentials::*;
use database_table::DatabaseTable;
use rql::Id;
use std::convert::TryFrom;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Id<User>,
    pub token: String,
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn login(credentials: Credentials) -> Result<UserSession, StatusCode> {
    User::find(|user| *user.name() == credentials.username)
        .ok_or(StatusCode::NOT_FOUND)
        .and_then(|entry| {
            let user = entry.data();
            if *user.password() == credentials.password {
                Ok(entry)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        })
        .and_then(|entry| {
            let user = entry.data().clone();
            let id = entry.id().clone();
            JWT::try_from(&user)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                .map(move |jwt| (id, jwt))
        })
        .map(|(id, jwt)| UserSession {
            user_id: id.clone(),
            token: jwt.to_string(),
        })
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn register(user: User) -> Result<UserSession, StatusCode> {
    if User::find(|u| u.name() == user.name()).is_none() {
        let id = User::insert(user.clone());
        JWT::try_from(&user)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .map(move |jwt| UserSession {
                user_id: id.clone(),
                token: jwt.to_string(),
            })
    } else {
        Err(StatusCode::CONFLICT)
    }
}
