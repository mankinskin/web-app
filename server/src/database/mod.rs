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

#[derive(Clone, Debug, Default)]
pub struct REST<T> {
    _ty: std::marker::PhantomData<T>,
}
impl<'a, T> REST<T>
    where T: DatabaseTable<'a> + Clone + 'a
{
    pub fn post(obj: T) -> status::Accepted<Json<Id<T>>> {
        let id = T::table_mut().insert(obj);
        status::Accepted(Some(Json(id)))
    }
    pub fn get(id: Id<T>) -> Option<Json<T>> {
        T::table()
          .get(id)
          .map(|entry| Json(entry.clone()))
    }
    pub fn get_all() -> Option<Json<Vec<T>>> {
        Some(Json(T::table()
            .rows()
            .map(|row| row.data.clone())
            .collect()
        ))
    }
}
pub trait DatabaseTable<'a> : Sized + serde::Serialize {
    fn table() -> TableGuard<'a, Self>;
    fn table_mut() -> TableGuardMut<'a, Self>;
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
