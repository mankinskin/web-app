#[macro_use] extern crate itertools;
extern crate petgraph;
extern crate pretty_assertions;
#[macro_use] extern crate lazy_static;
extern crate nalgebra;

pub mod edge;
pub mod node;

use petgraph::{
    Direction,
    graph::{
        DiGraph,
        EdgeIndex,
        NodeIndex,
        EdgeReference,
    },
    dot::{
        Dot,
    },
    visit::{
        EdgeRef
    },
};
use std::fmt::{
    Debug,
};
use node::{
    NodeData,
    NodeWeight,
};
use edge::{
    EdgeData,
};
use std::ops::{
    Deref,
    DerefMut,
};
#[derive(Debug)]
pub struct Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    graph: DiGraph<NodeWeight<N>, E>,
}
impl<'a, N, E> Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }

    pub fn add_edge(&'a mut self, li: NodeIndex, ri: NodeIndex, w: E) -> EdgeIndex {
        let e = self.find_edge(li, ri, &w);
        let ei = if let Some(i) = e {
            i
        } else {
            self.graph.add_edge(li, ri, w)
        };
        ei
    }
    fn find_node(&'a self, element: &N) -> Option<NodeIndex> {
        self.graph
            .node_indices()
            .find(|i| self.graph[*i].data == *element)
            .map(|i| i.clone())
    }
    pub fn find_edge(&'a self, li: NodeIndex, ri: NodeIndex, w: &E) -> Option<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .find(|e| *e.weight() == *w)
            .map(|e| e.id())
    }
    pub fn find_nodes(&'a self, elems: &[N]) -> Option<Vec<NodeIndex>> {
        elems.iter().map(|e| self.find_node(e)).collect()
    }
    pub fn add_node(&'a mut self, element: N) -> NodeIndex {
        if let Some(i) = self.find_node(&element) {
            i
        } else {
            self.graph.add_node(
                NodeWeight::new(element)
            )
        }
    }
    pub fn contains(&self, element: &N) -> bool {
        self.find_node(element).is_some()
    }
    //pub fn contains_seq(&self, seq: &[N]) -> bool {
    //    TextPath::from_text(&self, text.clone()).is_some()
    //}
    //pub fn get_edges_directed(&'a self, index: NodeIndex, d: Direction) -> GraphEdges<'a> {
    //    GraphEdges::new(
    //        self.graph
    //            .edges_directed(index, d)
    //            .map(|e| self.get_edge(e.id()))
    //            .collect::<HashSet<_>>().iter().cloned()
    //    )
    //}
    //pub fn get_edges_incoming(&'a self, index: NodeIndex) -> GraphEdges<'a> {
    //    self.get_edges_directed(index, Direction::Incoming)
    //}
    //pub fn get_edges_outgoing(&'a self, index: NodeIndex) -> GraphEdges<'a> {
    //    self.get_edges_directed(index, Direction::Outgoing)
    //}
    //pub fn get_edges(&'a self, index: NodeIndex) -> GraphEdges<'a> {
    //    let edges = self.get_edges_incoming(index).into_iter()
    //        .chain(self.get_edges_outgoing(index));
    //    GraphEdges::new(edges)
    //}
    //pub fn get_text_path(&'a self, nodes: Vec<Node<'a>>) -> Option<TextPath<'a>> {
    //    TextPath::from_nodes(nodes)
    //}
    //pub fn find_text_path(&'a self, elems: Vec<TextElement>) -> Option<TextPath<'a>> {
    //    TextPath::from_elements(self, elems)
    //}
}
impl<N: NodeData, E: EdgeData> Deref for Graph<N, E> {
    type Target = DiGraph<NodeWeight<N>, E>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData, E: EdgeData> DerefMut for Graph<N, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}
#[derive(Debug)]
pub struct SequenceGraph<N>
    where N: NodeData,
{
    graph: Graph<N, usize>,
}
impl<'a, N> SequenceGraph<N>
    where N: NodeData,
{
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }
    pub fn read_sequence(&'a mut self, seq: &[N]) {
        for index in 0..seq.len() {
            self.read_sequence_element(seq, index);
        }
    }
    pub fn read_sequence_element(&'a mut self, seq: &[N], index: usize) {
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
    pub fn insert_element_neighborhood(&'a mut self,
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
impl<N: NodeData> Deref for SequenceGraph<N> {
    type Target = Graph<N, usize>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData> DerefMut for SequenceGraph<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_node() {
        let mut g = SequenceGraph::new();
        g.add_node('a');
        g.add_node('b');
        g.add_node('c');
        assert!(g.contains(&'a'));
        assert!(g.contains(&'b'));
        assert!(g.contains(&'c'));
        assert!(!g.contains(&'d'));
        assert!(!g.contains(&'e'));
    }
}
