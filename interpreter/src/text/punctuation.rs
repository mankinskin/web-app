use super::*;
#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum Punctuation {
    Dot,
    Comma,
    Colon,
    Semicolon,
    Quote,
    DoubleQuote,
    QuestionMark,
    ExclamationMark,
    Equals,
    Greater,
    Less,
}
use std::fmt::{Debug, Display, self};
impl Display for Punctuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Dot => ".",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::Quote => "\'",
            Self::DoubleQuote => "\"",
            Self::QuestionMark => "?",
            Self::ExclamationMark => "!",
            Self::Equals => "=",
            Self::Greater => ">",
            Self::Less => "<",
        })
    }
}
use crate::parse::*;

impl<'a> Parse<'a> for Punctuation {
    named!(
        parse(&'a str) -> Self,
        alt!(
            map!(tag!(".") , |_| Self::Dot) |
            map!(tag!("=") , |_| Self::Equals) |
            map!(tag!(">") , |_| Self::Greater) |
            map!(tag!("<") , |_| Self::Less) |
            map!(tag!("!") , |_| Self::ExclamationMark) |
            map!(tag!("?") , |_| Self::QuestionMark) |
            map!(tag!(",") , |_| Self::Comma) |
            map!(tag!(":") , |_| Self::Colon) |
            map!(tag!(";") , |_| Self::Semicolon) |
            map!(tag!("\'"), |_| Self::Quote) |
            map!(tag!("\""), |_| Self::DoubleQuote)
        )
    );
}
mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_punctuation() {
        assert_eq!(Punctuation::parse(",").unwrap().1, Punctuation::Comma);
        assert_eq!(Punctuation::parse(".").unwrap().1, Punctuation::Dot);
        assert_eq!(Punctuation::parse(":").unwrap().1, Punctuation::Colon);
        assert_eq!(Punctuation::parse(";").unwrap().1, Punctuation::Semicolon);
        assert_eq!(Punctuation::parse("\'").unwrap().1, Punctuation::Quote);
        assert_eq!(Punctuation::parse("\"").unwrap().1, Punctuation::DoubleQuote);
        assert_eq!(Punctuation::parse("!").unwrap().1, Punctuation::ExclamationMark);
        assert_eq!(Punctuation::parse("?").unwrap().1, Punctuation::QuestionMark);
        assert_eq!(Punctuation::parse("=").unwrap().1, Punctuation::Equals);
        assert_eq!(Punctuation::parse("<").unwrap().1, Punctuation::Less);
        assert_eq!(Punctuation::parse(">").unwrap().1, Punctuation::Greater);
    }
}
