use app_model::{
    auth::{
        credentials::*,
        jwt::*,
    },
    UserSession,
    project::Project,
    task::Task,
    user::User,
};
use database_table::*;
use rocket::{
    http::*,
    request::FromParam,
    response::*,
    post,
};
use rocket_contrib::json::Json;
use std::convert::TryFrom;
use rql::*;
use std::sync::Mutex;
use lazy_static::lazy_static;
use seqraph::*;

schema! {
    pub Schema {
        user: User,
        task: Task,
        project: Project,
    }
}
lazy_static! {
    pub static ref TG: Mutex<SequenceGraph<char>> = Mutex::new(SequenceGraph::new());
    pub static ref DB: Schema = Schema::new("binance_bot_database", rql::BinaryStable).unwrap();
}
impl<'db> Database<'db, User> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.user_mut()
    }
}
impl<'db> Database<'db, Project> for Schema {
    fn table() -> TableGuard<'db, Project> {
        DB.project()
    }
    fn table_mut() -> TableGuardMut<'db, Project> {
        DB.project_mut()
    }
}
impl<'db> Database<'db, Task> for Schema {
    fn table() -> TableGuard<'db, Task> {
        DB.task()
    }
    fn table_mut() -> TableGuardMut<'db, Task> {
        DB.task_mut()
    }
}

#[post("/api/auth/login", data = "<credentials>")]
pub fn login(credentials: Json<Credentials>) -> std::result::Result<Json<UserSession>, Status> {
    let credentials = credentials.into_inner();
    <User as DatabaseTable<'_, Schema>>::find(|user| *user.name() == credentials.username)
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
    if <User as DatabaseTable<'_, Schema>>::find(|u| u.name() == user.name()).is_none() {
        let id = <User as DatabaseTable<'_, Schema>>::insert(user.clone());
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
