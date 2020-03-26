#[macro_use] extern crate rouille;
extern crate chrono;
extern crate colored;

use rouille::{
    Request,
    Response,
    ResponseBody,
};
use chrono::{
    Utc,
};
use std::{
    io::{
        Read,
    },
    fs::{
        File,
    },
};
use colored::*;


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
        (GET) (/) => {
            match File::open("./index.html") {
                Ok(file) => Response::from_file("text/html", file),
                Err(e) => Response::text(e.to_string()),
            }
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
