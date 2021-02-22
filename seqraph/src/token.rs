use crate::{
    graph::{
        node::NodeData,
    },
    node::Node,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    ops::{
        Add,
    },
    fmt::{
        self,
        Debug,
        Display,
    },
};
#[allow(unused)]
use tracing::{
    debug,
};
pub trait Wide {
    fn width(&self) -> usize;
}
impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}

pub trait TokenData: NodeData + Wide {}
impl<T: NodeData + Wide> TokenData for T {}

/// Type for storing elements of a sequence
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub enum Token<T: TokenData> {
    Element(T),
    Tokens(Vec<Self>),
    Start,
    End,
}
impl<T: TokenData + Display> Display for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Element(t) => t.to_string(),
                Token::Tokens(v) => format!(
                    "{:#?}",
                    v.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                ),
                Token::Start => "START".to_string(),
                Token::End => "END".to_string(),
            }
        )
    }
}
impl<T: TokenData> Add for Token<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        match self {
            Token::Tokens(mut v) => match other {
                Token::Tokens(t) => {
                    v.extend(t);
                    Token::Tokens(v)
                },
                _ => {
                    v.push(other);
                    Token::Tokens(v)
                },
            },
            _ => match other {
                Token::Tokens(t) => {
                    let mut v = vec![self];
                    v.extend(t);
                    Token::Tokens(v)
                },
                _ => Token::Tokens(vec![self, other]),
            },
        }
    }
}
impl<T: TokenData> Add<&Token<T>> for Token<T> {
    type Output = Self;
    fn add(self, other: &Self) -> Self::Output {
        self + other.clone()
    }
}
impl<T: TokenData> Add<Token<T>> for &Token<T> {
    type Output = Token<T>;
    fn add(self, other: Token<T>) -> Self::Output {
        self.clone() + other
    }
}
impl<T: TokenData> Add for &Token<T> {
    type Output = Token<T>;
    fn add(self, other: Self) -> Self::Output {
        self.clone() + other.clone()
    }
}
impl<T: TokenData> Wide for Token<T> {
    fn width(&self) -> usize {
        match self {
            Token::Element(t) => t.width(),
            Token::Tokens(v) => v.iter().fold(0, |acc, x| acc + x.width()),
            Token::Start => 0,
            Token::End => 0,
        }
    }
}
impl<T: TokenData> From<T> for Token<T> {
    fn from(e: T) -> Self {
        Token::Element(e)
    }
}
impl<T: TokenData> PartialEq<Token<T>> for &Token<T> {
    fn eq(&self, rhs: &Token<T>) -> bool {
        **self == *rhs
    }
}
impl<T: TokenData> PartialEq<T> for Token<T> {
    fn eq(&self, rhs: &T) -> bool {
        match self {
            Token::Element(e) => *e == *rhs,
            _ => false,
        }
    }
}
impl<T: TokenData> PartialEq<Node<T>> for Token<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        *self == *rhs.token()
    }
}
impl PartialEq<Token<char>> for char {
    fn eq(&self, rhs: &Token<char>) -> bool {
        *rhs == *self
    }
}
