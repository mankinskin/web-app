use rql::*;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Entry<T> {
    pub id: Id<T>,
    pub data: T,
}
impl<T> Entry<T> {
    pub fn new(id: Id<T>, data: T) -> Self {
        Self {
            id,
            data,
        }
    }
    pub fn id(&self) -> &Id<T> {
        &self.id
    }
    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
impl<T> From<Row<'_, T>> for Entry<T>
    where T: Clone
{
    fn from(row: Row<'_, T>) -> Self {
        Self::new(row.id, (*row.data).clone())
    }
}
impl<T> From<Id<T>> for Entry<T>
    where T: Clone + Default
{
    fn from(id: Id<T>) -> Self {
        Self::from((id, T::default()))
    }
}
impl<T> From<(Id<T>, T)> for Entry<T>
    where T: Clone
{
    fn from((id, data): (Id<T>, T)) -> Self {
        Self::new(id, data)
    }
}

