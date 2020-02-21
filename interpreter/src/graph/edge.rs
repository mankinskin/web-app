use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use std::collections::{HashSet, HashMap};

pub type EdgeRef<'a> = EdgeReference<'a, HashSet<usize>>;

#[derive(Clone, Debug)]
pub struct GraphEdge<'a>  {
    edge: EdgeReference<'a, HashSet<usize>>,
}
impl<'a> std::ops::Deref for GraphEdge<'a> {
    type Target = EdgeRef<'a>;
    fn deref(&self) -> &Self::Target {
        &self.edge
    }
}
impl<'a> From<EdgeRef<'a>> for GraphEdge<'a>  {
    fn from(edge: EdgeRef<'a>) -> Self {
        Self {
            edge
        }
    }
}
impl<'a> GraphEdge<'a>  {
    pub fn contains_weight(&self, w: &usize) -> bool {
        self.weight().contains(w)
    }
    pub fn max_weight(&self) -> Option<&'a usize> {
        self.weight().iter().max()
    }
}

