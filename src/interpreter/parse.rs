#![allow(unused)]
use crate::currency::*;
use crate::transaction::*;
use crate::person::*;
use crate::purpose::*;
use ::chrono::*;
pub use nom::{
    *,
    character::{
        *,
        complete::*,
    },
    combinator::*,
    error::*,
};

pub trait Parse<'a> : Sized {
    fn parse(input: &'a str) -> IResult<&'a str, Self>;
}

impl<'a> Parse<'a> for Date<Utc> {
    named!(
        parse(&'a str) -> Self,
        alt!(
            tag!("Today") => { |_| Utc::today() } |
            tag!("Yesterday") => { |_| Utc::today() - Duration::days(1) } |
            tag!("Tomorrow") => { |_| Utc::today() + Duration::days(1) } |
            map_res!( // <d> days ago
            terminated!(
                digit1,
                preceded!(
                    space1,
                    tag!("days ago")
                )
                ),
                Units::parse
                ) => { |(_, d): (&str, Units)| Utc::today() - Duration::days(d.into()) } |
            map_res!( // in <d> days
            delimited!(
                terminated!(
                    tag!("in"),
                    space1
                ),
                digit1,
                preceded!(
                    space1,
                    tag!("days")
                )
                ),
                Units::parse
                ) => { |(_, d): (&str, Units)| Utc::today() + Duration::days(d.into()) }
            )
        );
}

impl<'a> Parse<'a> for NaiveTime {
    // parse <u32>:<u32>(:<u32>)
    named!(parse(&'a str) -> Self,
    map_res!(
        alt!(
            complete!(separated_pair!(
                    Units::parse,
                    char!(':'),
                    separated_pair!(
                        Units::parse,
                        char!(':'),
                        Units::parse
                        )
                    )) => {|(h, (m, s))| (h, m, s) } |
            separated_pair!(
                Units::parse,
                char!(':'),
                Units::parse
                ) => {|(h,m)| (h,m,0)}
            ),
            |(h, m, s)| {
                Ok(NaiveTime::from_hms(h as u32, m as u32, s as u32)) as Result<NaiveTime, (&[u8], ErrorKind)>
            })
    );
}
impl<'a> Parse<'a> for DateTime<Utc> {
    named!(
        parse(&'a str) -> Self,
        map!(
            tuple!(
                Date::parse,
                preceded!(
                    space0,
                    opt!(NaiveTime::parse)
                    )
                ),
                |(d, t)| match t {
                    Some(time) => d.and_time(time).unwrap(),
                    None => d.and_hms(0, 0, 0)
                }
            )
        );
}


impl<'a> Parse<'a> for Units {
    named!(parse(&'a str) -> Self,
    map_res!(
        digit1,
        |i| {
            Units::from_str_radix(i, 10)
                .map_err(|_e| (i, ErrorKind::ParseTo))
        })
    );
}

impl<'a> Parse<'a> for Euro {
    named!(parse(&'a str) -> Self,
    map!(
        alt!(
            preceded!(
                tag!("€"),
                Units::parse
                ) |
            terminated!(
                Units::parse,
                tag!("€")
                ) |
            complete!(terminated!(
                Units::parse,
                tag!(" Euros")
                )) |
            terminated!(
                Units::parse,
                tag!(" Euro")
                )
            ),
            |u| Euro::from(u)
        )
    );
}
impl<'a> Parse<'a> for Person {
    named!(
        parse(&'a str) -> Self,
        map!(
            alphanumeric1,
            |a| Person::from(a)
            )
        );
}

pub enum Subject {
    Me,
    Person(Person),
}
impl<'a> Parse<'a> for Subject {
    named!(
        parse(&'a str) -> Self,
        alt!(
            tag!("I") => { |_| Self::Me } |
            tag!("me") => { |_| Self::Me } |
            preceded!(space0,
                      Person::parse) => { |a| Self::Person(a) }
            )
        );
}
pub enum Action {
    Give,
    Get,
}

impl<'a> Parse<'a> for Action {
    named!(
        parse(&'a str) -> Self,
        alt!(
            tag!("have gaven") => { |_| Self::Give } |
            tag!("have gotten") => { |_| Self::Get } |
            tag!("gave") => { |_| Self::Give } |
            tag!("got") => { |_| Self::Get } |
            tag!("give") => { |_| Self::Give } |
            tag!("get") => { |_| Self::Get } |
            tag!("will give") => { |_| Self::Give } |
            tag!("will get") => { |_| Self::Get }
            )
        );
}

mod tests {
    // TODO
    // - Accept transaction declarations
    //  - <Time><Kind><Currency>(Purpose*)(Person)
    //  - (<Date>) (at <Time>) I got/gave <Currency> for <Purpose>* from/to <Person>.
    //    where
    //     Date <- { Today, On <Date>, },
    //     Time <- { at , On <Date>, },

    use super::*;
    use crate::cartesian;
    use crate::interpreter::*;
    #[test]
    fn parse_units() {
        for u in vec![7, 32, 1823, 0, 99999999] {
            assert_eq!(Units::parse(&format!("{}", u)).unwrap().1,
                u as Units);
        }
    }
    #[test]
    fn parse_euro() {
        cartesian!{
            ["{}€", "{} Euro", "€{}", "{} Euros"],
            [{1}, {32}, {1823}, {99999999}]
            ($fmt:tt {$u:expr}) => {
                assert_eq!(
                    Euro::parse(&format!($fmt, $u)).unwrap().1,
                    Euro::from($u)
                    );
            }
        }
    }
    #[test]
    fn parse_date() {
        assert_eq!(Date::parse("Today").unwrap().1,
        Utc::today());
        assert_eq!(Date::parse("Yesterday").unwrap().1,
        Utc::today() - Duration::days(1));
        assert_eq!(Date::parse("Tomorrow").unwrap().1,
        Utc::today() + Duration::days(1));
        for &d in &[1, 2, 3, 20, 100, 2134242] {
            assert_eq!(Date::parse(&format!("{} days ago", d)).unwrap().1,
            Utc::today() - Duration::days(d));
        }
        for &d in &[1, 2, 3, 20, 100, 2134242] {
            assert_eq!(Date::parse(&format!("in {} days", d)).unwrap().1,
            Utc::today() + Duration::days(d));
        }
    }
    #[test]
    fn parse_time() {
        for &(h, m) in &[
            (12, 0),
            (2, 1),
            (1, 12),
            (22, 0),
            (0, 30)] {
                assert_eq!(NaiveTime::parse(&format!("{}:{}", h, m)).unwrap().1,
                NaiveTime::from_hms(h, m, 0));
            }
        for &(h, m, s) in &[
            (12, 0, 12),
            (2, 1, 59),
            (1, 12, 27),
            (22, 0, 3),
            (0, 30, 0)] {
                assert_eq!(NaiveTime::parse(&format!("{}:{}:{}", h, m, s)).unwrap().1,
                NaiveTime::from_hms(h, m, s));
            }
    }
    mod transaction {
        use super::*;
        #[test]
        fn basic() {
            let parsed = Transaction::parse("Today I gave 5€").unwrap().1;
            assert!(parsed  ==
                    Transaction {
                        amount: Euro::from(-5),
                        date: parsed.date,
                        partner: None,
                        purposes: None,
                    }
                   );
        }
    }
}
