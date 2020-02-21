use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::collections::{HashSet, HashMap};

use crate::text::*;
mod edges;
use crate::graph::edges::*;
mod node;
use crate::graph::node::*;
mod nodes;
use crate::graph::nodes::*;

#[derive(Debug)]
pub struct TextGraph  {
    graph: DiGraph<TextElement, HashSet<usize>>,
}
impl std::ops::Deref for TextGraph {
    type Target = DiGraph<TextElement, HashSet<usize>>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl std::ops::DerefMut for TextGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}
impl Into<DiGraph<TextElement, HashSet<usize>>> for TextGraph {
    fn into(self) -> DiGraph<TextElement, HashSet<usize>> {
        self.graph
    }
}
impl TextGraph {
    pub fn new() -> Self {
        let mut n = Self {
            graph: DiGraph::new(),
        };
        // TODO should use enum_iterator with is_stop()
        // All stop symbols could be followed by empty
        //n.add_edge(
        //    &TextElement::Punctuation(Punctuation::Dot),
        //    &TextElement::Empty,
        //    1
        //);
        n
    }
    pub fn contains(&self, element: &TextElement) -> bool {
        self.get_node_index(element).is_some()
    }
    pub fn get_node_index(&self, element: &TextElement) -> Option<NodeIndex> {
        self.graph.node_indices()
            .find(|i| self.graph[*i] == *element)
            .map(|i| i.clone())
    }
    pub fn get_node(&self, index: NodeIndex) -> GraphNode {
        GraphNode::new(
            &self,
            index
        )
    }
    pub fn find_node(&self, element: &TextElement) -> Option<GraphNode> {
        self.get_node_index(element).map(|i|
            self.get_node(i)
        )
    }
    pub fn insert_elements(&mut self, l: &TextElement, r: &TextElement, distance: usize) {
        if l.is_stop() {
            self.add_edge(&TextElement::Empty, r, distance);
        } else {
            self.add_edge(l, r, distance);
        }
    }
    pub fn insert_text(&mut self, text: Text) {
        let len = text.len();
        if len > 0 {
            self.insert_elements(&TextElement::Empty, &text[0], 1);
        }
        let mut next_stop = 0;
        for i in 0..len-1 {
            if i == next_stop {
                // search for next stop symbol
                // to stop counting distance between elements
                while {
                    next_stop += 1;
                    next_stop < len && !text[next_stop].is_stop()
                }
                { }
                //continue;
            }
            for j in (i+1)..=next_stop {
                let left = &text[i];
                let right = &text[j];
                self.insert_elements(left, right, j-i);
            }
        }
    }
    pub fn add(&mut self, element: &TextElement) -> NodeIndex {
        match self.get_node_index(element) {
            Some(i) => i,
            None => {
                self.graph.add_node(element.clone())
            }
        }
    }
    pub fn add_edge(&mut self, l: &TextElement, r: &TextElement, distance: usize) {
        let li = self.add(l);
        let ri = self.add(r);
        let old_edge = self.graph.find_edge(li, ri);
        match old_edge {
            Some(i) => {
                self.graph.edge_weight_mut(i).unwrap().insert(distance);
            },
            None => {
                let mut new = HashSet::new();
                new.insert(distance);
                self.graph.update_edge(li, ri, new);
            }
        }
    }
    pub fn element_info(&self, element: &TextElement) {
        match self.find_node(element) {
            Some(n) => n.info(),
            None => {}
        }
    }
}
