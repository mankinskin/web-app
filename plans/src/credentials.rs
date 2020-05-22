use updatable::{
    *,
};
#[derive(Clone, Debug, Updatable)]
pub struct Credentials {
    username: String,
    password: String,
}
impl Credentials {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
        }
    }
}
