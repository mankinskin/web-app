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
    pub date: Option<DateTime<Utc>>,
}
#[cfg(target_arch="wasm32")]
fn get_time_now() -> DateTime<Utc> {
    use stdweb::*;
    let timestamp = stdweb::web::Date::now();
    let secs: i64 = (timestamp/1000.0).floor() as i64;
    let nanoes: u32 = (timestamp as u32%1000)*1_000_000;
    let naivetime = chrono::NaiveDateTime::from_timestamp(secs, nanoes);
    chrono::DateTime::<Utc>::from_utc(naivetime, Utc)
}
#[cfg(not(target_arch="wasm32"))]
fn get_time_now() -> DateTime<Utc> {
    chrono::Utc::now()
}
impl<C: Currency> Default for Transaction<C> {
    fn default() -> Self {
        let datetime = get_time_now();
        Transaction {
            amount: C::from(0),
            partner: None,
            purposes: None,
            date: Some(datetime),
        }
    }
}

impl<C: Currency> Transaction<C> {
    pub fn get<Amt: Into<C>>(amount: Amt) -> Self {
        let amt = amount.into();
        assert!(amt != C::from(0));
        Self {
            amount: amt,
            ..Self::default()
        }
    }
    pub fn give<Amt: Into<C>>(amount: Amt) -> Self {
        let amt = amount.into();
        assert!(amt != C::from(0));
        Self {
            amount: -amt,
            ..Self::default()
        }
    }
    pub fn set_amount<Amt: Into<C>>(&mut self, amt: Amt)
        -> &mut Self {
            self.amount = amt.into();
            self
        }
    pub fn set_date(&mut self, date: DateTime<Utc>)
        -> &mut Self {
            self.date = Some(date);
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
    pub fn get_date_string(&self) -> String {
        match self.date {
            Some(d) => d.format("%d.%m.%Y %H:%M").to_string(),
            None => "unknown".into(),
        }
    }
    pub fn get_amount_string(&self) -> String {
        format!("{}", self.amount)
    }
    pub fn get_partner_string(&self) -> String {
        self.partner.clone().map(|a| format!("{}", a)).unwrap_or("None".into())
    }
    pub fn get_purpose_string(&self) -> String {
        self.purposes.clone()
             .map(|ps| format!("{}",
                               ps.iter().fold(String::new(),
                                |acc, x| format!("{}{}, ", acc, x.to_string()))))
                  .unwrap_or("None".into())
    }
}

use tabular::{row, Row};
impl<C: Currency> Into<Row> for Transaction<C> {
    fn into(self) -> Row {
        row!(self.get_date_string(),
             self.get_amount_string(),
             self.get_partner_string(),
             self.get_purpose_string()
             )
    }
}
use crate::interpreter::parse::*;
use crate::currency::*;
use crate::person::*;

impl<'a> Parse<'a> for Transaction<Euro> {
    named!(parse(&'a str) -> Self,
    map!(
        tuple!(
            // (Date)
            opt!(
                preceded!(
                    space0,
                    DateTime::<Utc>::parse
                    )
                ),
            // Sender
            preceded!(
                space0,
                Subject::parse
                ),
            // Action
            preceded!(
                space1,
                Action::parse
                ),
            // Currency
            preceded!(
                space1,
                Euro::parse
                ),
            // (Recipient)
            opt!(
                preceded!(
                    delimited!(
                        space1,
                        tag!("to"),
                        space1
                    ),
                    Subject::parse
                    )
                ),
            // (Purpose)
            opt!(
                preceded!(
                    delimited!(
                        space1,
                        tag!("for"),
                        space1
                    ),
                    Purpose::parse
                    )
                )
            ),
        |(date, sender, action, amount, recipient, purpose)| {
                    let mut t = Transaction::default();
                    match date {
                        Some(d) => {t.set_date(d);},
                        None => {},
                    };
                    t.set_amount(match action {
                        Action::Get => amount,
                        Action::Give => -amount
                    });
                    match recipient {
                        Some(Subject::Person(p)) => {t.set_partner(p);},
                        Some(Subject::Me) => {},
                        None => {}
                    };
                    match purpose {
                        Some(p) => {t.set_purpose(p);},
                        None => {}
                    };
                    t
        })
        );
}
