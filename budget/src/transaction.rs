use crate::purpose::{
    Purpose,
    Purposes,
};
use crate::subject::{
    Subject,
};
use crate::currency::{
    Currency,
};
use ::chrono::{
    DateTime,
    Utc,
};
use tabular::{row, Row};
use crate::interpreter::parse::*;
use crate::currency::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction<C: Currency> {
    pub amount: C,
    pub purposes: Option<Purposes>,
    pub sender: Subject,
    pub recipient: Option<Subject>,
    pub date: Option<DateTime<Utc>>,
}

#[cfg(target_arch="wasm32")]
fn get_time_now() -> DateTime<Utc> {
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
            sender: Subject::Me,
            recipient: None,
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
    pub fn get_amount(&self)
        -> C  {
            self.amount.clone()
    }
    pub fn set_amount<Amt: Into<C>>(&mut self, amt: Amt)
        -> &mut Self {
            self.amount = amt.into();
            self
        }
    pub fn get_date(&self)
        -> Option<DateTime<Utc>> {
            self.date.clone()
    }
    pub fn set_date(&mut self, date: DateTime<Utc>)
        -> &mut Self {
            self.date = Some(date);
            self
        }
    pub fn get_sender(&self)
        -> Subject {
            self.sender.clone()
    }
    pub fn set_sender<S: Into<Subject>>(&mut self, subject: S)
        -> &mut Self {
            self.sender = subject.into();
            self
        }
    pub fn get_recipient(&self)
        -> Option<Subject> {
        self.recipient.clone()
    }
    pub fn set_recipient<S: Into<Subject>>(&mut self, subject: S)
        -> &mut Self {
            self.recipient = Some(subject.into());
            self
        }
    pub fn get_purposes(&self)
        -> Option<Purposes> {
        self.purposes.clone()
    }
    pub fn set_purposes<P: Into<Purpose> + Clone>(&mut self, purposes: Vec<P>)
        -> &mut Self {
            let ps = Purposes::from(
                purposes.iter()
                        .map(|p| p.clone().into())
                        .collect::<Vec<Purpose>>()
                        );
            self.purposes = Some(ps);
            self
        }
    pub fn add_purpose<P: Into<Purpose>>(&mut self, purpose: P)
        -> &mut Self {
            if let Some(ps) = &mut self.purposes {
                ps.push(purpose.into());
            } else {
                self.purposes = Some(Purposes::from(vec![purpose.into()]));
            }
            self
        }
}

impl<C: Currency> Into<Row> for Transaction<C> {
    fn into(self) -> Row {
        row!(
            self.get_date().map(|d| d.to_string()).unwrap_or("None".into()),
            self.get_amount().to_string(),
            self.get_sender().to_string(),
            self.get_recipient().map(|s| s.to_string()).unwrap_or("None".into()),
            self.get_purposes().map(|ps| ps.to_string()).unwrap_or("None".into())
            )
    }
}
impl<'a> Parse<'a> for Transaction<Euro> {
    named!(parse(&'a str) -> Self,
    map!(
        tuple!(
            // (Date): Today, 3rd of November, ..
            opt!(
                terminated!(
                    DateTime::<Utc>::parse,
                    space1
                )
            ),
            // <Subject>: I | Name
            preceded!(
                space0,
                Subject::parse
            ),
            // <Action>: got, gave, get, give, ...
            preceded!(
                space1,
                Action::parse
                ),
            // <Object>: 10 euros, 1€,
            preceded!(
                space1,
                Euro::parse
                ),
            // (to <Recipient>): Me | Name
            opt!(
                preceded!(
                    space1,
                    preceded!(
                        terminated!(
                            alt!(
                                tag_no_case!("to") |
                                tag_no_case!("from")
                            ),
                            space1
                        ),
                        Subject::parse
                        )
                )
                ),
            // (for <Purpose>)
            opt!(
                preceded!(
                    delimited!(
                        space1,
                        tag_no_case!("for"),
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
                    t.set_sender(sender);
                    match recipient {
                        Some(subject) => {t.set_recipient(subject);},
                        None => {}
                    };
                    match purpose {
                        Some(p) => {t.add_purpose(p);},
                        None => {}
                    };
                    t
        })
        );
}
mod tests {
    #[allow(unused)]
    use super::*;
    #[allow(unused)]
    use crate::subject::*;

    // TODO
    // - Accept transaction declarations
    //  - <Time><Kind><Currency>(Purpose*)(Person)
    //  - (<Date>) (at <Time>) I got/gave <Currency> for <Purpose>* from/to <Person>.
    //    where
    //     Date <- { Today, On <Date>, },
    //     Time <- { at , On <Date>, },

    #[test]
    fn basic() {
        let parsed = Transaction::parse("Today I gave 5€").unwrap().1;
        assert!(parsed ==
            Transaction {
                amount: Euro::from(-5),
                date: parsed.date,
                sender: Subject::Me,
                recipient: None,
                purposes: None,
            }
        );
    }
    #[test]
    fn with_recipient() {
        let parsed = Transaction::parse("Today I gave 5€ to Recipient").unwrap().1;
        assert!(parsed  ==
            Transaction {
                amount: Euro::from(-5),
                date: parsed.date,
                sender: Subject::Me,
                recipient: Some(Subject::from("Recipient")),
                purposes: None,
            }
        );
    }
}
