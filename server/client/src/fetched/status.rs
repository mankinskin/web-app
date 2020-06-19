use seed::{
    fetch::{
        FetchError,
    },
    prelude::{
        JsValue,
    },
};
#[derive(Clone)]
pub enum Status<T> {
    Empty,
    Waiting,
    Ready(T),
    Failed(String),
}
impl<T> Default for Status<T> {
    fn default() -> Self {
        Self::Empty
    }
}
impl<T> Status<T> {
    pub fn get_ready(self) -> Result<T, FetchError> {
        if let Status::Ready(t) = self {
            Ok(t)
        } else {
            Err(FetchError::RequestError(JsValue::from_str("No data ready for POST/PUT.")))
        }
    }
}
