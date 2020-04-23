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
pub fn get_user(_req: &Request) -> Response {
    let user = User::new("Server");
    rouille::Response::json(&user)
}
pub fn post_note(req: &Request) -> Response {
    let note: Note = try_or_400!(json_input(req));
    println!("Got note: {:#?}", note);
    rouille::Response::empty_204()
}
pub fn get_task(_req: &Request) -> Response {
    let task =
        TaskBuilder::default()
            .title("Server Task".into())
            .description("This is the top level task.".into())
            .assignees(vec!["Heinz".into(), "Kunigunde".into(), "Andreas".into()])
            .children(vec![
                        TaskBuilder::default()
                            .title("First Item".into())
                            .description("This is the first sub task.".into())
                            .assignees(vec!["Heinz".into(), "Kunigunde".into()])
                            .children(vec![
                                TaskBuilder::default()
                                    .title("Second Level".into())
                                    .description("This is a sub sub task.".into())
                                    .assignees(vec!["Heinz".into(), "Kunigunde".into()])
                                    .children(vec![ ])
                                    .build()
                                    .unwrap(),
                            ])
                            .build()
                            .unwrap(),
                    ])
            .build()
            .unwrap();
    rouille::Response::json(&task)
}
pub fn post_task(req: &Request) -> Response {
    let task: Task = try_or_400!(json_input(req));
    println!("Got task: {:#?}", task);
    rouille::Response::empty_204()
}

pub fn setup() {
    let _user_1 = DB.user_mut().insert(User::new("Test User"));
    let _user_2 = DB.user_mut().insert(User::new("Alter Schwede"));
    let _task_1 = DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = DB.task_mut().insert(Task::new("NSA hacken"));
}
