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
    json::*,
};
use plans::{
    user::*,
    note::*,
    task::*,
    project::*,
    credentials::*,
};
use crate::{
    jwt::*,
};
use database::{
    *,
};
use rql::{
    *,
};
use std::io::{
    Result,
};
use std::str::FromStr;
use std::{
    path::{
        Path,
    },
};
use std::convert::TryFrom;
struct SerdeParam<T>(T)
    where T: FromStr;

impl<T> From<T> for SerdeParam<T>
    where T: FromStr
{
    fn from(o: T) -> Self {
        Self(o)
    }
}
impl<T> std::ops::Deref for SerdeParam<T>
    where T: FromStr
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'r, T> FromParam<'r> for SerdeParam<T>
    where T: FromStr,
          <T as FromStr>::Err: std::fmt::Display
{
    type Error = anyhow::Error;
    fn from_param(param: &'r RawStr) -> std::result::Result<Self, Self::Error> {
        T::from_str(param.as_str())
            .map(|t: T| Self::from(t))
            .map_err(|e|
                anyhow!(format!("Failed to parse \'{}\': {}", param, e)))
    }
}

pub fn get_file<P: AsRef<Path>>(path: P) -> Result<NamedFile> {
    NamedFile::open(path)
}
#[get("/")]
fn get_root_html() -> Result<NamedFile> {
    get_html("".into())
}
static CLIENT_DIR: &'static str = "client";
#[get("/<app>")]
fn get_html(app: &RawStr) -> Result<NamedFile> {
    let _ = app;
    get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/users/<id>")]
fn user_page(id: &RawStr) -> Result<NamedFile> {
    let _ = id;
    get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/projects/<id>")]
fn project_page(id: &RawStr) -> Result<NamedFile> {
    let _ = id;
    get_file(format!("./{}/app.html", CLIENT_DIR))
}
#[get("/<dir>/styles/<file_name>")]
fn get_style_css(dir: &RawStr, file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./{}/styles/{}", dir, file_name))
}
#[get("/pkg/<file_name>")]
fn get_pkg_js(file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./{}/pkg/{}", CLIENT_DIR, file_name))
}
#[get("/img/<file_name>")]
fn get_img_file(file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./img/{}", file_name))
}

define_api!(Project);
define_api!(User);
define_api!(Task);
define_api!(Note);

#[get("/api/token_valid")]
fn token_valid(token: JWT) {
    let _ = token;
}
#[post("/users/login", data="<credentials>")]
fn login(credentials: Json<Credentials>)
    -> std::result::Result<Json<UserSession>, Status>
{
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
        .map(|(id, jwt)|
             Json(UserSession {
                 user_id: id.clone(),
                 token: jwt.to_string(),
             })
        )
}
#[post("/users/register", data="<user>")]
fn register(user: Json<User>) -> std::result::Result<Json<()>, Status> {
    let user = user.into_inner();
    if User::find(|u| u.name() == user.name()).is_none() {
        User::insert(user);
        Ok(Json(()))
    } else {
        Err(Status::Conflict)
    }
}
pub fn start() {
    rocket::ignite()
        .mount("/",
            vec![
                routes![
                    get_root_html,
                    get_html,
                    user_page,
                    project_page,

                    token_valid,

                    get_style_css,
                    get_pkg_js,
                    get_img_file,

                    login,
                    register,

                ],
                define_api_routes!(Task),
                define_api_routes!(Project),
                define_api_routes!(User),
                define_api_routes!(Note),
            ].concat()
        )
        .launch();
}
