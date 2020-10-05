use crate::chrono::*;
use crate::parse::*;

impl<'a> Parse<'a> for Date<Utc> {
    named!(
        parse(&'a str) -> Self,
        alt!(
                tag_no_case!("today") => { |_| Utc::today() } |
                tag_no_case!("yesterday") => { |_| Utc::today() - Duration::days(1) } |
                tag_no_case!("tomorrow") => { |_| Utc::today() + Duration::days(1) } |
                map_res!( // <d> days ago
                    terminated!(
                        digit1,
                        preceded!(
                            space1,
                            tag_no_case!("days ago")
                        )
                    ),
                    Units::parse
                ) => { |(_, d): (&str, Units)| Utc::today() - Duration::days(d.into()) } |
                map_res!( // in <d> days
                    delimited!(
                        terminated!(
                            tag_no_case!("in"),
                            space1
                        ),
                        digit1,
                        preceded!(
                            space1,
                            tag_no_case!("days")
                        )
                    ),
                    Units::parse
                ) => { |(_, d): (&str, Units)| Utc::today() + Duration::days(d.into()) }
        )
    );
}

struct Hours(pub u32);
impl<'a> Parse<'a> for Hours {
    named!(
        parse(&'a str) -> Self,
        map_res!(digit1, |i| {
            let u = u32::from_str_radix(i, 10).map_err(|_e| (i, ErrorKind::ParseTo))?;
            if (0..24).contains(&u) {
                Ok(Hours(u))
            } else {
                Err((i, ErrorKind::ParseTo))
            }
        })
    );
}
impl Into<u32> for Hours {
    fn into(self) -> u32 {
        self.0
    }
}
struct Minutes(pub u32);
impl<'a> Parse<'a> for Minutes {
    named!(
        parse(&'a str) -> Self,
        map_res!(digit1, |i| {
            let u = u32::from_str_radix(i, 10).map_err(|_e| (i, ErrorKind::ParseTo))?;
            if (0..60).contains(&u) {
                Ok(Minutes(u))
            } else {
                Err((i, ErrorKind::ParseTo))
            }
        })
    );
}
impl Into<u32> for Minutes {
    fn into(self) -> u32 {
        self.0
    }
}
struct Seconds(pub u32);
impl<'a> Parse<'a> for Seconds {
    named!(
        parse(&'a str) -> Self,
        map_res!(digit1, |i| {
            let u = u32::from_str_radix(i, 10).map_err(|_e| (i, ErrorKind::ParseTo))?;
            if (0..60).contains(&u) {
                Ok(Seconds(u))
            } else {
                Err((i, ErrorKind::ParseTo))
            }
        })
    );
}
impl Into<u32> for Seconds {
    fn into(self) -> u32 {
        self.0
    }
}
impl<'a> Parse<'a> for NaiveTime {
    // <u32>(:<u32>(:<u32>))
    // <hours> o'clock
    // <hours>(:minutes) am | pm
    // <hours> am | pm
    named!(
        parse(&'a str) -> Self,
        map!(
            tuple!(
                Hours::parse,
                opt!(complete!(tuple!(
                    preceded!(tag!(":"), Minutes::parse),
                    opt!(complete!(preceded!(tag!(":"), Seconds::parse)))
                )))
            ),
            |(h, tail)| {
                let (m, s) = tail
                    .map(|(minutes, osec)| (minutes, osec.unwrap_or(Seconds(0))))
                    .unwrap_or((Minutes(0), Seconds(0)));
                NaiveTime::from_hms(h.into(), m.into(), s.into())
            }
        )
    );
}
impl<'a> Parse<'a> for DateTime<Utc> {
    named!(
        parse(&'a str) -> Self,
        map!(
            tuple!(
                Date::parse,
                opt!(complete!(preceded!(
                    delimited!(space0, tag_no_case!("at"), space0),
                    NaiveTime::parse
                )))
            ),
            |(d, t)| {
                match t {
                    Some(time) => d.and_time(time).unwrap(),
                    None => d.and_hms(0, 0, 0),
                }
            }
        )
    );
}
mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_date() {
        let utc_today = Utc::today();
        let utc_yesterday = Utc::today() - Duration::days(1);
        let utc_tomorrow = Utc::today() + Duration::days(1);
        assert_eq!(Date::parse("Today").unwrap().1, utc_today);
        assert_eq!(Date::parse("tOdAY").unwrap().1, utc_today);
        assert_eq!(Date::parse("Yesterday").unwrap().1, utc_yesterday);
        assert_eq!(Date::parse("Tomorrow").unwrap().1, utc_tomorrow);
        for &d in &[1, 2, 3, 20, 100, 2134242] {
            assert_eq!(
                Date::parse(&format!("{} days ago", d)).unwrap().1,
                utc_today - Duration::days(d)
            );
        }
        for &d in &[1, 2, 3, 20, 100, 2134242] {
            assert_eq!(
                Date::parse(&format!("in {} days", d)).unwrap().1,
                utc_today + Duration::days(d)
            );
        }
    }
    #[test]
    fn parse_time() {
        use itertools::*;
        use std::ops::Range;
        type ClockRange = (Range<u32>, Range<u32>, Range<u32>);
        let mut cases = vec![
            ("{}".to_string(), (0..24, 0..1, 0..1)),
            ("{}:{}".to_string(), (0..24, 0..60, 0..1)),
            ("{}:{}:{}".to_string(), (0..24, 0..60, 0..60)),
        ];
        let mut ampm = iproduct!(vec!["", " "], vec!["am", "pm",]).map(|(s, v)| s.to_string() + &v);

        let mut formats: Vec<(String, ClockRange)> = iproduct!(cases.clone(), ampm.clone())
            .map(|((fmt, range), end)| (fmt + &end, range))
            .collect();
        for (fmt, (h, m, s)) in &mut formats {
            *h = 0..13;
        }
        formats.append(&mut cases);

        let mut test_values = Vec::new();

        for val in iproduct!(
            &["0", "1", "6", "09", "11", "12", "13", "16", "23", "24", "25", "20109"],
            &["0", "1", "2", "30", "59", "60", "61", "343"],
            &["0", "1", "2", "30", "59", "60", "61", "7144"]
        ) {
            test_values.push(val);
        }

        let test_data =
            iproduct!(formats, test_values).map(|((fmt, (hrange, mrange, srange)), (h, m, s))| {
                let fmt = fmt
                    .replacen("{}", h, 1)
                    .replacen("{}", m, 1)
                    .replacen("{}", s, 1);

                let hv = h.parse::<u32>().unwrap();
                let mv = m.parse::<u32>().unwrap();
                let sv = s.parse::<u32>().unwrap();

                (fmt, (hrange, mrange, srange), (hv, mv, sv))
            });

        for (fmt, (hrange, mrange, srange), (h, m, s)) in test_data {
            //println!("Testing \"{}\" == ({},{},{})", fmt, h, m, s);
            let result = NaiveTime::parse(&fmt).map(|t| t.1);
            if hrange.contains(&h) && mrange.contains(&m) && srange.contains(&s) {
                let expected = if hrange.end < 13 && fmt.ends_with("pm") {
                    NaiveTime::from_hms((h/* + 12*/), m, s)
                } else {
                    NaiveTime::from_hms(h, m, s)
                };
                if mrange.contains(&m) && srange.contains(&s) {
                    assert_eq!(result, Ok(expected));
                }
            }
            // TODO: Improve this test. right now it filters a lot of cases
            // out, and errors are not tested
        }
    }
}
