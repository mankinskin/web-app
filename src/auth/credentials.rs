use crate::user::*;
use updatable::*;

#[derive(Clone, Debug, Default, PartialEq, Updatable, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
impl Credentials {
    pub fn new<S1: ToString, S2: ToString>(name: S1, password: S2) -> Self {
        Self {
            username: name.to_string(),
            password: password.to_string(),
        }
    }
    pub fn username_is_valid(&self) -> bool {
        self.username_invalid_text().is_empty()
    }
    pub fn password_is_valid(&self) -> bool {
        self.password_invalid_text().is_empty()
    }
    pub fn username_invalid_text(&self) -> String {
        match self.username.len() {
            0 | 8..=16 => String::new(),
            _ => String::from("Username must be between 8 and 16 characters long."),
        }
    }
    pub fn password_invalid_text(&self) -> String {
        match self.password.len() {
            0 | 8..=16 => String::new(),
            _ => String::from("Password must be between 8 and 16 characters long."),
        }
    }
}
impl From<&User> for Credentials {
    fn from(user: &User) -> Self {
        Self {
            username: user.name().clone(),
            password: user.password().clone(),
        }
    }
}
impl From<User> for Credentials {
    fn from(user: User) -> Self {
        Self {
            username: user.name().clone(),
            password: user.password().clone(),
        }
    }
}
