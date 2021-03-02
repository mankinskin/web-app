#![feature(test)]

extern crate test;

pub mod arithmetic_bool;
pub mod graph;
pub mod mapping;
pub mod node;
pub mod token;
//pub mod grammar;

use graph::Graph;
use mapping::{
    EdgeMapping,
    LoadedEdge,
    LoadedEdgeMapping,
    Edge,
};
use node::{
    LoadedNode,
    Node,
};
use petgraph::{
    graph::{
        NodeIndex,
    },
    Direction,
};
use std::fmt::Debug;
use std::ops::{
    Deref,
    DerefMut,
};
use token::{
    Token,
    TokenContext,
    Tokenize,
    ContextLink,
};
#[allow(unused)]
use tracing::debug;

/// Graph of T: TokenData + Mappable mapping possible distances
/// between nodes to prefix and postfix nodes
#[derive(Debug)]
pub struct SequenceGraph<T>
where
    T: Tokenize,
{
    graph: Graph<Node<T>, usize>,
}
impl<T> SequenceGraph<T>
where
    T: Tokenize,
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
    pub fn read_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) {
        let seq = T::tokenize(seq.into_iter());
        for index in 1..seq.len() {
            let ni = self.node(seq[index].clone());
            self.read_preceding_context(&seq[..index], ni);
        }
    }
    fn read_preceding_context(&mut self, text: &[Token<T>], node: NodeIndex) {
        let end = text.len();
        for index in 0..end {
            let element = &text[index];
            let rd = end - index;
            for pre in 0..index {
                let l = &text[pre];
                let ld = index - pre;
                let li = self.node(l.clone());
                let xi = self.node(element.clone());
                self.insert_node_neighborhood(li, ld, xi, rd, node);
            }
        }
    }
    fn read_from<N: Into<T>, TI: Iterator<Item=N>>(&mut self, text: TI) -> Option<()> {
        let cap = text.size_hint().0;
        T::tokenize(text).into_iter().fold(Vec::with_capacity(cap), |mut stack, token| {
            let ni = self.node(token.clone());
            self.read_preceding_context(&stack, ni);
            stack.push(token);
            stack
        });
        None
    }
    fn insert_node_neighborhood(
        &mut self,
        li: NodeIndex, // left-hand element
        ld: usize,   // distance to left-hand element
        xi: NodeIndex, // center element
        rd: usize,   // distance to right-hand element
        ri: NodeIndex, // right-hand element
    ) {
        let le = self.edge(li, xi, ld);
        let re = self.edge(xi, ri, rd);
        self.node_weight_mut(xi)
            .unwrap()
            .mapping_mut()
            .add_transition(le, re);
    }
    pub fn node(&mut self, token: Token<T>) -> NodeIndex {
        self.graph.node(&Node::new(token))
    }
    pub fn edge(&mut self, li: NodeIndex, ri: NodeIndex, w: usize) -> Edge {
        Edge::new(self.graph.edge(li, ri, w))
    }
    pub fn load_node_from<P: PartialEq<Node<T>> + Debug>(&self, p: P) -> Option<LoadedNode<T>> {
        let index = self.graph.find_node_index(p)?;
        self.load_node(index)
    }
    pub fn load_node(&self, index: NodeIndex) -> Option<LoadedNode<T>> {
        let node = self
            .graph
            .node_weight(index)
            .expect("Find node by index.")
            .clone();
        let mapping = self.load_mapping(node.mapping)?;
        Some(LoadedNode::new(index, node.token, mapping))
    }
    pub fn load_edge(&self, edge: Edge, direction: Direction) -> Option<LoadedEdge> {
        let index = *edge.index();
        let target = self.graph.edge_endpoint_directed(index, direction)?;
        let weight = self.graph.edge_weight(index)?.clone();
        Some(LoadedEdge {
            index,
            node: target,
            dist: weight,
        })
    }
    pub fn load_mapping(&self, mapping: EdgeMapping) -> Option<LoadedEdgeMapping> {
        let incoming: Vec<_> = mapping
            .incoming
            .into_iter()
            .map(|i| self.load_edge(i, Direction::Outgoing).unwrap())
            .collect();
        let outgoing: Vec<_> = mapping
            .outgoing
            .into_iter()
            .map(|i| self.load_edge(i, Direction::Incoming).unwrap())
            .collect();
        Some(LoadedEdgeMapping {
            matrix: mapping.matrix,
            incoming,
            outgoing,
        })
    }
    //pub fn knows_sequence(&self, seq: &[Token<T>]) -> bool {
    //    if let Some(nodes) = self.graph.find_node_weights(seq.into_iter()) {
    //        //nodes.iter().fold(
    //        true
    //    } else {
    //        false
    //    }
    //}
}
impl<T: Tokenize> Deref for SequenceGraph<T> {
    type Target = Graph<Node<T>, usize>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<T: Tokenize> DerefMut for SequenceGraph<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;
    use crate::assert_distances_match;
    use std::collections::HashSet;
    use test::Bencher;
    lazy_static::lazy_static! {
        pub static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c', 'd', 'e']);
        pub static ref SEQS: Vec<&'static str> = Vec::from([
            "",
            "bc",
            "aa",
            "abc",
            "bcade",
            "bcaade",
            "bcbcabc",
            "abcaa",
            "abcaabcbcabcbcade",
        ]);
        pub static ref EDGES: Vec<(Token<char>, Token<char>, usize)> = {
            Vec::from([
                (Token::Start, 'a'.into(), 1),
                (Token::Start, 'b'.into(), 2),
                (Token::Start, 'c'.into(), 3),
                (Token::Start, 'd'.into(), 4),
                ('a'.into(), Token::End, 4),
                ('b'.into(), Token::End, 3),
                ('c'.into(), Token::End, 2),
                ('d'.into(), Token::End, 1),
                ('a'.into(), 'b'.into(), 1),
                ('a'.into(), 'c'.into(), 2),
                ('a'.into(), 'd'.into(), 3),
                ('b'.into(), 'c'.into(), 1),
                ('b'.into(), 'd'.into(), 2),
                ('c'.into(), 'd'.into(), 1),
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
    #[bench]
    fn bench_read_sequence(b: &mut Bencher) {
        b.iter(|| {
            let mut g = SequenceGraph::<char>::new();
            for &s in SEQS.iter() {
                g.read_sequence(s.chars());
            }
        })
    }
    #[bench]
    fn bench_read_from(b: &mut Bencher) {
        b.iter(|| {
            let mut g = SequenceGraph::<char>::new();
            for &s in SEQS.iter() {
                g.read_from(s.chars());
            }
        })
    }
}
