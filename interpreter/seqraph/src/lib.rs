extern crate itertools;
extern crate petgraph;
extern crate pretty_assertions;
#[macro_use] extern crate lazy_static;
extern crate nalgebra;

pub mod graph;

use std::fmt::{
    self,
    Debug,
    Display,
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
pub trait Sequencable: NodeData {
    fn start() -> Self;
    fn end() -> Self;
    fn wrap_sequence<I: Iterator<Item=Self>>(seq: I) -> Vec<Self> {
        let mut v: Vec<Self> = vec!(Self::start());
        v.extend(seq);
        v.push(Self::end());
        v
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Sequenced<T: NodeData> {
    Element(T),
    Start,
    End,
}
impl<T: NodeData + Display> Display for Sequenced<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Sequenced::Element(t) => t.to_string(),
            Sequenced::Start => "START".to_string(),
            Sequenced::End => "END".to_string(),
        })
    }
}
impl<T: NodeData> Sequencable for Sequenced<T> {
    fn start() -> Self {
        Sequenced::Start
    }
    fn end() -> Self {
        Sequenced::End
    }
}
impl From<char> for Sequenced<char> {
    fn from(c: char) -> Self {
        Sequenced::Element(c)
    }
}
#[derive(Debug)]
pub struct SequenceGraph<N>
    where N: NodeData + Sequencable,
{
    graph: Graph<N, usize>,
}
impl<'a, N> SequenceGraph<N>
    where N: NodeData + Sequencable,
{
    pub fn new() -> Self {
        let graph = Graph::new();
        Self {
            graph,
        }
    }
    pub fn read_sequence<T: Into<N> + Clone>(&'a mut self, seq: &[T]) {
        let seq = N::wrap_sequence(seq.iter().cloned().map(Into::into));
        for index in 0..seq.len() {
            self.read_sequence_element(&seq[..], index);
        }
    }
    fn read_sequence_element(&'a mut self, seq: &[N], index: usize) {
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
    fn insert_element_neighborhood(&'a mut self,
        l: N,
        ld: usize,
        x: N,
        rd: usize,
        r: N) {
        let li = self.add_node(l);
        let xi = self.add_node(x);
        let ri = self.add_node(r);
        let le = self.add_edge(li, xi, ld);
        let re = self.add_edge(xi, ri, rd);
        self.graph.node_weight_mut(xi)
            .unwrap()
            .mapping
            .add_transition(le, re);
    }
}
impl<N: NodeData + Sequencable> Deref for SequenceGraph<N> {
    type Target = Graph<N, usize>;
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
    use pretty_assertions::{
        assert_eq,
    };
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
        static ref G: SequenceGraph<Sequenced<char>> = {
            let mut g = SequenceGraph::new();
            for &s in SEQS.iter() {
                g.read_sequence(&s.chars().collect::<Vec<_>>()[..]);
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
