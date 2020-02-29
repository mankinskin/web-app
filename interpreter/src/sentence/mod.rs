mod graph;
pub use graph::*;

use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::ops::{Deref, DerefMut};
use std::fmt::{self, Debug, Display, Formatter};

use crate::*;
use crate::text::*;
use crate::graph::*;
use std::collections::{HashMap, HashSet};
use std::iter::{FromIterator};

#[derive(Clone)]
pub struct Sentence<'a> {
    stack: Vec<GraphNode<'a>>,
    graph: &'a TextGraph
}

impl<'a> Debug for Sentence<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl<'a> Display for Sentence<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}",
               self
                .stack
                .iter()
                .fold(String::new(),
                |acc, n| acc + &n.weight().to_string() + " ")
                .trim_end()
               )
    }
}
impl<'a> Deref for Sentence<'a> {
    type Target = Vec<GraphNode<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}
impl<'a> DerefMut for Sentence<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stack
    }
}
impl<'a> Sentence<'a> {
    pub fn new(graph: &'a TextGraph, v: Vec<TextElement>) -> Option<Self> {
        let mut stack = Vec::new();
        stack.reserve(v.len());
        for e in &v {
            stack.push(graph.find_node(e)?);
        }
        Some(Self {
            stack,
            graph
        })
    }
    pub fn new_empty(graph: &'a TextGraph) -> Self {
        Self {
            stack: Vec::new(),
            graph
        }
    }
    pub fn push(&mut self, node: NodeIndex) {
        self.stack.push(self.graph.get_node(node));
    }
    pub fn push_front(&mut self, node: NodeIndex) {
        let node = self.graph.get_node(node);
        let mut tmp = vec![node];
        tmp.extend(self.stack.clone());
        self.stack = tmp;
    }
    pub fn edges_incoming(&'a self) -> HashMap<GraphEdge<'a>, HashSet<usize>> {
        let mut iter = self.stack.iter();
        if iter.len() < 1 {
            return HashMap::new();
        }
        // iterator over each grouping in the stack
        // a grouping contains all of the edges of a node, grouped
        // by weight
        let mut groupings = iter
            .map(|node| self.graph.get_edges_incoming(node.index()))
            .map(|edges| edges.group_by_weight());

        let root: Vec<Vec<GraphEdge<'a>>> =
            groupings
                .next()
                .unwrap()
                // grouping of first element
                .iter()
                .map(|edges| {
                    edges.iter().map(|edge| {
                        edge.clone()
                        //(edge.clone(), edge.weight().clone())
                    }).collect()
                })
                .collect();

        groupings
            .enumerate()
            // i is the index of the element
            .fold(root, |acc_groupings, (i, groups)| {
                // fold all groupings into one grouping
                acc_groupings
                    .iter()
                    .zip(groups.iter().skip(i + 1))
                    .map(move |(acc_group, elem_group)| {
                        // acc_group is of distance i
                        // elem_group is of distance i + 1
                        acc_group
                            .iter()
                            .filter_map(|acc_edge| {
                                elem_group
                                    .iter()
                                    .find(|elem_edge| {
                                        elem_edge.source() ==
                                            acc_edge.source()
                                    })
                                    //.map(|elem_edge| {
                                    //    acc_edge.1
                                    //        .intersection(elem_edge.weight())
                                    //        .map(Clone::clone)
                                    //        .collect::<HashSet<_>>()
                                    //})
                                    //.map(|weights| {
                                    //    if weights.len() > 0 {
                                    //        Some((acc_edge.0.clone(), weights))
                                    //    } else {
                                    //        None
                                    //    }
                                    //})
                                    //.flatten()
                            })
                            .map(Clone::clone)
                            .collect()
                    }).collect()
            })
            .iter()
            .map(Clone::clone)
            .enumerate()
            .fold(HashMap::new(), |mut acc, (d, edges)| {
                for edge in edges {
                    acc.entry(edge)
                        .or_insert(HashSet::new())
                        .insert(d);
                }
                acc
            })
    }
    pub fn neighbors_incoming(&'a self) -> HashSet<GraphNode<'a>> {
        self.edges_incoming()
            .iter()
            .map(|(e, w)| GraphNode::new(self.graph, e.source().clone()))
            .collect()
    }
    pub fn edges_incoming_with_distance(&'a self, d: usize) -> HashMap<GraphEdge<'a>, HashSet<usize>> {
        self.edges_incoming()
            .iter()
            .filter(|(e, w)| w.contains(&d))
            .map(|(e, w)| (e.clone(), w.clone()))
            .collect()
    }
    pub fn neighbors_incoming_with_distance(&'a self, d: usize) -> HashSet<GraphNode<'a>> {
        self.edges_incoming_with_distance(d)
            .iter()
            .map(|(e, w)| GraphNode::new(self.graph, e.source().clone()))
            .collect()
    }
    pub fn predecessors(&'a self) -> HashSet<GraphNode<'a>> {
        self.neighbors_incoming_with_distance(1)
    }
    pub fn edges_outgoing(&'a self) -> HashMap<GraphEdge<'a>, HashSet<usize>> {
        let mut iter = self.stack.iter();
        let len = iter.len();
        if len < 1 {
            return HashMap::new();
        }
        // iterator over each grouping in the stack
        // a grouping contains all of the edges of a node, grouped
        // by weight
        let mut element_groupings = iter
            .map(|node| self.graph.get_edges_outgoing(node.index()))
            .map(|edges| edges.group_by_weight());

        let root: Vec<Vec<GraphEdge<'a>>> =
            element_groupings
                .next()
                .unwrap()
                // grouping of first element
                .iter()
                .skip(len - 1)
                .map(|edges| {
                    edges.iter().map(|edge| {
                        edge.clone()
                    }).collect()
                })
                .collect();

        element_groupings
            .enumerate()
            // i is the index of the element
            .fold(root, |acc_groupings, (i, groups)| {
                // fold all groupings into one grouping
                acc_groupings
                    .iter()
                    .zip(groups.iter().skip((len - 2) - i))
                    .map(move |(acc_group, elem_group)| {
                        // j is the distance of the group
                        // acc_group is of distance i
                        // elem_group is of distance i + 1
                        acc_group
                            .iter()
                            .filter_map(|acc_edge| {
                                elem_group
                                    .iter()
                                    .find(|elem_edge| {
                                        elem_edge.target() ==
                                            acc_edge.target()
                                    })
                                    //.map(|elem_edge| {
                                    //    acc_edge.1
                                    //        .intersection(elem_edge.weight())
                                    //        .map(Clone::clone)
                                    //        .collect::<HashSet<_>>()
                                    //})
                                    .map(|weights| {
                                        acc_edge.clone()
                                    })
                                    //.flatten()
                            })
                            //.map(Clone::clone)
                            .collect()
                    }).collect()
            })
            .iter()
            .map(Clone::clone)
            .enumerate()
            .fold(HashMap::new(), |mut acc, (d, edges)| {
                for edge in edges {
                    acc.entry(edge)
                        .or_insert(HashSet::new())
                        .insert(d);
                }
                acc
            })
    }
    pub fn neighbors_outgoing(&'a self) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing()
            .iter()
            .map(|(e, w)| GraphNode::new(self.graph, e.target().clone()))
            .collect()
    }
    pub fn edges_outgoing_with_distance(&'a self, d: usize) -> HashMap<GraphEdge<'a>, HashSet<usize>> {
        self.edges_outgoing()
            .iter()
            .filter(|(e, w)| w.contains(&d))
            .map(|(e, w)| (e.clone(), w.clone()))
            .collect()
    }
    pub fn neighbors_outgoing_with_distance(&'a self, d: usize) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing_with_distance(d)
            .iter()
            .map(|(e, w)| GraphNode::new(self.graph, e.target().clone()))
            .collect()
    }
    pub fn successors(&'a self) -> HashSet<GraphNode<'a>> {
        self.neighbors_outgoing_with_distance(1)
    }
}

mod tests {

    use crate::*;
    use crate::graph::*;
    use crate::text::*;
    use pretty_assertions::{assert_eq};
    #[test]
    fn test_sentence() {
        let mut tg = TextGraph::new();
        tg.insert_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));
        tg.write_to_file("test_graph");

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        let dot = tg.find_node(&(Punctuation::Dot.into())).unwrap();

        let a_sentence = tg.get_sentence(vec![
            Word::from("A").into(),
        ]).unwrap();
        let b_sentence = tg.get_sentence(vec![
            Word::from("B").into(),
        ]).unwrap();
        let ab = tg.get_sentence(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
        let bc = tg.get_sentence(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let bcd = tg.get_sentence(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();

        let mut a_preds = a_sentence.neighbors_incoming();
        let mut a_succs = a_sentence.neighbors_outgoing();
        assert_eq!(a_preds, set![
            empty.clone(),
            b.clone(),
            d.clone(),
            a.clone()
        ]);
        assert_eq!(a_succs, set![
            b.clone(),
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            dot.clone()
        ]);

        let mut b_preds = b_sentence.neighbors_incoming();
        let mut b_succs = b_sentence.neighbors_outgoing();
        assert_eq!(b_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(b_succs, set![
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            dot.clone()
        ]);

        let mut ab_preds = ab.neighbors_incoming();
        let mut ab_succs = ab.neighbors_outgoing();
        assert_eq!(ab_preds, set![
            empty.clone()
        ]);
        assert_eq!(ab_succs, set![
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            dot.clone()
        ]);

        let mut bc_preds = bc.neighbors_incoming();
        let mut bc_succs = bc.neighbors_outgoing();
        assert_eq!(bc_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(bc_succs, set![
            d.clone(),
            e.clone(),
            dot.clone()
        ]);

        let mut bcd_preds = bcd.neighbors_incoming();
        let mut bcd_succs = bcd.neighbors_outgoing();
        assert_eq!(bcd_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(bcd_succs, set![
            e.clone(),
            dot.clone()
        ]);
    }
}
