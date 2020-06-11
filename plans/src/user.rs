use crate::credentials::{
    *,
};
use updatable::{
    *,
};
use rql::{
    *,
};
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Updatable,
    )]
pub struct User {
    name: String,
    password: String,
}
impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        User::new(credentials.username, credentials.password)
    }
}
use std::fmt::{self, Display};
impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl User {
    pub fn empty() -> Self {
        Self {
            name: String::default(),
            password: String::default(),
        }
    }
    pub fn new<S1: ToString, S2: ToString>(name: S1, password: S2) -> Self {
        Self {
            name: name.to_string(),
            password: password.to_string(),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn password(&self) -> &String {
        &self.password
    }
    pub fn credentials(&self) -> Credentials {
        Credentials::from(self)
    }
}
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Updatable,
    PartialEq,
    )]
pub struct UserProfile {
    user: User,
}
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            user,
        }
    }
}
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    )]
pub struct UserSession {
    pub user_id: Id<User>,
    pub token: String,
}
