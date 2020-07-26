extern crate itertools;
extern crate petgraph;
extern crate pretty_assertions;
#[allow(unused_imports)] // only used in tests
#[macro_use] extern crate lazy_static;
extern crate nalgebra;

pub mod graph;
pub mod mapping;

use petgraph::{
    graph::{
        EdgeIndex,
    },
};
use mapping::{
    Mapped,
    Mappable,
    Sequenced,
    Wide,
};
use std::{
    fmt::{
        Debug,
    },
};
use graph::{
    Graph,
    node::{
        NodeData,
    },
};
use std::ops::{
    Deref,
    DerefMut,
};
#[derive(Debug)]
pub struct SequenceGraph<N>
    where N: NodeData + Mappable,
{
    graph: Graph<Mapped<N>, usize>,
}
impl<N> SequenceGraph<N>
    where N: NodeData + Mappable,
{
    pub fn new() -> Self {
        let graph = Graph::new();
        Self {
            graph,
        }
    }
    pub fn query<T: Into<N>, I: Iterator<Item=T> + Clone>(&self, seq: I) -> Option<String> {
        let sym = seq.clone().next().unwrap();
        self.get_node_info(&Sequenced::from(sym.into() as N))
    }
    pub fn read_sequence<T: Into<N>, I: Iterator<Item=T>>(&mut self, seq: I) {
        let seq = N::sequenced(seq);
        for index in 0..seq.len() {
            self.read_sequence_element(&seq[..], index);
        }
    }
    fn read_sequence_element(&mut self, seq: &[Sequenced<N>], index: usize) {
        let element = &seq[index];
        let end = seq.len()-1;
        for pre in 0..index {
            let l = &seq[pre];
            let ld = index-pre;
            for succ in (index+1)..=end {
                let r = &seq[succ];
                let rd = succ-index;
                self.insert_element_neighborhood(
                    l.clone(),
                    ld,
                    element.clone(),
                    rd,
                    r.clone());
            }
        }
    }
    fn insert_element_neighborhood(
        &mut self,
        l: Sequenced<N>,
        ld: usize,
        x: Sequenced<N>,
        rd: usize,
        r: Sequenced<N>
    ) {
        let li = self.add_node(l);
        let xi = self.add_node(x);
        let ri = self.add_node(r);
        let le = self.add_edge(li, xi, ld);
        let re = self.add_edge(xi, ri, rd);
        self.graph
            .node_weight_mut(xi)
            .unwrap()
            .mapping
            .add_transition(le, re);
    }
    pub fn get_node_info<T: PartialEq<Mapped<N>>>(&self, element: &T) -> Option<String> {
        let node = self.find_node_weight(element)?;
        let pre_groups: Vec<Vec<Mapped<N>>> = node.mapping.incoming_distance_groups(&self);
        let post_groups: Vec<Vec<Mapped<N>>> = node.mapping.outgoing_distance_groups(&self);
        Some(format!("Pre Groups: {:#?}\nPost Groups: {:#?}",
                     pre_groups, post_groups))
    }
}
impl<N: NodeData + Mappable> Deref for SequenceGraph<N> {
    type Target = Graph<Mapped<N>, usize>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData + Mappable> DerefMut for SequenceGraph<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    lazy_static!{
        static ref ELEMS: Vec<char> = {
            Vec::from(['a', 'b', 'c'])
        };
        static ref SEQS: Vec<&'static str> = {
            Vec::from([
                "abc",
                "abb",
                "bcb"
            ])
        };
        static ref EDGES: Vec<(Sequenced<char>, Sequenced<char>, usize)> = {
            Vec::from([
                (Sequenced::Start, 'a'.into(), 1),
                (Sequenced::Start, 'b'.into(), 1),
                (Sequenced::Start, 'b'.into(), 2),
                (Sequenced::Start, 'b'.into(), 3),
                (Sequenced::Start, 'c'.into(), 2),
                (Sequenced::Start, 'c'.into(), 3),

                ('a'.into(), Sequenced::End, 3),
                ('b'.into(), Sequenced::End, 3),
                ('b'.into(), Sequenced::End, 2),
                ('b'.into(), Sequenced::End, 1),
                ('c'.into(), Sequenced::End, 2),
                ('c'.into(), Sequenced::End, 1),

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
        static ref G: SequenceGraph<char> = {
            let mut g = SequenceGraph::new();
            for &s in SEQS.iter() {
                g.read_sequence(s.chars());
            }
            g
        };
    }
    #[test]
    fn has_read_seq() {
        G.write_to_file("seq_graph").unwrap();
        for (l, r, w) in EDGES.iter() {
            assert!(G.has_node_edge(l, r, w), format!("({}, {}, {})", l, r, w));
        }
    }
}
