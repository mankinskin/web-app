use crate::{auth::credentials::*, DB};
use database_table::DatabaseTable;
use rql::*;
use updatable::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Updatable)]
pub struct User {
    credentials: Credentials,
    full_name: Option<String>,
    followers: Vec<Id<User>>,
}
impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self {
            credentials,
            full_name: None,
            followers: vec![],
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
            credentials: Credentials::new(name, password),
            full_name: None,
            followers: vec![],
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
    pub fn followers(&self) -> &Vec<Id<User>> {
        &self.followers
    }
    pub fn full_name(&self) -> &Option<String> {
        &self.full_name
    }
}
impl<'a> DatabaseTable<'a> for User {
    fn table() -> TableGuard<'a, Self> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.user_mut()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, Updatable, PartialEq)]
pub struct UserProfile {
    user: User,
}
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self { user }
    }
}
