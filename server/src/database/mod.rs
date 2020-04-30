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
pub fn get_note(req: &Request) -> Response {
    match req.data() {
        None => rouille::Response::empty_400(),
        Some(body) => {
            match serde_json::from_reader(body) {
                Err(e) => {
                    rouille::Response::text(e.to_string())
                        .with_status_code(500)
                }
                Ok(id) => {
                    let note = DB.note().get(id).unwrap().clone();
                    rouille::Response::json(&note)
                }
            }
        },
    }
}
pub fn post_user(req: &Request) -> Response {
    //let user = User::new("Server");
    let user: User = try_or_400!(json_input(req));
    println!("Got user: {:#?}", user);
    let user_id = DB.user_mut().insert(user);
    rouille::Response::json(&user_id)
}
pub fn get_user(req: &Request) -> Response {
    match req.data() {
        None => rouille::Response::empty_400(),
        Some(body) => {
            match serde_json::from_reader(body) {
                Err(e) => {
                    rouille::Response::text(e.to_string())
                        .with_status_code(500)
                }
                Ok(id) => {
                    let user = DB.user().get(id).unwrap().clone();
                    rouille::Response::json(&user)
                }
            }
        },
    }
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
    //                        .children(vec![
    //                            TaskBuilder::default()
    //                                .title("Second Level".into())
    //                                .description("This is a sub sub task.".into())
    //                                .assignees(vec!["Heinz".into(), "Kunigunde".into()])
    //                                .children(vec![ ])
    //                                .build()
    //                                .unwrap(),
    //                        ])
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
pub fn get_task(req: &Request) -> Response {
    match req.data() {
        None => rouille::Response::empty_400(),
        Some(body) => {
            let task = Task::new("Server Task");//DB.task().get(id).unwrap().clone();
            let json = serde_json::to_string(&task).unwrap();
            println!("Tasks: {:#?}", json);
            let mut response = rouille::Response::text(json.clone());
            response.data = rouille::ResponseBody::from_string(json);
            println!("Response: {:#?}", response);
            response
            //match serde_json::from_reader(body) as std::result::Result<Id<Task>, serde_json::Error> {
            //    Ok(id) => {
            //        let task = Task::new("Server Task");//DB.task().get(id).unwrap().clone();
            //        rouille::Response::json(&task)
            //    },
            //    Err(e) => {
            //        rouille::Response::text(e.to_string())
            //            .with_status_code(500)
            //    },
            //}
        },
    }
}
pub fn get_tasks(_req: &Request) -> Response {
    let tasks: Vec<Task> = DB.task().rows().map(|row| row.data.clone()).collect();
    //let json = String::from("Hello world");
    let json = serde_json::to_string(&tasks).unwrap();
    println!("Tasks: {:#?}", json);
    let mut response = rouille::Response::text(json.clone());
    response.data = rouille::ResponseBody::from_string(json);
    println!("Response: {:#?}", response);
    response
}

pub fn setup() {
    let _user_1 = DB.user_mut().insert(User::new("Test User"));
    let _user_2 = DB.user_mut().insert(User::new("Alter Schwede"));
    let _task_1 = DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = DB.task_mut().insert(Task::new("NSA hacken"));
}
