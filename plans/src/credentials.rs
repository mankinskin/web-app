use updatable::{
    *,
};
#[derive(Clone, Debug, Updatable, Default)]
pub struct Credentials {
    username: String,
    password: String,
}
impl Credentials {
    pub fn new() -> Self {
        Self::default()
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
            _ => String::from(
                "Username must be between 8 and 16 characters long."
            )
        }
    }
    pub fn password_invalid_text(&self) -> String {
        match self.password.len() {
            0 | 8..=16 => String::new(),
            _ => String::from(
                "Password must be between 8 and 16 characters long."
            )
        }
    }
}
