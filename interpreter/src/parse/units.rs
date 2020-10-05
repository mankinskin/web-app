pub type Units = i32;
use crate::parse::*;

impl<'a> Parse<'a> for Units {
    named!(
        parse(&'a str) -> Self,
        map_res!(pair!(opt!(tag!("-")), digit1), |(neg, i): (
            Option<&str>,
            _
        )| {
            Units::from_str_radix(i, 10)
                .map_err(|_e| (i, ErrorKind::ParseTo))
                .map(|r| if neg.is_some() { -r } else { r })
        })
    );
}

mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_units() {
        for u in vec![
            "1", "7", "32", "1823", "0", "0002", "99999999", "-12", "-021", "-000",
        ] {
            assert_eq!(Units::parse(&u).unwrap().1, u.parse::<Units>().unwrap());
        }
    }
}
