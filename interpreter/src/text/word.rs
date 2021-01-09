use super::*;
use serde::{
	Serialize,
	Deserialize,
};
#[derive(PartialEq, Eq, Clone, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Word {
	chars: String,
}

impl Word {
	fn new(s: &str) -> Self {
		Self {
			chars: s.to_lowercase()
		}
	}
}

impl From<String> for Word {
	fn from(s: String) -> Self {
		Self::new(&s)
	}
}
impl From<&str> for Word {
	fn from(s: &str) -> Self {
		Self::new(s)
	}
}
impl AsRef<[u8]> for Word {
	fn as_ref(&self) -> &[u8] {
		self.chars.as_ref()
	}
}
use nom::*;
use nom::combinator::*;
use nom::multi::*;
use nom_unicode::complete::{alpha1, alphanumeric1};

impl<'a> Parse<'a> for Word {
	named!(
		parse(&'a str) -> Self,
		map!(
			alphanumeric1,
			|w| Self::new(w)
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
		write!(f, "{}", self)
	}
}
mod tests {
	#[allow(unused)]
	use super::*;
	#[test]
	fn parse_number() {
		let words = vec![
			"19",
			"20",
			"01012",
		];
		for word in words {
			assert_eq!(Word::parse(word).unwrap().1, Word::from(word));
		}
	}
	fn parse_word() {
		let words = vec![
			"hello",
			"Hello",
			"Hi",
			"yes",
			"aha",
			"Mathematik",
			"mathmatical",
			"erfuellen",
			"erfüllen"
		];
		for word in words {
			assert_eq!(Word::parse(word).unwrap().1, Word::from(word));
		}
	}
}
