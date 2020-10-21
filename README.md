# database-table

Utility crate for generic access to schemas defined using [rql](https://github.com/kaikalii/rql).

```rust
schema! {
    pub Schema {
        user: User,
        subscription: Subscription,
    }
}

lazy_static! {
    pub static ref DB: Schema = Schema::new("example_database", rql::BinaryStable).unwrap();
}
impl<'db> Database<'db, User> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.user_mut()
    }
}
impl<'db> Database<'db, Subscription> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.subscription()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.subscription_mut()
    }
}
```
```
// login into any Database for Users
pub async fn login<'db, D: Database<'db, User>>(credentials: Credentials) -> Result<UserSession, Error> {
    DatabaseTable::<'db, D>::find(|user| *user.name() == credentials.username)
        .ok_or(ErrorNotFound("User not found"))
        .and_then(|entry| {
            let user = entry.data();
            ...
        })
        ...
}

```
