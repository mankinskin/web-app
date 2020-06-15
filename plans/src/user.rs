use crate::{
    credentials::*,
};
use updatable::{
    *,
};
use rql::{
    *,
};
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Updatable,
    )]
pub struct User {
    credentials: Credentials,
}
impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self {
            credentials,
        }
    }
}
use std::fmt::{self, Display};
impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.credentials.username)
    }
}
impl User {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn new<S1: ToString, S2: ToString>(name: S1, password: S2) -> Self {
        Self {
            credentials:
                Credentials {
                    username: name.to_string(),
                    password: password.to_string()
                },
        }
    }
    pub fn name(&self) -> &String {
        &self.credentials.username
    }
    pub fn password(&self) -> &String {
        &self.credentials.password
    }
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
    pub fn credentials_mut(&mut self) -> &mut Credentials {
        &mut self.credentials
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
