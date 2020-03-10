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
                |acc, n| acc + &n.weight().element().to_string() + " ")
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
        let len = iter.len();
        if len < 1 {
            return HashMap::new();
        }
        //println!("edges_incoming(sentence: \"{}\")", self);
        // iterator over each grouping in the stack
        // a grouping contains all of the edges of a node, grouped
        // by weight
        let mut element_groupings = iter
            .map(|node| self.graph.get_edges_incoming(node.index()))
            .map(|edges| edges.group_by_weight())
            .collect::<Vec<_>>();

        //println!("element_groupings: {:#?}", element_groupings);
        let root: Vec<Vec<GraphEdge<'a>>> =
            element_groupings
                .iter()
                .next()
                .unwrap()
                // grouping of first element
                .iter()
                // skipped to distance behind sentence
                .map(|edges| Vec::from_iter(edges.clone()))
                .collect();

        //println!("root: {:#?}", root);

        let result = element_groupings
            .iter()
            .skip(1)
            .enumerate()
            // i is the index of elements after the first element
            .fold(root, |acc_grouping, (i, grouping)| {
                // fold all groupings into one grouping
                //println!("folding element {}: {:#?}", i, grouping);
                let distance_offset = i + 1;

                acc_grouping // accumulated groupings 1..n
                    .iter()
                    .zip(grouping.iter().skip(distance_offset))
                    .map(move |(acc_group, elem_group)| {
                        //println!("merging group {:#?} into {:#?}", elem_group, acc_group);
                        let res = acc_group
                            .iter()
                            .filter(move |edge| {
                                elem_group
                                    .iter()
                                    .find(move |e| e.source() == edge.source())
                                    .is_some()
                            })
                            .map(|e| e.clone())
                            .collect();
                        //println!("resulting group {:#?}", res);
                        res
                    }).collect()
            })
            .iter()
            .map(Clone::clone)
            .enumerate()
            .fold(HashMap::new(), |mut acc, (d, edges)| {
                for edge in edges {
                    acc.entry(edge)
                        .or_insert(HashSet::new())
                        .insert(d+1);
                }
                acc
            });
        //println!("result: {:#?}", result);
        result
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
        //println!("edges_outgoing(sentence: \"{}\")", self);
        // iterator over each grouping in the stack
        // a grouping contains all of the edges of a node, grouped
        // by weight
        let mut element_groupings = iter
            .map(|node| self.graph.get_edges_outgoing(node.index()))
            .map(|edges| edges.group_by_weight())
            .collect::<Vec<_>>();

        //println!("element_groupings: {:#?}", element_groupings);
        let root: Vec<Vec<GraphEdge<'a>>> =
            element_groupings
                .iter()
                .next()
                .unwrap()
                // grouping of first element
                .iter()
                .skip(len - 1)
                // skipped to distance behind sentence
                .map(|edges| Vec::from_iter(edges.clone()))
                .collect();

        //println!("root: {:#?}", root);

        let result = element_groupings
            .iter()
            .skip(1)
            .enumerate()
            // i is the index of elements after the first element
            .fold(root, |acc_grouping, (i, grouping)| {
                // fold all groupings into one grouping
                //println!("folding element {}: {:#?}", i, grouping);
                let element_position = i + 2;
                let distance_offset = len - element_position;

                acc_grouping // accumulated groupings 1..n
                    .iter()
                    .zip(grouping.iter().skip(distance_offset))
                    .map(move |(acc_group, elem_group)| {
                        //println!("merging group {:#?} into {:#?}", elem_group, acc_group);
                        let res = acc_group
                            .iter()
                            .filter(move |edge| {
                                elem_group
                                    .iter()
                                    .find(move |e| e.target() == edge.target())
                                    .is_some()
                            })
                            .map(|e| e.clone())
                            .collect();
                        //println!("resulting group {:#?}", res);
                        res
                    }).collect()
            })
            .iter()
            .map(Clone::clone)
            .enumerate()
            .fold(HashMap::new(), |mut acc, (d, edges)| {
                for edge in edges {
                    acc.entry(edge)
                        .or_insert(HashSet::new())
                        .insert(d+1);
                }
                acc
            });
        //println!("result: {:#?}", result);
        result
    }
    pub fn neighbors_outgoing(&'a self) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing()
            .iter()
            .map(|(e, w)| GraphNode::new(self.graph, e.target().clone()))
            .collect()
    }
    pub fn edges_outgoing_with_distance(&'a self, d: usize) -> HashSet<GraphEdge<'a>> {
        self.edges_outgoing()
            .iter()
            .filter(|(e, w)| w.contains(&d))
            .map(|(e, w)| e.clone())
            .collect()
    }
    pub fn neighbors_outgoing_with_distance(&'a self, d: usize) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing_with_distance(d)
            .iter()
            .map(|e| GraphNode::new(self.graph, e.target().clone()))
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
    fn direct_neighbors() {
        let mut tg = TextGraph::new();
        tg.insert_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));
        tg.write_to_file("graphs/test_graph");

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        let dot = tg.find_node(&(Punctuation::Dot.into())).unwrap();

        let empty_a_sentence = tg.get_sentence(vec![
            TextElement::Empty,
            Word::from("A").into(),
        ]).unwrap();
        let empty_a_preds = empty_a_sentence.predecessors();
        let empty_a_succs = empty_a_sentence.successors();
        //println!("{:#?}", empty_a_succs);
        assert_eq!(empty_a_preds, set![]);
        assert_eq!(empty_a_succs, set![
            b.clone(),
            a.clone()
        ]);
        let a_sentence = tg.get_sentence(vec![
            Word::from("A").into(),
        ]).unwrap();
        let a_preds = a_sentence.predecessors();
        let a_succs = a_sentence.successors();
        assert_eq!(a_preds, set![
            empty.clone(),
            d.clone(),
            a.clone()
        ]);
        assert_eq!(a_succs, set![
            a.clone(),
            b.clone(),
            c.clone(),
            dot.clone()
        ]);

        let b_sentence = tg.get_sentence(vec![
            Word::from("B").into(),
        ]).unwrap();
        let b_preds = b_sentence.predecessors();
        let b_succs = b_sentence.successors();
        assert_eq!(b_preds, set![
            a.clone()
        ]);
        assert_eq!(b_succs, set![
            c.clone(),
            d.clone()
        ]);

        let ab = tg.get_sentence(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
        let ab_preds = ab.predecessors();
        let ab_succs = ab.successors();
        assert_eq!(ab_preds, set![
            empty.clone()
        ]);
        assert_eq!(ab_succs, set![
            c.clone(),
            d.clone()
        ]);

        let bc = tg.get_sentence(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let bc_preds = bc.predecessors();
        let bc_succs = bc.successors();
        assert_eq!(bc_preds, set![
            a.clone()
        ]);
        assert_eq!(bc_succs, set![
            d.clone()
        ]);

        let bcd = tg.get_sentence(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();
        let bcd_preds = bcd.predecessors();
        let bcd_succs = bcd.successors();
        assert_eq!(bcd_preds, set![
            a.clone()
        ]);
        assert_eq!(bcd_succs, set![
            e.clone()
        ]);

        let aa = tg.get_sentence(vec![
            Word::from("A").into(),
            Word::from("A").into()
        ]).unwrap();
        let aa_preds = aa.predecessors();
        let aa_succs = aa.successors();
        assert_eq!(aa_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(aa_succs, set![
            a.clone(),
            c.clone(),
            dot.clone()
        ]);
    }
    #[test]
    fn neighbors() {
        let mut tg = TextGraph::new();
        tg.insert_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));
        tg.write_to_file("graphs/test_graph");

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        let dot = tg.find_node(&(Punctuation::Dot.into())).unwrap();


        let b_sentence = tg.get_sentence(vec![
            Word::from("B").into(),
        ]).unwrap();
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

        let a_sentence = tg.get_sentence(vec![
            Word::from("A").into(),
        ]).unwrap();
        let mut a_preds = a_sentence.neighbors_incoming();
        let mut a_succs = a_sentence.neighbors_outgoing();
        assert_eq!(a_preds, set![
            empty.clone(),
            b.clone(),
            d.clone(),
            a.clone()
        ]);

        let empty_a_sentence = tg.get_sentence(vec![
            TextElement::Empty,
            Word::from("A").into(),
        ]).unwrap();
        let mut empty_a_preds = empty_a_sentence.neighbors_incoming();
        let mut empty_a_succs = empty_a_sentence.neighbors_outgoing();
        assert_eq!(empty_a_preds, set![]);
        assert_eq!(empty_a_succs, set![
            b.clone(),
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            dot.clone()
        ]);
        assert_eq!(a_succs, set![
            b.clone(),
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            dot.clone()
        ]);
        let ab = tg.get_sentence(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
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

        let bc = tg.get_sentence(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
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

        let bcd = tg.get_sentence(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();
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
