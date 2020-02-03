use super::*;
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Text {
    elements: Vec<TextElement>,
}
use nom::*;
impl<'a> Parse<'a> for Text {
    named!(
        parse(&'a str) -> Self,
        map!(
            many1!(
                delimited!(
                    space0,
                    complete!(TextElement::parse),
                    multispace0
                )
            ),
            |es| Self { elements: es }
        )
    );
}
use std::collections::HashMap;

impl Text {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
    pub fn push<E: Into<TextElement>>(&mut self, e: E) {
        self.elements.push(e.into())
    }
    pub fn clear(&mut self) {
        self.elements.clear()
    }
    pub fn unique_elements(&self) -> Vec<TextElement> {
        let mut r = self.elements.clone();
        r.sort();
        r.dedup();
        r
    }
    fn element_occurrences(&self) -> HashMap<TextElement, u32> {
        let mut occurrences = HashMap::new();
        for e in &self.elements {
            *occurrences.entry(e.clone()).or_insert(0) += 1;
        }
        occurrences
    }
    pub fn iter(self) -> <Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl From<Vec<TextElement>> for Text {
    fn from(elements: Vec<TextElement>) -> Self {
        Self {
            elements,
        }
    }
}

use std::fmt::{Debug, Display, self};
impl Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

use std::iter::{Iterator, IntoIterator};
impl IntoIterator for Text {
    type Item = TextElement;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
use std::ops::{Index, Range, RangeFull};
impl Index<Range<usize>> for Text {
    type Output = <Vec<TextElement> as Index<Range<usize>>>::Output;
    fn index(&self, idx: Range<usize>) -> &Self::Output {
        &self.elements[idx]
    }
}
impl Index<RangeFull> for Text {
    type Output = <Vec<TextElement> as Index<RangeFull>>::Output;
    fn index(&self, idx: RangeFull) -> &Self::Output {
        &self.elements[idx]
    }
}

mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_multiline() {
        let text = "Als \
                    Klasse
                    gilt
                    in

                    .";

        assert_eq!(Text::parse(text).unwrap().1,
                   Text::from(vec![
                           TextElement::Word(Word::from("Als")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("gilt")),
                           TextElement::Word(Word::from("in")),
                           TextElement::Punctuation(Punctuation::Dot),
                               ]
                   ))
    }
    fn parse_text() {
        let text = "Als Klasse gilt in der Mathematik, Klassenlogik und \
                    Mengenlehre eine Zusammenfassung beliebiger Objekte, \
                    definiert durch eine logische Eigenschaft, die alle \
                    Objekte der Klasse erfuellen.";

        assert_eq!(Text::parse(text).unwrap().1,
                   Text::from(vec![
                           TextElement::Word(Word::from("Als")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("gilt")),
                           TextElement::Word(Word::from("in")),
                           TextElement::Word(Word::from("der")),
                           TextElement::Word(Word::from("Mathematik")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("Klassenlogik")),
                           TextElement::Word(Word::from("und")),
                           TextElement::Word(Word::from("Mengenlehre")),
                           TextElement::Word(Word::from("eine")),
                           TextElement::Word(Word::from("Zusammenfassung")),
                           TextElement::Word(Word::from("beliebiger")),
                           TextElement::Word(Word::from("Objekte")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("definiert")),
                           TextElement::Word(Word::from("durch")),
                           TextElement::Word(Word::from("eine")),
                           TextElement::Word(Word::from("logische")),
                           TextElement::Word(Word::from("Eigenschaft")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("die")),
                           TextElement::Word(Word::from("alle")),
                           TextElement::Word(Word::from("Objekte")),
                           TextElement::Word(Word::from("der")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("erfuellen")),
                           TextElement::Punctuation(Punctuation::Dot)
                               ]
                   ))
    }
}
