use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum TextElement {
    Word(Word),
    Punctuation(Punctuation)
}
impl From<Word> for TextElement {
    fn from(w: Word) -> Self {
        Self::Word(w)
    }
}
impl From<Punctuation> for TextElement {
    fn from(p: Punctuation) -> Self {
        Self::Punctuation(p)
    }
}
use std::fmt::{Debug, Display, self};
impl Display for TextElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
use crate::parse::*;
impl<'a> Parse<'a> for TextElement {
    named!(
        parse(&'a str) -> Self,
        alt!(
            map!(
                Word::parse,
                |s| Self::from(s)
            ) |
            map!(
                Punctuation::parse,
                |s| Self::from(s)
            )
        )
    );
}
