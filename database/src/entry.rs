use rql::*;
use updatable::*;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Entry<T>(Id<T>, T);

impl<T> Entry<T> {
    pub fn new(id: Id<T>, data: T) -> Self {
        Self(id, data)
    }
    pub fn id(&self) -> &Id<T> {
        &self.0
    }
    pub fn data(&self) -> &T {
        &self.1
    }
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.1
    }
}
impl<T> Updatable for Entry<T>
    where T: Updatable
{
     type Update = <T as Updatable>::Update;
}
impl<T> Update<Entry<T>> for <T as Updatable>::Update
    where T: Updatable
{
    fn update(&self, data: &mut Entry<T>) {
        self.update(data.data_mut());
    }
}

impl<T> From<Row<'_, T>> for Entry<T>
    where T: Clone
{
    fn from(row: Row<'_, T>) -> Self {
        Self(row.id, (*row.data).clone())
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
        Self(id, data)
    }
}
