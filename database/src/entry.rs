use rql::*;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Entry<T>(Id<T>, T);

impl<T> Entry<T> {
    pub fn id(&self) -> &Id<T> {
        &self.0
    }
    pub fn data(&self) -> &T {
        &self.1
    }
}

impl<T> From<Row<'_, T>> for Entry<T>
    where T: Clone
{
    fn from(row: Row<'_, T>) -> Self {
        Self(row.id, (*row.data).clone())
    }
}
