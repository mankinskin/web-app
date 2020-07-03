use rql::*;
use plans::{
    user::*,
    task::*,
    project::*,
};
use crate::{
    entry::*,
};
use updatable::{
    *,
};

schema! {
    pub Schema {
        user: User,
        task: Task,
        project: Project,
    }
}

lazy_static!{
    pub static ref DB: Schema = Schema::new("test_database", rql::HumanReadable).unwrap();
}
pub trait DatabaseTable<'a>
    : Sized
    + Clone
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
    + Updatable
    + 'a
{
    fn table() -> TableGuard<'a, Self>;
    fn table_mut() -> TableGuardMut<'a, Self>;

    fn insert(obj: Self) -> Id<Self> {
        Self::table_mut()
            .insert(obj)
    }
    fn get(id: Id<Self>) -> Option<Entry<Self>> {
        Self::table()
            .get(id)
            .map(|data| Entry::from((id, data.clone())))
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
    fn get_list(ids: Vec<Id<Self>>) -> Vec<Entry<Self>> {
        ids.iter()
            .filter_map(|id|
                 Self::get(*id)
            )
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

impl<'a> DatabaseTable<'a> for User {
    fn table() -> TableGuard<'a, Self> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.user_mut()
    }
}

impl<'a> DatabaseTable<'a> for Task {
    fn table() -> TableGuard<'a, Self> {
        DB.task()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.task_mut()
    }
}

impl<'a> DatabaseTable<'a> for Project {
    fn table() -> TableGuard<'a, Self> {
        DB.project()
    }
    fn table_mut() -> TableGuardMut<'a, Self> {
        DB.project_mut()
    }
}
