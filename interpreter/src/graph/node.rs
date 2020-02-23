use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use crate::graph::edges::*;
use crate::graph::*;

#[derive(Debug, Clone)]
pub struct GraphNode<'a>  {
    graph: &'a TextGraph,
    index: NodeIndex,
}
impl<'a> std::ops::Deref for GraphNode<'a> {
    type Target = TextElement;
    fn deref(&self) -> &Self::Target {
        &self.graph[self.index]
    }
}
impl<'a> From<&'a GraphNode<'a>> for &'a TextElement {
    fn from(n: &'a GraphNode<'a>) -> Self {
        &n
    }
}
impl<'a> Into<NodeIndex> for GraphNode<'a> {
    fn into(self) -> NodeIndex {
        self.index
    }
}
use std::fmt::{self, Display, Formatter};
impl<'a> Display for GraphNode<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let node_weight = self.weight();
        writeln!(f, "## {}", node_weight);

        /// Incoming
        let incoming_edges = self.incoming_edges();
        let max_incoming_distance = incoming_edges.max_weight().unwrap_or(0);
        let incoming_weight_groups = incoming_edges.clone().group_by_weight();
        let incoming_groups_counts: Vec<usize> = incoming_weight_groups.iter().map(|group|
            group.clone().count()
        ).collect();
        let incoming_node_groups: Vec<Vec<_>> = incoming_weight_groups.iter().map(|group|
            group.clone().map(|edge| self.graph.get_node(edge.source())).collect::<Vec<_>>()
        ).collect();
        writeln!(f, "Incoming edges:\n\
            \tcount: {},\n\
            \tmax distance: {},\n\
            \tgroup counts: {:?}",
            incoming_edges.count(),
            max_incoming_distance,
            incoming_groups_counts);


        /// Outgoing
        let outgoing_edges = self.outgoing_edges();
        let max_outgoing_distance = outgoing_edges.max_weight().unwrap_or(0);

        let outgoing_weight_groups = outgoing_edges.clone().group_by_weight();
        let outgoing_groups_counts: Vec<usize> = outgoing_weight_groups.iter().map(|group|
            group.clone().count()
        ).collect();
        let outgoing_node_groups: Vec<Vec<_>> = outgoing_weight_groups.iter().map(|group|
            group.clone().map(|edge| self.graph.get_node(edge.target())).collect()
        ).collect();
        writeln!(f, "Outgoing edges:\n\
            \tcount: {},\n\
            \tmax distance: {},\n\
            \tgroup counts: {:?}",
            outgoing_edges.count(),
            max_outgoing_distance,
            outgoing_groups_counts)
    }
}

impl<'a> GraphNode<'a> {
    pub fn new(graph: &'a TextGraph, index: NodeIndex) -> Self {
        GraphNode {
            graph,
            index,
        }
    }
    pub fn weight(&'a self) -> &'a TextElement {
        <&TextElement>::from(self)
    }
    pub fn is_at_distance(&'a self, other: GraphNode<'a>, distance: usize) -> bool {
        self.graph
            .find_edge(self.index, other.into())
            .map(|e| self.graph.edge_weight(e).map(|w| w.contains(&distance)).unwrap())
            .unwrap_or(false)
    }

    pub fn weight(&'a self) -> &'a TextElement {
        <&TextElement>::from(self)
    }
    pub fn edges_directed(&self, d: Direction) -> GraphEdges<'a> {
        self.graph.edges_directed(self.index, d).into()
    }
    pub fn incoming_edges(&self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Incoming)
    }
    pub fn outgoing_edges(&self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Outgoing)
    }
}
