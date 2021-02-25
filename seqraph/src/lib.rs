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
            self.read_to_node(&seq[..], index);
        }
    }
    fn read_to_node(&mut self, seq: &[Token<T>], index: usize) {
        let element = &seq[index];
        let len = seq.len();
        for pre in 0..index {
            let l = &seq[pre];
            let ld = index - pre;
            for post in (index + 1)..len {
                let r = &seq[post];
                let rd = post - index;
                self.insert_node_neighborhood(l.clone(), ld, element.clone(), rd, r.clone());
            }
        }
    }
    pub fn add_node(&mut self, token: Token<T>) -> NodeIndex {
        self.graph.add_node(&Node::new(token))
    }
    pub fn add_edge(&mut self, li: NodeIndex, ri: NodeIndex, w: usize) -> Edge {
        Edge::new(self.graph.add_edge(li, ri, w))
    }
    pub fn load_node<P: PartialEq<Node<T>> + Debug>(&self, p: P) -> Option<LoadedNode<T>> {
        let index = self.graph.find_node_index(p)?;
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
    fn insert_node_neighborhood(
        &mut self,
        l: Token<T>, // left-hand element
        ld: usize,   // distance to left-hand element
        x: Token<T>, // center element
        rd: usize,   // distance to right-hand element
        r: Token<T>, // right-hand element
    ) {
        let li = self.add_node(l.clone());
        let xi = self.add_node(x.clone());
        let ri = self.add_node(r.clone());
        let le = self.add_edge(li, xi, ld);
        let re = self.add_edge(xi, ri, rd);
        //debug!("Inserting node neighborhood {:?}({}) -{}> {:?}({}) -{}> {:?}({})",
        //l, li.index(), ld, x, xi.index(), rd, r, ri.index());
        self.node_weight_mut(xi)
            .unwrap()
            .mapping_mut()
            .add_transition(le, re);
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
    lazy_static::lazy_static! {
        pub static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c', 'd', 'e']);
        pub static ref SEQS: Vec<&'static str> = Vec::from([
            "bc",
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
    #[traced_test]
    #[test]
    fn read_sequence() {
        debug!(
            "{:#?}",
            G.node_indices()
                .zip(G.all_node_weights())
                .collect::<Vec<_>>()
        );
        let b_node = G.load_node('b').unwrap();
        let c_node = G.load_node('c').unwrap();
        let bm = b_node.mapping();
        assert_distances_match!("Incoming", G, bm.incoming_sources().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::Start),
            ]);
        assert_distances_match!("Outgoing", G, bm.outgoing_targets().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::Element('c')),
                (2, Token::End),
            ]);
        let cm = c_node.mapping();
        assert_distances_match!("Incoming", G, cm.incoming_sources().collect::<HashSet<_>>(),
            Token<char>,
            [
                (2, Token::Start),
                (1, Token::Element('b')),
            ]);
        assert_distances_match!("Outgoing", G, cm.outgoing_targets().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::End),
            ]);
    }
}
