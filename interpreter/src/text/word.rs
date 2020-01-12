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
impl<'a> Parse<'a> for Word {
    named!(
        parse(&'a str) -> Self,
        map!(
            take_while1!(nom::AsChar::is_alpha),
            |s| Self { chars: s.into() }
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
