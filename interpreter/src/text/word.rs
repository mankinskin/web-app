use super::*;

#[derive(PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub struct Word {
    chars: String,
}

impl From<&str> for Word {
    fn from(s: &str) -> Self {
        Self {
            chars: s.into(),
        }
    }
}
use nom::character::complete::*;
use nom::*;
use nom::combinator::*;
use nom::multi::*;
use nom_unicode::complete::{alpha1};

impl<'a> Parse<'a> for Word {
    named!(
        parse(&'a str) -> Self,
        map!(
            alpha1,
            |w| Self { chars: w.to_string() }
            )
    );
}
use std::fmt::{Debug, Display, self};
impl Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.chars)
    }
}
impl Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.chars)
    }
}
mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_word() {
        let words = vec![
            "hello",
            "Hello",
            "Hi",
            "yes",
            "aha",
            "Mathematik",
            "mathmatical",
            "erfuellen"
        ];
        for word in words {
            assert_eq!(Word::parse(word).unwrap().1, Word::from(word));
        }
    }
}
