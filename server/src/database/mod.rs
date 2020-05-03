use rql::*;
use rouille::{
    Request,
    Response,
    input::json::*,
};
use std::{
    fs::{
        File,
    },
    path::{
        Path,
    },
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
pub fn get_html<P: AsRef<Path>>(path: P) -> Response {
    match File::open(path) {
        Ok(file) => Response::from_file("text/html", file),
        Err(e) => Response::text(e.to_string()),
    }
}
pub fn post_note(req: &Request) -> Response {
    //let user = User::new("Server");
    let note: Note = try_or_400!(json_input(req));
    println!("Got note: {:#?}", note);
    let note_id = DB.note_mut().insert(note);
    rouille::Response::json(&note_id)
}
pub fn get_note(_: &Request, id: Id<Note>) -> Response {
    match DB.note().get(id).clone() {
        Some(note) => rouille::Response::json(&note),
        None => rouille::Response::empty_404(),
    }
}
pub fn post_user(req: &Request) -> Response {
    //let user = User::new("Server");
    let user: User = try_or_400!(json_input(req));
    println!("Got user: {:#?}", user);
    let user_id = DB.user_mut().insert(user);
    rouille::Response::json(&user_id)
}
pub fn get_user(_: &Request, _: Id<User>) -> Response {
    let user = User::new("Server User");
    rouille::Response::json(&user)
    //match DB.user().get(id).clone() {
    //    Some(user) => rouille::Response::json(&user),
    //    None => rouille::Response::empty_404(),
    //}
}
pub fn post_task(req: &Request) -> Response {
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
    let task: Task = try_or_400!(json_input(req));
    println!("Got task: {:#?}", task);
    let task_id = DB.task_mut().insert(task);
    rouille::Response::json(&task_id)
}
pub fn get_task(_: &Request, id: Id<Task>) -> Response {
    match DB.task().get(id).clone() {
        Some(task) => rouille::Response::json(&task),
        None => rouille::Response::empty_404(),
    }
}
pub fn get_tasks(_req: &Request) -> Response {
    let tasks: Vec<Task> = DB.task().rows().map(|row| row.data.clone()).collect();
    rouille::Response::json(&tasks)
}

pub fn setup() {
    let _user_1 = DB.user_mut().insert(User::new("Test User"));
    let _user_2 = DB.user_mut().insert(User::new("Alter Schwede"));
    let _task_1 = DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = DB.task_mut().insert(Task::new("NSA hacken"));
}
