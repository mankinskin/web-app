
#[derive(Clone)]
pub enum Status<T>
    where T: Clone,
{
    Empty,
    Loading,
    Loaded(T)
}
