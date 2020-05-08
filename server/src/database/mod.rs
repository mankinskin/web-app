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
pub fn post_note(note: Note) -> status::Accepted<Json<Id<Note>>> {
    println!("Got note: {:#?}", note);
    let note_id = DB.note_mut().insert(note);
    status::Accepted(Some(Json(note_id)))
}
pub fn get_note(id: Id<Note>) -> Option<Json<Note>> {
    DB.note()
      .get(id)
      .map(|note| Json(note.clone()))
}
pub fn post_user(user: User) -> status::Accepted<Json<Id<User>>> {
    //let user = User::new("Server");
    println!("Got user: {:#?}", user);
    let user_id = DB.user_mut().insert(user);
    status::Accepted(Some(Json(user_id)))
}
pub fn get_user(_: Id<User>) -> Option<Json<User>> {
    Some(Json(User::new("Server User")))
}
pub fn post_task(task: Task) -> status::Accepted<Json<Id<Task>>> {
    //let task =
    //    TaskBuilder::default()
    //        .title("Server Task".into())
    //        .description("This is the top level task.".into())
    //        .assignees(vec!["Heinz".into(), "Kunigunde".into(), "Andreas".into()])
    //        .children(vec![
    //                    TaskBuilder::default()
    //                        .title("First Item".into())
    //                        .description("This is the first sub task.".into())
    //                        .assignees(vec!["Heinz".into(), "Kunigunde".into()])
    //                        .children(vec![])
    //                        .build()
    //                        .unwrap(),
    //                ])
    //        .build()
    //        .unwrap();
    let task_id = DB.task_mut().insert(task);
    status::Accepted(Some(Json(task_id)))
}
pub fn get_task(id: Id<Task>) -> Option<Json<Task>> {
    DB.task()
      .get(id)
      .map(|task| Json(task.clone()))
}
pub fn get_tasks() -> Option<Json<Vec<Task>>> {
    let tasks: Vec<Task> = DB.task().rows().map(|row| row.data.clone()).collect();
    Some(Json(tasks))
}

pub fn setup() {
    let user_1 = DB.user_mut().insert(User::new("Test User"));
    println!("Test user ID: {}", user_1);
    let _user_2 = DB.user_mut().insert(User::new("Alter Schwede"));
    let _task_1 = DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = DB.task_mut().insert(Task::new("NSA hacken"));
}
