use updatable::{
    *,
};
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Updatable,
    PartialEq,
    )]
pub struct Note {
    text: String,
}

impl Note {
    pub fn new<S: ToString>(text: S) -> Self {
        Self {
            text: text.to_string(),
        }
    }
    pub fn set_text<S: ToString>(&mut self, text: S) {
        self.text = text.to_string();
    }
}
