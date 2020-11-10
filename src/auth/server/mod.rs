pub mod jwt;
use crate::{
    user::*,
};

pub use {
    actix_web::error::{
        Error,
        ErrorConflict,
        ErrorInternalServerError,
        ErrorNotFound,
        ErrorUnauthorized,
    },
    crate::auth::{
        UserSession,
        credentials::*,
    },
    database_table::{
        Database,
        DatabaseTable,
    },
    jwt::*,
    std::convert::TryFrom,
};
pub async fn login<'db, D: Database<'db, User>>(
    credentials: Credentials,
) -> Result<UserSession, Error> {
    DatabaseTable::<'db, D>::find(|user| *user.name() == credentials.username)
        .ok_or(ErrorNotFound("User not found"))
        .and_then(|entry| {
            let user = entry.data();
            if *user.password() == credentials.password {
                Ok(entry)
            } else {
                Err(ErrorUnauthorized("Unauthorized"))
            }
        })
        .and_then(|entry| {
            let user = entry.data().clone();
            let id = entry.id().clone();
            JWT::try_from(&user)
                .map_err(|_| ErrorInternalServerError(""))
                .map(move |jwt| (id, jwt))
        })
        .map(|(id, jwt)| {
            UserSession {
                user_id: id.clone(),
                token: jwt.to_string(),
            }
        })
}
pub async fn register<'db, D: Database<'db, User>>(user: User) -> Result<UserSession, Error> {
    if DatabaseTable::<'db, D>::find(|u| u.name() == user.name()).is_none() {
        let id = DatabaseTable::<'db, D>::insert(user.clone());
        JWT::try_from(&user)
            .map_err(|_| ErrorInternalServerError(""))
            .map(move |jwt| {
                UserSession {
                    user_id: id.clone(),
                    token: jwt.to_string(),
                }
            })
    } else {
        Err(ErrorConflict("Username already taken"))
    }
}
