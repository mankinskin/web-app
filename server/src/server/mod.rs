use rouille::{
    Request,
    Response,
};
use chrono::{
    Utc,
};
use colored::*;
use crate::{
    database::*,
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
        (GET) (/api/tasks) => {
            get_tasks(request)
        },
        (GET) (/api/task) => {
            get_task(request)
        },
        (POST) (/api/task) => {
            post_task(request)
        },
        (GET) (/api/user) => {
            get_user(request)
        },
        (POST) (/api/user) => {
            post_user(request)
        },
        (GET) (/api/note) => {
            get_note(request)
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

pub fn start() {
    let address = "0.0.0.0:8000";
    println!("Serving on {}", address);
    rouille::start_server(address, handle_request)
}
