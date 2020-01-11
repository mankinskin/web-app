use super::*;
#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum Punctuation {
    Dot,
    Comma,
    Colon,
    Semicolon,
    Quote,
    DoubleQuote,
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
        })
    }
}
use crate::parse::*;

impl<'a> Parse<'a> for Punctuation {
    named!(
        parse(&'a str) -> Self,
                map!(
                    alt!(
                        tag!(".") |
                        tag!(",") |
                        tag!(":") |
                        tag!(";") |
                        tag!("\'") |
                        tag!("\"")
                    ),
                    |s| match s {
                        "." => Self::Dot,
                        "," => Self::Comma,
                        ":" => Self::Colon,
                        ";" => Self::Semicolon,
                        "\'" => Self::Quote,
                        "\"" => Self::DoubleQuote,
                        _ => panic!("Unknown Punctuation"),
                    }
                )
    );
}
