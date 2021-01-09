use crate::{
	*,
	set::*,
	parse::*,
};

use nom::*;
use nom::combinator::*;
use nom::multi::*;
use nom_unicode::complete::{alpha1, alphanumeric1};

/// A first level parse token
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
	/// an identifier "abc" "abc1"
	Ident(String),
	/// numeric digits "abc"
	Digits(String),
	/// +
	Plus,
	/// -
	Minus,
	/// /
	Slash,
	/// \
	Backslash,
	/// a path separator ":"
	Colon,
	/// a path separator "::"
	PathSeparator,
	/// a semicolon ";"
	Semicolon,
	/// a semicolon "."
	Dot,
	/// a comma ","
	Comma,
}
impl<'a> Parse<'a> for Token {
	named!(
		parse(&'a str) -> Self,
		map!(
			alt!(
				map!(digit1,
					 |digits| Token::Digits(String::from(digits))
				) |
				map!(alphanumeric1,
					 |alpha| Token::Ident(String::from(alpha))
				) |
				map!(char!('+'),
					 |_| Token::Plus
				) |
				map!(char!('-'),
					 |_| Token::Minus
				) |
				map!(char!('/'),
					 |_| Token::Slash
				) |
				map!(char!('\\'),
					 |_| Token::Backslash
				) |
				map!(char!(':'),
					 |_| Token::Colon
				) |
				map!(char!(';'),
					 |_| Token::Semicolon
				) |
				map!(char!('.'),
					 |_| Token::Dot
				) |
				map!(char!(','),
					 |_| Token::Comma
				) |
				map!(tag!("::"),
					 |_| Token::PathSeparator
				)
			),
			|token| token
		)
	);
}
/// A parsed model file
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleFile {
	buffer: String,
}
impl ModuleFile {
	pub fn tokenize(self) -> TokenBuffer {
		TokenBuffer::parse(&self.buffer).unwrap().1
	}
}
impl From<String> for ModuleFile {
	fn from(buffer: String) -> Self {
		Self {
			buffer
		}
	}
}
/// A tokenized model file
#[derive(Debug, Clone, PartialEq, Eq)]
struct TokenBuffer {
	buffer: Vec<Token>,
}
impl From<Vec<Token>> for TokenBuffer {
	fn from(buffer: Vec<Token>) -> Self {
		Self {
			buffer
		}
	}
}
impl TokenBuffer {
	fn monomorphize(self) -> MonoBuffer {
		MonoBuffer::from(self.buffer)
	}
}
impl<'a> Parse<'a> for TokenBuffer {
	named!(
		parse(&'a str) -> Self,
		map!(
			many0!(Token::parse),
			|tokens| Self::from(tokens)
		)
	);
}
/// A monomorphized token buffer
#[derive(Debug, Clone, PartialEq, Eq)]
struct MonoBuffer {
	buffer: Vec<Token>,
}
impl From<Vec<Token>> for MonoBuffer {
	fn from(buffer: Vec<Token>) -> Self {
		Self {
			buffer
		}
	}
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct AttributeDefinition {
	name: String,
	set: SetId,
}
/// Module path of the set
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleDefinition {
	name: String,
	submodules: Vec<ModuleDefinition>,
}
/// Module path of the set
#[derive(Debug, Clone, PartialEq, Eq)]
struct SetPath {

}
/// Defines set
#[derive(Debug, Clone, PartialEq, Eq)]
struct SetDefinition {
	/// name of the set
	name: String,
	/// module path
	path: Vec<String>,
	/// set attributes
	attributes: Vec<AttributeDefinition>,
}
impl SetDefinition {
	/// defines an empty set at path
	pub fn from_path(path: SetPath, name: String) -> Self {
		Self {
			name,
			path: Vec::new(),
			attributes: Vec::new(),
		}
	}
}


mod tests {
	use super::*;
	lazy_static! {
		static ref SET_DEFS: String = {
			String::from("\
				Landfahrzeuge
				Schiffe
				Flugzeuge
			")
		};
		static ref JOIN_CALL: String = {
			String::from("\
				Transportmittel = Landfahrzeuge + Schiffe + Flugzeuge\
			")
		};
	}
	#[test]
	fn parse() {
		let file = ModuleFile::from(SET_DEFS.clone());
		let tokens = file.tokenize();
		let mono = tokens.monomorphize();
	}
	#[test]
	fn new_sets() {
		let file = ModuleFile::from(SET_DEFS.clone());
	}
	#[test]
	fn joining() {
		let file = ModuleFile::from(SET_DEFS.clone() + &JOIN_CALL);
	}
}

