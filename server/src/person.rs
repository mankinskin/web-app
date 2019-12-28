use crate::interpreter::parse::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Person {
    name: String,
}

impl Person {
    fn new<S: ToString>(name: S) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl From<&str> for Person {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl<'a> Parse<'a> for Person {
    named!(
        parse(&'a str) -> Self,
        map!(
            alphanumeric1,
            |a| Person::new(a)
            )
        );
}

pub enum Subject {
    Me,
    Person(Person),
}
impl<'a> Parse<'a> for Subject {
    named!(
        parse(&'a str) -> Self,
        alt!(
            tag!("I") => { |_| Self::Me } |
            tag!("me") => { |_| Self::Me } |
            preceded!(space0,
                      Person::parse) => { |a| Self::Person(a) }
            )
        );
}
