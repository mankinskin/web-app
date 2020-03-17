use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use std::collections::{HashSet, HashMap};
use std::fmt::{self, Debug, Display, Formatter};
use petgraph::visit::{EdgeRef as PetgraphEdgeRef};
use std::hash::{self, Hash, Hasher};
use crate::graph::*;


#[derive(PartialEq)]
pub struct TextGraphEdgeWeight  {
    distance: usize,
}
impl TextGraphEdgeWeight {
    pub fn new(distance: usize) -> Self {
        Self {
            distance,
        }
    }

    pub fn distance(&self) -> usize {
        self.distance
    }
}
use std::cmp::{Ord, PartialOrd, Ordering};
impl PartialOrd<usize> for TextGraphEdgeWeight {
    fn partial_cmp(&self, distance: &usize) -> Option<Ordering> {
        self.distance.partial_cmp(distance)
    }
}
impl PartialEq<usize> for TextGraphEdgeWeight {
    fn eq(&self, distance: &usize) -> bool {
        self.distance == *distance
    }
}
impl Debug for TextGraphEdgeWeight {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.distance)
    }
}
impl std::ops::Deref for TextGraphEdgeWeight {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.distance
    }
}
impl std::ops::DerefMut for TextGraphEdgeWeight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.distance
    }
}

pub type EdgeRef<'a> = EdgeReference<'a, TextGraphEdgeWeight>;

#[derive(Clone, Copy)]
pub struct GraphEdge<'a>  {
    graph: &'a TextGraph,
    index: EdgeIndex,
}

impl<'a> GraphEdge<'a>  {
    pub fn new(graph: &'a TextGraph, index: EdgeIndex) -> Self {
        let r = graph.edge_references().nth(index.index()).unwrap();
        Self {
            graph,
            index,
        }
    }
    pub fn index(&'a self) -> &'a EdgeIndex {
        &self.index
    }
    pub fn edge_ref(&'a self) -> EdgeRef<'a> {
        self.graph
            .edge_references()
            .nth(self.index.index())
            .unwrap()
    }
}
impl<'a> PartialEq for GraphEdge<'a> {
    fn eq(&self, other: &GraphEdge<'a>) -> bool {
        self.index == other.index &&
            self.graph as *const _ == other.graph
    }
}
impl<'a> Debug for GraphEdge<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl<'a> Display for GraphEdge<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?} --{:?}--> {:?}",
               self.source(),
               self.weight(),
               self.target())
    }
}
impl<'a> Hash for GraphEdge<'a> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.id().hash(h);
    }
}
impl<'a> Eq for GraphEdge<'a> {
}
impl<'a> petgraph::visit::EdgeRef for GraphEdge<'a> {
    type NodeId = <EdgeRef<'a> as petgraph::visit::EdgeRef>::NodeId;
    type EdgeId = <EdgeRef<'a> as petgraph::visit::EdgeRef>::EdgeId;
    type Weight = <EdgeRef<'a> as petgraph::visit::EdgeRef>::Weight;
    fn source(&self) -> Self::NodeId {
        self.edge_ref().source()
    }
    fn target(&self) -> Self::NodeId {
        self.edge_ref().target()
    }
    fn weight(&self) -> &Self::Weight {
        self.edge_ref().weight()
    }
    fn id(&self) -> Self::EdgeId {
        self.edge_ref().id()
    }
}
