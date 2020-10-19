use app_model::user::User;
use crate::shared::subscription::PriceSubscription;
use rql::*;
use lazy_static::lazy_static;

schema! {
    pub Schema {
        user: User,
        subscription: PriceSubscription,
    }
}

lazy_static! {
    pub static ref DB: Schema = Schema::new("binance_bot_database", rql::BinaryStable).unwrap();
}
