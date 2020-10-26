use crate::shared::subscriptions::PriceSubscription;
use rql::*;
use lazy_static::lazy_static;
use database_table::{
    Database,
};
use app_model::user::User;

schema! {
    pub Schema {
        user: User,
        subscription: PriceSubscription,
    }
}

lazy_static! {
    pub static ref DB: Schema = Schema::new("binance_bot_database", rql::BinaryStable).unwrap();
}
impl<'db> Database<'db, User> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.user_mut()
    }
}
impl<'db> Database<'db, PriceSubscription> for Schema {
    fn table() -> TableGuard<'db, PriceSubscription> {
        DB.subscription()
    }
    fn table_mut() -> TableGuardMut<'db, PriceSubscription> {
        DB.subscription_mut()
    }
}
