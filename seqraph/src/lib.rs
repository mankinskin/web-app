extern crate itertools;
extern crate petgraph;
extern crate pretty_assertions;
#[allow(unused_imports)] // only used in tests
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate serde;
extern crate serde_json;

pub mod graph;
pub mod mapping;
//pub mod grammar;

use graph::{
    node::NodeData,
    Graph,
};
use mapping::{
    Mappable,
    Mapped,
    Sequenced,
    Symbol,
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

#[derive(Debug)]
pub struct SequenceGraph<N>
where
    N: NodeData + Mappable,
{
    graph: Graph<Symbol<N>, usize>,
}
impl<N> SequenceGraph<N>
where
    N: NodeData + Mappable,
{
    pub fn new() -> Self {
        let graph = Graph::new();
        Self { graph }
    }
    pub fn query<T: Into<N> + Into<char> + Clone, I: Iterator<Item = T> + Clone>(
        &self,
        seq: I,
    ) -> Option<NodeInfo<N>> {
        let sym = seq.clone().next().unwrap();
        let sym = match <T as Into<char>>::into(sym.clone()) {
            '*' => Sequenced::Start,
            '#' => Sequenced::End,
            _ => Sequenced::Element(<T as Into<N>>::into(sym)),
        };
        self.get_node_info(&sym)
    }
    pub fn read_sequence<T: Into<N>, I: Iterator<Item = T>>(&mut self, seq: I) {
        let seq = N::sequenced(seq);
        for index in 0..seq.len() {
            self.read_sequence_element(&seq[..], index);
        }
    }
    //pub fn knows_sequence(&self, seq: &[Sequenced<N>]) -> bool {
    //	let mappings: Option<Vec<Symbol<N>>> =
    //		seq.iter().map(|sym| self.find_node_weight(sym))
    //		.collect();
    //	if let Some(mappings) = mappings {
    //		mappings
    //			.iter()
    //			.fold(
    //	} else { return false; }
    //}
    fn read_sequence_element(&mut self, seq: &[Sequenced<N>], index: usize) {
        let element = &seq[index];
        let end = seq.len() - 1;
        for pre in 0..index {
            let l = &seq[pre];
            let ld = index - pre;
            for succ in (index + 1)..=end {
                let r = &seq[succ];
                let rd = succ - index;
                self.insert_element_neighborhood(l.clone(), ld, element.clone(), rd, r.clone());
            }
        }
    }
    fn insert_element_neighborhood(
        &mut self,
        l: Sequenced<N>,
        ld: usize,
        x: Sequenced<N>,
        rd: usize,
        r: Sequenced<N>,
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
            .add_transition(le, re);
    }
    #[allow(unused)]
    fn groups_to_string(groups: Vec<Vec<Symbol<N>>>) -> String {
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
                    format!("{}{} ", a, elem.clone().unwrap_or(String::new()))
                })
            )
        })
    }
    fn map_to_data(groups: Vec<Vec<Symbol<N>>>) -> Vec<Vec<Sequenced<N>>> {
        groups
            .iter()
            .map(|g| g.iter().map(|m| m.data.clone()).collect())
            .collect()
    }
    pub fn get_node_info<T: PartialEq<Symbol<N>>>(&self, element: &T) -> Option<NodeInfo<N>> {
        let node = self.find_node_weight(element)?;
        let mut incoming_groups: Vec<Vec<Symbol<N>>> =
            node.mapping().incoming_distance_groups(&self);
        incoming_groups.reverse();
        let outgoing_groups: Vec<Vec<Symbol<N>>> = node.mapping().outgoing_distance_groups(&self);
        Some(NodeInfo {
            element: node.data,
            incoming_groups: Self::map_to_data(incoming_groups),
            outgoing_groups: Self::map_to_data(outgoing_groups),
        })
    }
    ///// Join two EdgeMappings to a new EdgeMapping
    //pub fn join_mappings(&self, lhs: &Symbol<N>, rhs: &Symbol<N>) -> Option<Symbol<N>> {
    //	// TODO: make lhs and rhs contain indices
    //	//let left_index = self.find_node_index(&lhs.data)?;
    //	//let right_index = self.find_node_index(&rhs.data)?;
    //	let left_outgoing = &lhs.mapping().outgoing;
    //	let right_incoming = &rhs.mapping().incoming;

    //	// find all edges connecting left to right with their indices
    //	// in the matrices
    //	let connecting_edges: Vec<(usize, usize, EdgeIndex)> = left_outgoing
    //		.iter()
    //		.enumerate()
    //		.filter_map(|(li, e)| Some((li, right_incoming.iter().position(|r| r == e)?, e.clone())))
    //		.collect();

    //	// take left rows and right columns of matrix for connecting edges
    //	let left_matrix = &lhs.mapping().matrix;
    //	let right_matrix = &rhs.mapping().matrix;

    //	//let incoming_context = left_matrix.row(left_matrix_index);
    //	//let outgoing_context = right_matrix.column(right_matrix_index);

    //	// intersect left incoming groups i with right incoming groups i + left.width
    //	let left_width = lhs.data.width();
    //	let left_incoming_groups = lhs.mapping().incoming_distance_groups(&self);
    //	let right_incoming_groups = rhs.mapping().incoming_distance_groups(&self);

    //	// intersect left outgoing groups i + right.width with right outgoing groups i
    //	let right_width = rhs.data.width();
    //	let left_outgoing_groups = lhs.mapping().outgoing_distance_groups(&self);
    //	let right_outgoing_groups = rhs.mapping().outgoing_distance_groups(&self);
    //	//
    //	Some(lhs.clone())
    //}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo<N: NodeData> {
    pub element: Sequenced<N>,
    pub incoming_groups: Vec<Vec<Sequenced<N>>>,
    pub outgoing_groups: Vec<Vec<Sequenced<N>>>,
}
impl<N: NodeData + Mappable> Deref for SequenceGraph<N> {
    type Target = Graph<Symbol<N>, usize>;
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
    lazy_static! {
        static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c']);
        static ref SEQS: Vec<&'static str> = Vec::from(["abc", "abb", "bcb"]);
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
