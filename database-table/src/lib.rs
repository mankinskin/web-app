pub mod entry;
pub use entry::*;
pub mod table;
pub use table::*;

use rql::*;

pub trait Database<'db, T: DatabaseTable<'db, Self>> : Sized {
    fn table() -> TableGuard<'db, T>;
    fn table_mut() -> TableGuardMut<'db, T>;
    fn insert(obj: T) -> Id<T> {
        Self::table_mut()
            .insert(obj)
    }
    fn get(id: Id<T>) -> Option<Entry<T>> {
        Self::table()
            .get(id)
            .map(|data| Entry::from((id, data.clone())))
    }
    fn delete(id: Id<T>) -> Option<T> {
        Self::table_mut()
          .delete_one(id)
    }
    fn get_all() -> Vec<Entry<T>> {
        Self::table()
            .rows()
            .map(|row| row.into())
            .collect()
    }
    fn get_list(ids: Vec<Id<T>>) -> Vec<Entry<T>> {
        ids.iter()
            .filter_map(|id|
                 Self::get(*id)
            )
            .collect()
    }
    fn filter<F>(f: F) -> Vec<Entry<T>>
        where F: Fn(&T) -> bool
    {
        Self::table()
            .wher(|row| f(row.data))
            .map(|row| row.into())
            .collect()
    }
    fn find<F>(f: F) -> Option<Entry<T>>
        where F: Fn(&T) -> bool
    {
        Self::table()
            .find(|row| f(row.data))
            .map(|row| row.into())
    }
}
