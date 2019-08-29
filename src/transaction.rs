use crate::purpose::{
    Purpose,
};
use crate::actor::{
    Actor,
};
use crate::currency::{
    Currency,
};
use ::chrono::{
    DateTime,
    Utc,
};

#[derive(Clone, Debug)]
pub struct Transaction<C: Currency> {
    pub amount: C,
    pub purposes: Option<Vec<Purpose>>,
    pub partner: Option<Actor>,
    pub date: DateTime<Utc>,
}

impl<C: Currency> Transaction<C> {
    pub fn get(amount: C) -> Self {
        assert!(amount != C::from(0));
        Self {
            amount,
            partner: None,
            purposes: None,
            date: Utc::now(),
        }
    }
    pub fn give(amount: C) -> Self {
        assert!(amount != C::from(0));
        Self {
            amount: -amount,
            partner: None,
            purposes: None,
            date: Utc::now(),
        }
    }
    pub fn set_partner(&mut self, partner: Actor)
        -> &mut Self {
            self.partner = Some(partner);
            self
        }
    pub fn set_purposes(&mut self, purposes: Vec<Purpose>)
        -> &mut Self {
            self.purposes = Some(purposes);
            self
        }
    pub fn set_purpose(&mut self, purpose: Purpose)
        -> &mut Self {
            let mut ps = self.purposes.clone().unwrap_or(Vec::new());
            ps.push(purpose);
            self.purposes = Some(ps);
            self
        }
}
use tabular::{row, Row};
impl<C: Currency> Into<Row> for Transaction<C> {
    fn into(self) -> Row {
        row!(self.date.format("%d.%m.%Y %H:%M"),
             self.amount,
             self.partner.map(|a| format!("{}", a)).unwrap_or("None".into()),
             self.purposes
             .map(|ps| format!("{}",
                               ps.iter().fold(String::new(),
                                |acc, x| format!("{}{}, ", acc, x))))
                  .unwrap_or("None".into()))
    }
}
