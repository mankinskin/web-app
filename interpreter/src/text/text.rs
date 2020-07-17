use super::*;
use crate::sentence::*;
use serde::{
    Serialize,
    Deserialize,
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, self};
use std::ops::{Index, Range, RangeFull, RangeFrom, RangeTo, Deref};

#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize, Deserialize)]
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
                    multispace0,
                    complete!(TextElement::parse),
                    multispace0
                )
            ),
            |es| Self { elements: es }
        )
    );
}
impl<'a> TryFrom<&'a str> for Text {
    type Error = nom::Err<(&'a str, nom::error::ErrorKind)>;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s).map(|r| r.1)
    }
}

impl Text {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
    pub fn push<E: Into<TextElement>>(&mut self, e: E) {
        self.elements.push(e.into())
    }
    pub fn push_front<E: Into<TextElement>>(&mut self, e: E) {
        let mut tmp = vec![e.into()];
        tmp.extend(self.elements.clone());
        self.elements = tmp;
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
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn extend(&mut self, other: Self) {
        self.elements.extend(other.elements)
    }
    pub fn prepend(&mut self, other: Self) {
        let mut tmp = other.elements;
        tmp.extend(self.elements.clone());
        self.elements = tmp;
    }
    pub fn to_sentences(self) -> Vec<Text> {
        self.elements
            .split(|e| e.is_stop())
            .map(|s| Text::from(s))
            .filter(|t| t.len() > 0)
            .collect()
    }
}
impl From<Vec<TextElement>> for Text {
    fn from(elements: Vec<TextElement>) -> Self {
        Self {
            elements,
        }
    }
}
impl From<&[TextElement]> for Text {
    fn from(slice: &[TextElement]) -> Self {
        Self::from(
            slice.iter().cloned().collect::<Vec<_>>()
        )
    }
}
impl Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Deref for Text {
    type Target = [TextElement];
    fn deref(&self) -> &Self::Target {
        self.elements.deref()
    }
}
impl Index<usize> for Text {
    type Output = <Vec<TextElement> as Index<usize>>::Output;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.elements[idx]
    }
}
impl Index<RangeTo<usize>> for Text {
    type Output = <Vec<TextElement> as Index<RangeTo<usize>>>::Output;
    fn index(&self, idx: RangeTo<usize>) -> &Self::Output {
        &self.elements[idx]
    }
}
impl Index<RangeFrom<usize>> for Text {
    type Output = <Vec<TextElement> as Index<RangeFrom<usize>>>::Output;
    fn index(&self, idx: RangeFrom<usize>) -> &Self::Output {
        &self.elements[idx]
    }
}
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

        assert_eq!(Text::try_from(text).unwrap(),
                   Text::from(vec![
                           TextElement::Word(Word::from("Als")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("gilt")),
                           TextElement::Word(Word::from("in")),
                           TextElement::Punctuation(Punctuation::Dot),
                               ]
                   ))
    }
    #[test]
    fn parse_text() {
        let text = "Als Klasse gilt in der Mathematik, Klassenlogik und \
                    Mengenlehre eine Zusammenfassung beliebiger Objekte, \
                    definiert durch eine logische Eigenschaft, die alle \
                    Objekte der Klasse erfuellen.";

        assert_eq!(Text::try_from(text).unwrap(),
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
    #[test]
    fn to_sentences() {
        let text = Text::try_from("A B C. A C D. C B A").unwrap();
        let sentences = text.to_sentences();
        let a = TextElement::Word(Word::from("A"));
        let b = TextElement::Word(Word::from("B"));
        let c = TextElement::Word(Word::from("C"));
        let d = TextElement::Word(Word::from("D"));
        assert_eq!(sentences, vec![
            Text::from(vec![a.clone(), b.clone(), c.clone()]),
            Text::from(vec![a.clone(), c.clone(), d]),
            Text::from(vec![c, b, a])
        ]);
    }
}
