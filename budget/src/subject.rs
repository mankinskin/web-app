use interpreter::parse::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Subject {
    Me,
    Person(String),
}
impl<'a> Parse<'a> for Subject {
    named!(
        parse(&'a str) -> Self,
        alt!(
            tag_no_case!("i") => { |_| Self::Me } |
            tag_no_case!("me") => { |_| Self::Me } |
            alpha1 => { |a: &str| Self::Person(a.into()) }
            )
        );
}

impl<S: Into<String>> From<S> for Subject {
    fn from(s: S) -> Self {
        Self::Person(s.into())
    }
}
impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Me => "Me".to_string(),
            Self::Person(p) => p.to_string(),
        })
    }
}
