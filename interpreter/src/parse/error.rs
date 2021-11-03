use nom::error::*;

#[derive(Clone, Debug)]
struct ParseError<'a> {
	chain: Vec<(&'a str, ErrorKind)>,
}

impl<'a> nom::error::ParseError<&'a str> for ParseError<'a> {
	fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
		Self {
			chain: vec![(input, kind)],
		}
	}
	fn append(input: &'a str, kind: ErrorKind, other: Self) -> Self {
		let mut next = other.clone();
		next.chain.push((input, kind));
		next
	}
}

impl<'a> From<(&'a str, ErrorKind)> for ParseError<'a> {
	fn from((input, kind): (&'a str, ErrorKind)) -> Self {
		<Self as nom::error::ParseError<&'a str>>::from_error_kind(input, kind)
	}
}
