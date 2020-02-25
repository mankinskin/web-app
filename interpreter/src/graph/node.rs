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

    pub fn edges(&'a self) -> GraphEdges<'a> {
        GraphEdges::from(self.graph.edges(self.index))
    }
    pub fn edges_directed(&'a self, d: Direction) -> GraphEdges<'a> {
        GraphEdges::from(self.graph.edges_directed(self.index, d))
    }
    pub fn incoming_edges(&'a self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Incoming)
    }
    pub fn outgoing_edges(&'a self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Outgoing)
    }
    pub fn edges_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges().filter(move |e| e.weight().contains(distance))
    }
    pub fn edges_with_distance_directed(&'a self, distance: &'a usize, direction: Direction)
        -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_directed(direction)
            .filter(move |e| e.weight().contains(distance))
    }
    pub fn incoming_edges_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_with_distance_directed(distance, Direction::Incoming)
    }
    pub fn outgoing_edges_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_with_distance_directed(distance, Direction::Outgoing)
    }

    pub fn neighbors(&'a self) -> Vec<NodeIndex> {
        self.graph.neighbors(self.index).collect()
    }
    pub fn neighbors_directed(&'a self, direction: Direction) -> impl Iterator<Item=NodeIndex> + 'a {
        self.graph.neighbors_directed(self.index, direction)
    }
    pub fn neighbors_incoming(&'a self) -> impl Iterator<Item=NodeIndex> + 'a {
        self.neighbors_directed(Direction::Incoming)
    }
    pub fn neighbors_outgoing(&'a self) -> impl Iterator<Item=NodeIndex> + 'a {
        self.neighbors_directed(Direction::Outgoing)
    }
    pub fn neighbors_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=NodeIndex> + 'a {
        self.neighbors_incoming_with_distance(distance)
            .chain(self.neighbors_outgoing_with_distance(distance))
    }
    pub fn neighbors_incoming_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=NodeIndex> + 'a {
        self.incoming_edges_with_distance(distance)
            .map(|e| e.source())
    }
    pub fn neighbors_outgoing_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=NodeIndex> + 'a {
        self.outgoing_edges_with_distance(distance)
            .map(|e| e.target())
    }
    pub fn predecessors(&'a self) -> impl Iterator<Item=NodeIndex> + 'a {
        self.neighbors_incoming_with_distance(&1)
    }
    pub fn successors(&'a self) -> impl Iterator<Item=NodeIndex> + 'a {
        self.neighbors_outgoing_with_distance(&1)
    }
}
