#[macro_use] extern crate lazy_static;
extern crate serde_json;
extern crate serde;
extern crate rql;
extern crate updatable;
extern crate plans;

pub mod entry;
pub use entry::*;
pub mod table;
pub use table::*;

use plans::{
    user::*,
    note::*,
    task::*,
};
use rql::*;

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

pub fn setup() {
    let test_user = crate::DB.user_mut().insert(User::new("test_user", "test_password"));
    println!("Test user ID: {}", test_user);
    let _user_2 = crate::DB.user_mut().insert(User::new("Alter Schwede", "test_password"));
    let _task_1 = crate::DB.task_mut().insert(Task::new("Aufgabe Test"));
    let _task_2 = crate::DB.task_mut().insert(Task::new("NSA hacken"));
}
