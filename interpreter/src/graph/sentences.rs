use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::ops::{Deref, DerefMut};
use std::fmt::{self, Debug, Formatter};

use crate::text::*;
use crate::graph::*;

pub enum SentenceGraphWeight {
    Empty,
}
impl<'a> Debug for SentenceGraphWeight {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "")
    }
}
pub type InternalSentenceGraph<'a> = DiGraph<Sentence<'a>, SentenceGraphWeight>;
pub struct SentenceGraph<'a> {
    graph: InternalSentenceGraph<'a>,
    root: NodeIndex,
}

impl<'a> SentenceGraph<'a> {
    pub fn from_sentence(sentence: Sentence<'a>) -> Self {
        let mut graph = DiGraph::new();
        let root = Self::build_succ_graph(&mut graph, sentence).unwrap();
        Self {
            graph,
            root,
        }
    }

    fn build_pred_graph(g: &mut InternalSentenceGraph<'a>, sentence: Sentence<'a>) -> Option<NodeIndex> {
        if *sentence.iter().clone().next()?.weight() == TextElement::Empty {
            return None;
        }
        let root = g.add_node(sentence.clone());
        let preds = sentence.predecessors()?;
        for p in preds {
            let mut new_sentence = sentence.clone();
            new_sentence.push_front(p.into());

            if let Some(index) = Self::build_pred_graph(g, new_sentence) {
                g.add_edge(index, root, SentenceGraphWeight::Empty);
            }
        }
        Some(root)
    }
    fn build_succ_graph(g: &mut InternalSentenceGraph<'a>, sentence: Sentence<'a>) -> Option<NodeIndex> {
        let root = g.add_node(sentence.clone());
        let succs = sentence.successors()?;
        for s in succs {
            let mut new_sentence = sentence.clone();
            new_sentence.push(s.into());

            if let Some(index) = Self::build_succ_graph(g, new_sentence) {
                g.add_edge(root, index, SentenceGraphWeight::Empty);
            }
        }
        Some(root)
    }
    pub fn write_to_file<S: Into<String>>(&self, name: S) -> std::io::Result<()> {
        std::fs::write(
            name.into() + ".dot",
            format!("{:?}", Dot::new(&self.graph)))
    }
}

impl<'a> From<Sentence<'a>> for SentenceGraph<'a> {
    fn from(s: Sentence<'a>) -> Self {
        Self::from_sentence(s)
    }
}

impl<'a> std::ops::Deref for SentenceGraph<'a> {
    type Target = InternalSentenceGraph<'a>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<'a> std::ops::DerefMut for SentenceGraph<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[derive(Clone)]
pub struct Sentence<'a> {
    stack: Vec<GraphNode<'a>>,
    graph: &'a TextGraph
}

impl<'a> Debug for Sentence<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}",
               self
                .stack
                .iter()
                .fold(String::new(),
                |acc, n| acc + " " + &n.weight().to_string())
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
    pub fn predecessors(&'a self) -> Option<Vec<GraphNode<'a>>> {
        let mut iter = self.stack.iter();
        let root = self.graph.find_node(iter.next()?)?;
        let mut preds = root.predecessors().collect::<HashSet<_>>();
        for (i, element) in iter.enumerate() {
            let node = self.graph.find_node(element)?;
            let node_preds = node.neighbors_incoming_with_distance(&(i+2)).collect::<HashSet<_>>();
            preds = preds.intersection(&node_preds)
                .map(Clone::clone)
                .collect::<HashSet<_>>();
        }
        Some(preds.iter().map(|i| GraphNode::new(self.graph, *i)).collect())
    }
    pub fn successors(&'a self) -> Option<Vec<GraphNode<'a>>> {
        let len = self.stack.len();
        let mut iter = self.stack.iter();
        let root = self.graph.find_node(iter.next()?)?;
        let mut succs = root.neighbors_outgoing_with_distance(&len).collect::<HashSet<_>>();
        for (i, element) in iter.enumerate() {
            let node = self.graph.find_node(element)?;
            let node_succs = node.neighbors_outgoing_with_distance(&(len - (i+1))).collect::<HashSet<_>>();
            succs = succs.intersection(&node_succs)
                .map(Clone::clone)
                .collect::<HashSet<_>>();
        }
        Some(succs.iter().map(|i| GraphNode::new(self.graph, *i)).collect())
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
}
