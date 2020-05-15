use rql::*;
use rocket::{
    response::{
        *,
    },
};
use rocket_contrib::{
    json::Json
};
use plans::{
    user::*,
    note::*,
    task::*,
};
use updatable::{
    *,
};
schema! {
    Schema {
        user: User,
        task: Task,
        note: Note,
    }
}
lazy_static!{
    static ref DB: Schema = Schema::new("test_database", rql::HumanReadable).unwrap();
}

pub trait DatabaseTable<'a> : Sized + Clone + serde::Serialize + Updatable + 'a {
    fn table() -> TableGuard<'a, Self>;
    fn table_mut() -> TableGuardMut<'a, Self>;
    fn post(obj: Self) -> status::Accepted<Json<Id<Self>>> {
        status::Accepted(Some(Json(
            Self::table_mut().insert(obj)
        )))
    }
    fn get(id: Id<Self>) -> Option<Json<Self>> {
        Self::table()
          .get(id)
          .map(|entry| Json(entry.clone()))
    }
    fn delete(id: Id<Self>) -> Option<Json<Self>> {
        Self::table_mut()
          .delete_one(id)
          .map(|entry| Json(entry.clone()))
    }
    fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Option<Json<Self>> {
        Self::table_mut()
          .get_mut(id)
          .map(move |entry| {
              update.update(entry);
              Json(entry.clone())
          })
    }
    fn get_all() -> Option<Json<Vec<Self>>> {
        Some(Json(Self::table()
            .rows()
            .map(|row| row.data.clone())
            .collect()
        ))
    }
}
impl<'a> DatabaseTable<'a> for Note {
    fn table() -> TableGuard<'a, Self> {
        DB.note()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.note_mut()
    }
}
impl<'a> DatabaseTable<'a> for User {
    fn table() -> TableGuard<'a, Self> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.user_mut()
    }
}
impl<'a> DatabaseTable<'a> for Task {
    fn table() -> TableGuard<'a, Self> {
        DB.task()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.task_mut()
    }
}

pub fn setup() {
    let user_1 = DB.user_mut().insert(User::new("Test User"));
    println!("Test user ID: {}", user_1);
    let _user_2 = DB.user_mut().insert(User::new("Alter Schwede"));
    let _task_1 = DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = DB.task_mut().insert(Task::new("NSA hacken"));
}
