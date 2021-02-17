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
    Node,
    Token,
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

/// Graph of N: NodeData + Mappable mapping possible distances
/// between nodes to prefix and postfix nodes
#[derive(Debug)]
pub struct SequenceGraph<N>
where
    N: NodeData + Mappable,
{
    graph: Graph<Node<N>, usize>,
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
            '*' => Token::Start,
            '#' => Token::End,
            _ => Token::Element(<T as Into<N>>::into(sym)),
        };
        self.get_node_info(&sym)
    }
    pub fn read_sequence<T: Into<N>, I: Iterator<Item = T>>(&mut self, seq: I) {
        let seq = N::sequenced(seq);
        for index in 0..seq.len() {
            self.read_to_node(&seq[..], index);
        }
    }
    //pub fn knows_sequence(&self, seq: &[Token<N>]) -> bool {
    //	let mappings: Option<Vec<Node<N>>> =
    //		seq.iter().map(|sym| self.find_node_weight(sym))
    //		.collect();
    //	if let Some(mappings) = mappings {
    //		mappings
    //			.iter()
    //			.fold(
    //	} else { return false; }
    //}
    fn read_to_node(&mut self, seq: &[Token<N>], index: usize) {
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
    fn insert_node_neighborhood(
        &mut self,
        l: Token<N>, // left-hand element
        ld: usize,   // distance to left-hand element
        x: Token<N>, // center element
        rd: usize,   // distance to right-hand element
        r: Token<N>, // right-hand element
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
    fn groups_to_string(groups: Vec<Vec<Node<N>>>) -> String {
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
    fn map_to_tokens(groups: Vec<Vec<Node<N>>>) -> Vec<Vec<Token<N>>> {
        groups
            .iter()
            .map(|g| g.iter().map(|m| m.token.clone()).collect())
            .collect()
    }
    pub fn get_node_info<T: PartialEq<Node<N>>>(&self, element: &T) -> Option<NodeInfo<N>> {
        let node = self.find_node_weight(element)?;
        let mut incoming_groups: Vec<Vec<Node<N>>> = node.mapping().incoming_distance_groups(&self);
        incoming_groups.reverse();
        let outgoing_groups: Vec<Vec<Node<N>>> = node.mapping().outgoing_distance_groups(&self);
        Some(NodeInfo {
            element: node.token,
            incoming_groups: Self::map_to_tokens(incoming_groups),
            outgoing_groups: Self::map_to_tokens(outgoing_groups),
        })
    }
    ///// Join two EdgeMappings to a new EdgeMapping
    //pub fn join_mappings(&self, lhs: &Node<N>, rhs: &Node<N>) -> Option<Node<N>> {
    //	// TODO: make lhs and rhs contain indices
    //	//let left_index = self.find_node_index(&lhs.token)?;
    //	//let right_index = self.find_node_index(&rhs.token)?;
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
    //	let left_width = lhs.token.width();
    //	let left_incoming_groups = lhs.mapping().incoming_distance_groups(&self);
    //	let right_incoming_groups = rhs.mapping().incoming_distance_groups(&self);

    //	// intersect left outgoing groups i + right.width with right outgoing groups i
    //	let right_width = rhs.token.width();
    //	let left_outgoing_groups = lhs.mapping().outgoing_distance_groups(&self);
    //	let right_outgoing_groups = rhs.mapping().outgoing_distance_groups(&self);
    //	//
    //	Some(lhs.clone())
    //}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo<N: NodeData> {
    pub element: Token<N>,
    pub incoming_groups: Vec<Vec<Token<N>>>,
    pub outgoing_groups: Vec<Vec<Token<N>>>,
}
impl<N: NodeData + Mappable> Deref for SequenceGraph<N> {
    type Target = Graph<Node<N>, usize>;
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
    lazy_static::lazy_static! {
        static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c']);
        static ref SEQS: Vec<&'static str> = Vec::from(["abc", "abb", "bcb"]);
        static ref EDGES: Vec<(Token<char>, Token<char>, usize)> = {
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
            assert!(G.has_node_edge(l, r, w), "({}, {}, {})", l, r, w);
        }
    }
}
