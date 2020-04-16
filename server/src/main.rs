#[macro_use] extern crate rouille;
extern crate chrono;
extern crate colored;
extern crate serde_json;
extern crate plans;

use rouille::{
    Request,
    Response,
    input::json::*,
};
use chrono::{
    Utc,
};
use std::{
    fs::{
        File,
    },
    path::{
        Path,
    },
};
use colored::*;
use plans::{
    user::*,
    note::*,
    task::*,
};


fn log_request(r: &Request) {
    print!("[{}] {} {} {} ",
        Utc::now().format("%d.%m.%Y %T"),
        r.remote_addr().to_string().blue(),
        r.method(),
        r.raw_url());
}
fn log_response(r: &Response) {
    println!("{}", (|s: String| if r.is_error() {
        s.red()
    } else {
        s.green()
    })(r.status_code.to_string()));
}
fn get_html<P: AsRef<Path>>(path: P) -> Response {
    match File::open(path) {
        Ok(file) => Response::from_file("text/html", file),
        Err(e) => Response::text(e.to_string()),
    }
}
fn get_user(req: &Request) -> Response {
    let user = User::new("Server");
    rouille::Response::json(&user)
}
fn post_note(req: &Request) -> Response {
    let note: Note = try_or_400!(json_input(req));
    println!("Got note: {:#?}", note);
    rouille::Response::empty_204()
}
fn get_task(req: &Request) -> Response {
    let task = Task {
                    title: "Server Task".into(),
                    description: "This is the top level task.".into(),
                    assignees: vec!["Heinz".into(), "Kunigunde".into(), "Andreas".into()],
                    children: vec![
                            Task {
                                title: "First Item".into(),
                                description: "This is the first sub task.".into(),
                                assignees: vec!["Heinz".into(), "Kunigunde".into()],
                                children: vec![
                                    Task {
                                        title: "Second Level".into(),
                                        description: "This is a sub task of a sub task.".into(),
                                        assignees: vec!["Heinz".into(), "Kunigunde".into()],
                                        children: Vec::new(),
                                    },
                                ],
                            },
                            Task {
                                title: "Another Sub Task".into(),
                                description: "This sub task has many children.".into(),
                                assignees: vec!["Günter".into(), "Siegbert".into(), "Manfred".into(), "Georg".into()],
                                children: vec![
                                        Task {
                                            title: "Task 1.2.1".into(),
                                            description: "Child 1.".into(),
                                            assignees: vec!["Günter".into()],
                                            children: Vec::new(),
                                        },
                                        Task {
                                            title: "Task 1.2.2".into(),
                                            description: "Child 2.".into(),
                                            assignees: vec!["Siegbert".into()],
                                            children: Vec::new(),
                                        },
                                        Task {
                                            title: "Task 1.2.3".into(),
                                            description: "Child 3.".into(),
                                            assignees: vec!["Manfred".into(), "Georg".into()],
                                            children: Vec::new(),
                                        },
                                ],
                            },
                        ],
                    };
    rouille::Response::json(&task)
}

fn handle_request(request: &Request) -> Response {
    log_request(request);
    let response = router!(request,
        (GET) (/tasks/tools) => {
            get_html("./tasks/index.html")
        },
        (GET) (/tasks) => {
            get_html("./tasks/index.html")
        },
        (GET) (/budget) => {
            get_html("./home/index.html")
        },
        (GET) (/api/task) => {
            get_task(request)
        },
        (GET) (/api/user) => {
            get_user(request)
        },
        (POST) (/api/note) => {
            post_note(request)
        },
        (GET) (/user) => {
            get_html("./home/index.html")
        },
        (GET) (/profile) => {
            get_html("./home/index.html")
        },
        (GET) (/note) => {
            get_html("./home/index.html")
        },
        (GET) (/) => {
            get_html("./home/index.html")
        },
        _ => rouille::match_assets(request, "./")
    );
    log_response(&response);
    response
}
fn main() {
    let address = "0.0.0.0:8000";
    println!("Serving on {}", address);
    rouille::start_server(address, handle_request)
}
