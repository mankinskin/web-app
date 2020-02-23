use crate::graph::edges::*;
use crate::graph::*;

#[derive(Debug, Clone)]
pub struct GraphNodes<'a>  {
    graph: &'a TextGraph,
    indices: HashSet<NodeIndex>,
}

use std::collections::hash_set::*;
impl<'a> GraphNodes<'a> {
    pub fn new(graph: &'a TextGraph) -> Self {
        Self {
            graph,
            indices: HashSet::new()
        }
    }
    pub fn add(&mut self, node: NodeIndex)  {
        self.indices.insert(node);
    }

    fn intersect_neighbors<F: Fn(&NodeIndex) -> HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::NodeId>>(&'a self, node_selector: F) -> Vec<NodeIndex> {
            self.indices.iter()
                .map(node_selector)
            .fold(None, |intersection: Option<HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::NodeId>>,
                         neighbors: HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::NodeId>|
                Some(match intersection {
                    None => neighbors,
                    Some(set) => set.intersection(&neighbors).map(Clone::clone).collect::<HashSet<_>>()
                })
            ).unwrap_or(HashSet::new())
            .iter()
            .map(Clone::clone)
            .collect()
    }
    fn intersect_edges<F: Fn(&NodeIndex) -> HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::EdgeId>>(&'a self, edge_selector: F) -> Vec<GraphEdge<'a>> {
            self.indices.iter()
                .map(edge_selector)
            .fold(None, |intersection: Option<HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::EdgeId>>,
                         edges: HashSet<<GraphEdge<'a> as petgraph::visit::EdgeRef>::EdgeId>|
                Some(match intersection {
                    None => edges,
                    Some(set) => set.intersection(&edges).map(Clone::clone).collect::<HashSet<_>>()
                })
            ).unwrap_or(HashSet::new())
            .iter()
            .flat_map(move |id| self.graph.edge_references().nth(id.index()))
            .map(GraphEdge::from)
            .collect()
    }
    pub fn edges(&'a self) -> Vec<GraphEdge<'a>> {
        self.intersect_edges(move |node_index: &NodeIndex|
                self.graph
                    .edges(*node_index)
                    .map(|e| e.id())
                    .collect::<HashSet<_>>())
    }
    pub fn edges_directed(&'a self, d: Direction) -> Vec<GraphEdge<'a>> {
        self.intersect_edges(move |node_index: &NodeIndex|
                self.graph
                    .edges_directed(*node_index, d)
                    .map(|e| e.id())
                    .collect::<HashSet<_>>())
    }
    pub fn incoming_edges(&'a self) -> Vec<GraphEdge<'a>> {
        self.edges_directed(Direction::Incoming)
    }
    pub fn outgoing_edges(&'a self) -> Vec<GraphEdge<'a>> {
        self.edges_directed(Direction::Outgoing)
    }
    pub fn edges_with_distance(&'a self, distance: &'a usize) -> Vec<GraphEdge<'a>> {
        self.edges().iter().filter(move |e| e.weight().contains(distance)).map(Clone::clone).collect()
    }
    pub fn edges_with_distance_directed(&'a self, distance: &'a usize, direction: Direction)
        -> Vec<GraphEdge<'a>> {
        self.edges_directed(direction).iter()
            .filter(move |e| e.weight().contains(distance))
            .map(Clone::clone)
            .collect()
    }
    pub fn incoming_edges_with_distance(&'a self, distance: &'a usize) -> Vec<GraphEdge<'a>> {
        self.edges_with_distance_directed(distance, Direction::Incoming)
    }
    pub fn outgoing_edges_with_distance(&'a self, distance: &'a usize) -> Vec<GraphEdge<'a>> {
        self.edges_with_distance_directed(distance, Direction::Outgoing)
    }

    pub fn neighbors(&'a self) -> Vec<NodeIndex> {
        self.intersect_neighbors(move |node_index: &NodeIndex|
                self.graph
                    .neighbors(*node_index)
                    .collect::<HashSet<_>>())
    }
    pub fn neighbors_directed(&'a self, direction: Direction) -> Vec<NodeIndex> {
        self.intersect_neighbors(move |node_index: &NodeIndex|
                self.graph
                    .neighbors_directed(*node_index, direction)
                    .collect::<HashSet<_>>())
    }
    pub fn neighbors_incoming(&'a self) -> Vec<NodeIndex> {
        self.neighbors_directed(Direction::Incoming)
    }
    pub fn neighbors_outgoing(&'a self) -> Vec<NodeIndex> {
        self.neighbors_directed(Direction::Outgoing)
    }
    pub fn neighbors_with_distance(&'a self, distance: &'a usize) -> Vec<NodeIndex> {
        self.neighbors_incoming_with_distance(distance)
            .iter()
            .chain(self.neighbors_outgoing_with_distance(distance).iter())
            .map(Clone::clone)
            .collect()
    }
    pub fn neighbors_incoming_with_distance(&'a self, distance: &'a usize) -> Vec<NodeIndex> {
        self.incoming_edges_with_distance(distance)
            .iter()
            .map(|e| e.source())
            .collect()
    }
    pub fn neighbors_outgoing_with_distance(&'a self, distance: &'a usize) -> Vec<NodeIndex> {
        self.incoming_edges_with_distance(distance)
            .iter()
            .map(|e| e.source())
            .collect()
    }
}
