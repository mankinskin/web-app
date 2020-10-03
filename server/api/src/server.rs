use jwt::{
    *,
};
use rocket::{
    request::{
        FromParam,
    },
    response::{
        *,
    },
    http::{
        *,
    },
};
use rocket_contrib::{
    json::{
        Json,
    },
};
use plans::{
    credentials::*,
    user::*,
};
use database::{
    *,
};
use std::convert::TryFrom;

#[post("/api/auth/login", data="<credentials>")]
pub fn login(credentials: Json<Credentials>)
    -> std::result::Result<Json<UserSession>, Status>
{
    let credentials = credentials.into_inner();
}
#[post("/api/auth/register", data="<user>")]
pub fn register(user: Json<User>) -> std::result::Result<Json<UserSession>, Status> {
    let user = user.into_inner();
    if User::find(|u| u.name() == user.name()).is_none() {
        let id = User::insert(user.clone());
        JWT::try_from(&user)
            .map_err(|_| Status::InternalServerError)
            .map(move |jwt|
                Json(UserSession {
                    user_id: id.clone(),
                    token: jwt.to_string(),
                })
            )
    } else {
        Err(Status::Conflict)
    }
}
