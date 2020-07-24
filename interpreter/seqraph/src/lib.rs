extern crate itertools;
extern crate petgraph;
extern crate pretty_assertions;
#[allow(unused_imports)] // only used in tests
#[macro_use] extern crate lazy_static;
extern crate nalgebra;

pub mod graph;

use petgraph::{
    graph::{
        NodeIndex,
    },
};
use std::{
    fmt::{
        self,
        Debug,
        Display,
    },
    collections::{
        HashSet,
    },
};
use graph::{
    Graph,
    node::{
        NodeData,
        NodeWeight,
    },
};
use std::ops::{
    Deref,
    DerefMut,
};
pub trait Wide {
    fn width(&self) -> usize;
}
pub trait Sequencable: NodeData {
    fn sequenced<T: Into<Self>, I: Iterator<Item=T>>(seq: I) -> Vec<Sequenced<Self>> {
        let mut v = vec!(Sequenced::Start);
        v.extend(seq.map(|t| Sequenced::Element(t.into())));
        v.push(Sequenced::End);
        v
    }
}
impl<T: NodeData + Into<Sequenced<T>>> Sequencable for T {
}
#[derive(Debug, PartialEq, Clone)]
pub enum Sequenced<T: NodeData + Sequencable> {
    Element(T),
    Start,
    End,
}
impl<T: NodeData + Display + Sequencable> Display for Sequenced<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Sequenced::Element(t) => t.to_string(),
            Sequenced::Start => "START".to_string(),
            Sequenced::End => "END".to_string(),
        })
    }
}
impl<T: NodeData + Wide + Sequencable> Wide for Sequenced<T> {
    fn width(&self) -> usize {
        match self {
            Sequenced::Element(t) => t.width(),
            Sequenced::Start => 0,
            Sequenced::End => 0,
        }
    }
}
impl<N: NodeData> From<N> for Sequenced<N> {
    fn from(e: N) -> Self {
        Sequenced::Element(e)
    }
}
#[derive(Debug)]
pub struct SequenceGraph<N>
    where N: NodeData + Sequencable,
{
    graph: Graph<Sequenced<N>, usize>,
}
impl<N> SequenceGraph<N>
    where N: NodeData + Sequencable,
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
    fn insert_element_neighborhood(&mut self,
        l: Sequenced<N>,
        ld: usize,
        x: Sequenced<N>,
        rd: usize,
        r: Sequenced<N>) {
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
    pub fn get_node_info(&self, element: &Sequenced<N>) -> Option<String> {
        let node = self.get_node(element)?;
        let pre_groups: Vec<Vec<NodeWeight<Sequenced<N>>>> = node.mapping.incoming_groups(&self);
        let post_groups: Vec<Vec<NodeWeight<Sequenced<N>>>> = node.mapping.outgoing_groups(&self);
        Some(format!("Pre Groups: {:#?}\nPost Groups: {:#?}",
                     pre_groups, post_groups))
    }
}
impl<N: NodeData + Sequencable> Deref for SequenceGraph<N> {
    type Target = Graph<Sequenced<N>, usize>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData + Sequencable> DerefMut for SequenceGraph<N> {
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
        for (l, r, w) in EDGES.iter() {
            assert!(G.has_node_edge(l, r, w), format!("({}, {}, {})", l, r, w));
        }
    }
}
