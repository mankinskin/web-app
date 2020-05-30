use rql::*;
use plans::{
    user::*,
    note::*,
    task::*,
};
use crate::{
    entry::*,
};
use updatable::{
    *,
};

pub trait DatabaseTable<'a> : Sized + Clone + serde::Serialize + Updatable + 'a {
    fn table() -> TableGuard<'a, Self>;
    fn table_mut() -> TableGuardMut<'a, Self>;

    fn insert(obj: Self) -> Id<Self> {
        Self::table_mut()
            .insert(obj)
    }
    fn get(id: Id<Self>) -> Option<Self> {
        Self::table()
            .get(id)
            .map(|entry| entry.clone())
    }
    fn delete(id: Id<Self>) -> Option<Self> {
        Self::table_mut()
          .delete_one(id)
    }
    fn update(id: Id<Self>, update: <Self as Updatable>::Update) -> Option<Self> {
        Self::table_mut()
          .get_mut(id)
          .map(move |entry| {
              update.update(entry);
              entry.clone()
          })
    }
    fn get_all() -> Vec<Entry<Self>> {
        Self::table()
            .rows()
            .map(|row| row.into())
            .collect()
    }
    fn filter<F>(f: F) -> Vec<Entry<Self>>
        where F: Fn(&Self) -> bool
    {
        Self::table()
            .wher(|row| f(row.data))
            .map(|row| row.into())
            .collect()
    }
    fn find<F>(f: F) -> Option<Entry<Self>>
        where F: Fn(&Self) -> bool
    {
        Self::table()
            .find(|row| f(row.data))
            .map(|row| row.into())
    }
}
impl<'a> DatabaseTable<'a> for Note {
    fn table() -> TableGuard<'a, Self> {
        crate::DB.note()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        crate::DB.note_mut()
    }
}
impl<'a> DatabaseTable<'a> for User {
    fn table() -> TableGuard<'a, Self> {
        crate::DB.user()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        crate::DB.user_mut()
    }
}
impl<'a> DatabaseTable<'a> for Task {
    fn table() -> TableGuard<'a, Self> {
        crate::DB.task()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        crate::DB.task_mut()
    }
}

