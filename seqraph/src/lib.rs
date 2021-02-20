pub mod graph;
pub mod mapping;
//pub mod grammar;

use graph::{
    Graph,
};
use mapping::{
    Mappable,
    Mapped,
    Node,
    Token,
    Edge,
    TokenData,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fmt::Debug,
    ops::{
        Deref,
        DerefMut,
    },
};
use petgraph::graph::NodeIndex;

/// Graph of T: TokenData + Mappable mapping possible distances
/// between nodes to prefix and postfix nodes
#[derive(Debug)]
pub struct SequenceGraph<T>
where
    T: TokenData + Mappable,
{
    graph: Graph<Node<T>, usize>,
}
impl<T> SequenceGraph<T>
where
    T: TokenData + Mappable,
{
    pub fn new() -> Self {
        let graph = Graph::new();
        Self { graph }
    }
    //pub fn query<T: Into<T> + Into<char> + Clone, I: Iterator<Item = T> + Clone>(
    //    &self,
    //    seq: I,
    //) -> Option<NodeInfo<T>> {
    //    let sym = seq.clone().next().unwrap();
    //    let sym = match <T as Into<char>>::into(sym.clone()) {
    //        '*' => Token::Start,
    //        '#' => Token::End,
    //        _ => Token::Element(<T as Into<T>>::into(sym)),
    //    };
    //    self.get_node_info(&sym)
    //}
    pub fn read_sequence<N: Into<T>, I: Iterator<Item = N>>(&mut self, seq: I) {
        let seq = T::sequenced(seq);
        for index in 0..seq.len() {
            self.read_to_node(&seq[..], index);
        }
    }
    fn read_to_node(&mut self, seq: &[Token<T>], index: usize) {
        let element = &seq[index];
        let end = seq.len() - 1;
        for pre in 0..index {
            let l = &seq[pre];
            let ld = index - pre;
            for succ in (index + 1)..=end {
                let r = &seq[succ];
                let rd = succ - index;
                self.insert_node_neighborhood(l.clone(), ld, element.clone(), rd, r.clone());
            }
        }
    }
    pub fn add_node(&mut self, token: Token<T>) -> NodeIndex {
        self.graph.add_node(Node::new(token))
    }
    fn insert_node_neighborhood(
        &mut self,
        l: Token<T>, // left-hand element
        ld: usize,   // distance to left-hand element
        x: Token<T>, // center element
        rd: usize,   // distance to right-hand element
        r: Token<T>, // right-hand element
    ) {
        let li = self.add_node(l);
        let xi = self.add_node(x);
        let ri = self.add_node(r);
        let le = self.add_edge(li, xi, ld);
        let re = self.add_edge(xi, ri, rd);
        self.graph
            .node_weight_mut(xi)
            .unwrap()
            .mapping_mut()
            .add_transition(Edge::new(le, li, ld), Edge::new(re, ri, rd));
    }
    //pub fn knows_sequence(&self, seq: &[Token<T>]) -> bool {
    //    if let Some(nodes) = self.graph.find_node_weights(seq.into_iter()) {
    //        //nodes.iter().fold(
    //        true
    //    } else {
    //        false
    //    }
    //}
    //#[allow(unused)]
    //fn groups_to_string(groups: Vec<Vec<Node<T>>>) -> String {
    //    let mut lines = Vec::new();
    //    let max = groups.iter().map(Vec::len).max().unwrap_or(0);
    //    for i in 0..max {
    //        let mut line = Vec::new();
    //        for group in &groups {
    //            line.push(group.get(i).map(ToString::to_string));
    //        }
    //        lines.push(line);
    //    }
    //    lines.iter().fold(String::new(), |a, line| {
    //        format!(
    //            "{}{}\n",
    //            a,
    //            line.iter().fold(String::new(), |a, elem| {
    //                format!("{}{} ", a, elem.clone().unwrap_or(String::new()))
    //            })
    //        )
    //    })
    //}
    //fn map_to_tokens(groups: Vec<Vec<Node<T>>>) -> Vec<Vec<Token<T>>> {
    //    groups
    //        .iter()
    //        .map(|g| g.iter().map(|m| m.token.clone()).collect())
    //        .collect()
    //}
    //pub fn get_node_info<T: PartialEq<Node<T>>>(&self, element: &T) -> Option<NodeInfo<T>> {
    //    let node = self.find_node_weight(element)?;
    //    let mut incoming_groups: Vec<Vec<Node<T>>> = node.mapping().incoming_distance_groups(&self);
    //    incoming_groups.reverse();
    //    let outgoing_groups: Vec<Vec<Node<T>>> = node.mapping().outgoing_distance_groups(&self);
    //    Some(NodeInfo {
    //        element: node.token,
    //        incoming_groups: Self::map_to_tokens(incoming_groups),
    //        outgoing_groups: Self::map_to_tokens(outgoing_groups),
    //    })
    //}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo<T: TokenData> {
    pub element: Token<T>,
    pub incoming_groups: Vec<Vec<Token<T>>>,
    pub outgoing_groups: Vec<Vec<Token<T>>>,
}
impl<T: TokenData + Mappable> Deref for SequenceGraph<T> {
    type Target = Graph<Node<T>, usize>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<T: TokenData + Mappable> DerefMut for SequenceGraph<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    lazy_static::lazy_static! {
        pub static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c', 'd', 'e']);
        pub static ref SEQS: Vec<&'static str> = Vec::from([
            "abcd",
        ]);
        pub static ref EDGES: Vec<(Token<char>, Token<char>, usize)> = {
            Vec::from([
                (Token::Start, 'a'.into(), 1),
                (Token::Start, 'b'.into(), 1),
                (Token::Start, 'b'.into(), 2),
                (Token::Start, 'b'.into(), 3),
                (Token::Start, 'c'.into(), 2),
                (Token::Start, 'c'.into(), 3),
                ('a'.into(), Token::End, 3),
                ('b'.into(), Token::End, 3),
                ('b'.into(), Token::End, 2),
                ('b'.into(), Token::End, 1),
                ('c'.into(), Token::End, 2),
                ('c'.into(), Token::End, 1),
                ('a'.into(), 'b'.into(), 1),
                ('a'.into(), 'b'.into(), 1),
                ('a'.into(), 'b'.into(), 1),
                ('a'.into(), 'c'.into(), 2),
                ('b'.into(), 'c'.into(), 1),
                ('c'.into(), 'b'.into(), 1),
                ('b'.into(), 'b'.into(), 1),
                ('b'.into(), 'b'.into(), 2),
            ])
        };
        pub static ref G: SequenceGraph<char> = {
            let mut g = SequenceGraph::new();
            for &s in SEQS.iter() {
                g.read_sequence(s.chars());
            }
            g
        };
    }
    //#[test]
    //fn knows_sequence() {
    //    for (l, r, w) in EDGES.iter() {
    //        assert!(G.has_node_edge(l, r, w), "({}, {}, {})", l, r, w);
    //    }
    //}
}
