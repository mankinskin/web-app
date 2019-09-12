use crate::purpose::{
    Purpose,
};
use crate::person::{
    Person,
};
use crate::currency::{
    Currency,
};
use ::chrono::{
    DateTime,
    Utc,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction<C: Currency> {
    pub amount: C,
    pub purposes: Option<Vec<Purpose>>,
    pub partner: Option<Person>,
    pub date: DateTime<Utc>,
}

impl<C: Currency> Default for Transaction<C> {
    fn default() -> Self {
        Transaction {
            amount: C::from(0),
            partner: None,
            purposes: None,
            date: Utc::now(),
        }
    }
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
    pub fn set_amount(&mut self, amt: C)
        -> &mut Self {
            self.amount = amt;
            self
        }
    pub fn set_date(&mut self, date: DateTime<Utc>)
        -> &mut Self {
            self.date = date;
            self
        }
    pub fn set_partner(&mut self, partner: Person)
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
