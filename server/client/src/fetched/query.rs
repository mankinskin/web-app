use rql::{
    Id,
};
use serde::{
    Deserialize,
};

#[derive(Debug, Clone, Deserialize)]
pub enum Query<T>
{
    Id(Id<T>),
    All,
}
impl<T> Query<T>
{
    pub fn id(id: Id<T>) -> Self {
        Self::Id(id)
    }
    pub fn all() -> Self {
        Self::All
    }
}
impl<T> ToString for Query<T>
{
    fn to_string(&self) -> String {
        match self {
            Query::Id(id) => id.to_string(),
            Query::All => "".to_string(),
        }
    }
}
