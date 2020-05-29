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
use crate::{
    database::{
        *,
    },
};
use plans::{
    user::*,
    note::*,
    task::*,
    credentials::*,
    jwt::*,
};
use common::{
    database::*,
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
#[get("/tasks/tools")]
fn get_tasks_tools_html() -> Result<NamedFile> {
    get_file("./tasks/index.html")
}
#[get("/user")]
fn get_user_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/profile")]
fn get_profile_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/note")]
fn get_note_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/login")]
fn get_login_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/signup")]
fn get_signup_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/")]
fn get_root_html() -> Result<NamedFile> {
    get_file("./home/index.html")
}
#[get("/tasks")]
fn get_tasks_html() -> Result<NamedFile> {
    get_file("./tasks/index.html")
}
#[get("/budget")]
fn get_budget_html() -> Result<NamedFile> {
    get_file("./tasks/index.html")
}
#[get("/<app>/styles/<file_name>")]
fn get_style_css(app: &RawStr, file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./{}/styles/{}", app, file_name))
}
#[get("/<app>/pkg/<file_name>")]
fn get_pkg_js(app: &RawStr, file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./{}/pkg/{}", app, file_name))
}
#[get("/img/<file_name>")]
fn get_img_file(file_name: &RawStr) -> Result<NamedFile> {
    get_file(format!("./img/{}", file_name))
}
#[get("/api/tasks")]
fn get_tasks<'a>() -> Json<Vec<Entry<Task>>> {
    Json(Task::get_all())
}
#[get("/api/tasks/<id>")]
fn get_task(id: SerdeParam<Id<Task>>) -> Json<Option<Task>> {
    Json(Task::get(*id))
}
#[post("/api/tasks", data="<task>")]
fn post_task(task: Json<Task>) -> Json<Id<Task>> {
    Json(Task::insert(task.clone()))
}

#[get("/api/users")]
fn get_users() -> Json<Vec<Entry<User>>> {
    Json(User::get_all())
}
#[get("/api/users/<id>")]
fn get_user(id: SerdeParam<Id<User>>) -> Json<Option<User>> {
    Json(User::get(id.clone()))
}
#[post("/api/users", data="<user>")]
fn post_user(user: Json<User>) -> Json<Id<User>> {
    Json(User::insert(user.into_inner()))
}
#[delete("/api/users/<id>")]
fn delete_user(id: SerdeParam<Id<User>>) -> Json<Option<User>> {
    Json(User::delete(id.clone()))
}

#[get("/api/notes")]
fn get_notes() -> Json<Vec<Entry<Note>>> {
    Json(Note::get_all())
}
#[get("/api/notes/<id>")]
fn get_note(id: SerdeParam<Id<Note>>) -> Json<Option<Note>> {
    Json(Note::get(*id))
}
#[post("/api/notes", data="<note>")]
fn post_note(note: Json<Note>) -> Json<Id<Note>> {
    Json(Note::insert(note.into_inner()))
}
#[post("/login", data="<credentials>")]
fn login(credentials: Json<Credentials>)
    -> std::result::Result<Json<JWT>, Status>
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
            let user = entry.data();
            JWT::new_for(&user)
                .map_err(|_| Status::InternalServerError)
        })
        .map(|jwt| Json(jwt))
}
#[post("/signup", data="<user>")]
fn signup(user: Json<User>) -> Json<Id<User>> {
    let user = user.into_inner();
    Json(User::insert(user))
}

pub fn start() {
    database::setup();
    rocket::ignite()
        .mount("/",
            routes![
                get_tasks_tools_html,
                get_tasks_html,
                get_budget_html,
                get_user_html,
                get_profile_html,
                get_note_html,
                get_root_html,
                get_login_html,
                get_signup_html,

                get_style_css,
                get_pkg_js,
                get_img_file,

                login,
                signup,

                get_tasks,
                get_task,
                post_task,

                get_users,
                get_user,
                delete_user,
                post_user,

                get_notes,
                get_note,
                post_note,
            ])
        .launch();
}
