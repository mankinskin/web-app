use super::*;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Text {
    elements: Vec<TextElement>,
}
use nom::*;
impl<'a> Parse<'a> for Text {
    named!(
        parse(&'a str) -> Self,
        map!(
            preceded!(
                space0,
                    many1!(
                        terminated!(
                            TextElement::parse,
                            space0
                            )
                    )
            ),
            |es| Self { elements: es })
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
