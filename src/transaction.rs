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
    pub fn set_amount<Amt: Into<C>>(&mut self, amt: Amt)
        -> &mut Self {
            self.amount = amt.into();
            self
        }
    pub fn set_date(&mut self, date: DateTime<Utc>)
        -> &mut Self {
            self.date = date;
            self
        }
    pub fn set_partner<P: Into<Person>>(&mut self, partner: P)
        -> &mut Self {
            self.partner = Some(partner.into());
            self
        }
    pub fn set_purposes<P: Into<Purpose> + Clone>(&mut self, purposes: Vec<P>)
        -> &mut Self {
            let ps: Vec<Purpose> = purposes.iter()
                .map(|p| p.clone().into()).collect();
            self.purposes = Some(ps);
            self
        }
    pub fn set_purpose<P: Into<Purpose>>(&mut self, purpose: P)
        -> &mut Self {
            let mut ps = self.purposes.clone().unwrap_or(Vec::new());
            ps.push(purpose.into());
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
