use petgraph::graph::EdgeIndex;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fmt::{
        self,
        Debug,
        Display,
    },
    hash::Hash,
};

pub fn tokenizing_iter<T: Tokenize, C: Into<T>>(
    seq: impl Iterator<Item = C>,
) -> impl Iterator<Item = Token<T>> {
    seq.map(|c| c.into().into_token())
}
/// Trait for token that can be mapped in a sequence
pub trait Tokenize: TokenData + Wide + Hash + Eq + Copy + Debug {
    fn tokenize<T: Into<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Token<Self>> {
        let mut v = vec![];
        v.extend(tokenizing_iter(seq));
        //v.push(Token::End);
        v
    }
    fn into_token(self) -> Token<Self> {
        Token::Element(self)
    }
}
impl<T: TokenData + Wide + Hash + Eq + Copy + Debug> Tokenize for T {}

pub trait TokenData: Debug + PartialEq + Clone + Wide {}
impl<T: Debug + PartialEq + Clone + Wide> TokenData for T {}

pub trait Wide {
    fn width(&self) -> usize;
}
impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}
impl<T: Wide> Wide for &'_ T {
    fn width(&self) -> usize {
        (**self).width()
    }
}
impl<T: Wide> Wide for &'_ mut T {
    fn width(&self) -> usize {
        (**self).width()
    }
}

#[derive(Hash, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub struct NoToken;

impl Wide for NoToken {
    fn width(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo<T: Tokenize> {
    pub token: Token<T>,
    pub incoming_groups: Vec<Vec<Token<T>>>,
    pub outgoing_groups: Vec<Vec<Token<T>>>,
}
pub trait ContextLink: Sized + Clone {
    fn index(&self) -> &EdgeIndex;
    fn into_index(self) -> EdgeIndex {
        *self.index()
    }
}
impl ContextLink for EdgeIndex {
    fn index(&self) -> &EdgeIndex {
        self
    }
}
pub trait ContextMapping<E: ContextLink> {
    /// Get distance groups for incoming edges
    fn incoming(&self) -> &Vec<E>;
    fn outgoing(&self) -> &Vec<E>;

    ///// Get distance groups for incoming edges
    //fn incoming_distance_groups(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Self::Context>> {
    //    graph.distance_group_source_weights(self.incoming().iter().map(|e| e.into_index()))
    //}
    ///// Get distance groups for outgoing edges
    //fn outgoing_distance_groups(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Self::Context>> {
    //    graph.distance_group_target_weights(self.outgoing().iter().map(|e| e.into_index()))
    //}
}

pub trait TokenContext<T: Tokenize, E: ContextLink>: Sized {
    type Mapping: ContextMapping<E>;
    fn token(&self) -> &Token<T>;
    fn into_token(self) -> Token<T>;
    fn map_to_tokens(groups: Vec<Vec<Self>>) -> Vec<Vec<Token<T>>> {
        groups
            .into_iter()
            .map(|g| g.into_iter().map(|m| m.into_token()).collect())
            .collect()
    }
    fn mapping(&self) -> &Self::Mapping;
    fn mapping_mut(&mut self) -> &mut Self::Mapping;
    //fn get_info(&self, graph: &SequenceGraph<T>) -> ContextInfo<T> {
    //    let mut incoming_groups = self.mapping().incoming_distance_groups(graph);
    //    incoming_groups.reverse();
    //    let outgoing_groups = self.mapping().outgoing_distance_groups(graph);
    //    ContextInfo {
    //        token: self.token().clone(),
    //        incoming_groups: Self::map_to_tokens(incoming_groups),
    //        outgoing_groups: Self::map_to_tokens(outgoing_groups),
    //    }
    //}
}
pub fn groups_to_string<T: Tokenize, E: ContextLink, C: TokenContext<T, E> + Display>(
    groups: Vec<Vec<C>>,
) -> String {
    let mut lines = Vec::new();
    let max = groups.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..max {
        let mut line = Vec::new();
        for group in &groups {
            line.push(group.get(i).map(ToString::to_string));
        }
        lines.push(line);
    }
    lines.iter().fold(String::new(), |a, line| {
        format!(
            "{}{}\n",
            a,
            line.iter().fold(String::new(), |a, elem| {
                format!("{}{} ", a, elem.clone().unwrap_or_default())
            })
        )
    })
}

/// Type for storing elements of a sequence
#[derive(Copy, Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash)]
pub enum Token<T: Tokenize> {
    Element(T),
    Start,
    End,
}
impl<T: Tokenize + Display> Display for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Element(t) => t.to_string(),
                Token::Start => "START".to_string(),
                Token::End => "END".to_string(),
            }
        )
    }
}
impl<T: Tokenize> Wide for Token<T> {
    fn width(&self) -> usize {
        match self {
            Token::Element(t) => t.width(),
            Token::Start => 0,
            Token::End => 0,
        }
    }
}
impl<T: Tokenize> From<T> for Token<T> {
    fn from(e: T) -> Self {
        Token::Element(e)
    }
}
//impl<T: Tokenize> PartialEq<Token<T>> for &Token<T> {
//    fn eq(&self, rhs: &Token<T>) -> bool {
//        **self == *rhs
//    }
//}
impl<T: Tokenize> PartialEq<T> for Token<T> {
    fn eq(&self, rhs: &T) -> bool {
        match self {
            Token::Element(e) => *e == *rhs,
            _ => false,
        }
    }
}
//impl<T: Tokenize> PartialEq<Node<T>> for Token<T> {
//    fn eq(&self, rhs: &Node<T>) -> bool {
//        *self == *<Node<T> as TokenContext<T, Edge>>::token(rhs)
//    }
//}
impl PartialEq<Token<char>> for char {
    fn eq(&self, rhs: &Token<char>) -> bool {
        *rhs == *self
    }
}
