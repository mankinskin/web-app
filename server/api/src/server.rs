use app_model::{
    auth::{
        credentials::*,
        jwt::*,
    },
    user::*,
    UserSession,
};
use database_table::*;
use rocket::{
    http::*,
    request::FromParam,
    response::*,
};
use rocket_contrib::json::Json;
use std::convert::TryFrom;

#[post("/api/auth/login", data = "<credentials>")]
pub fn login(credentials: Json<Credentials>) -> std::result::Result<Json<UserSession>, Status> {
    let credentials = credentials.into_inner();
    User::find(|user| *user.name() == credentials.username)
        .ok_or(Status::NotFound)
        .and_then(|entry| {
            let user = entry.data();
            if *user.password() == credentials.password {
                Ok(entry)
            } else {
                Err(Status::Unauthorized)
            }
        })
        .and_then(|entry| {
            let user = entry.data().clone();
            let id = entry.id().clone();
            JWT::try_from(&user)
                .map_err(|_| Status::InternalServerError)
                .map(move |jwt| (id, jwt))
        })
        .map(|(id, jwt)| {
            Json(UserSession {
                user_id: id.clone(),
                token: jwt.to_string(),
            })
        })
}
#[post("/api/auth/register", data = "<user>")]
pub fn register(user: Json<User>) -> std::result::Result<Json<UserSession>, Status> {
    let user = user.into_inner();
    if User::find(|u| u.name() == user.name()).is_none() {
        let id = User::insert(user.clone());
        JWT::try_from(&user)
            .map_err(|_| Status::InternalServerError)
            .map(move |jwt| {
                Json(UserSession {
                    user_id: id.clone(),
                    token: jwt.to_string(),
                })
            })
    } else {
        Err(Status::Conflict)
    }
}
