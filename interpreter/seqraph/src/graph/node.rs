use petgraph::{
    graph::{
        EdgeIndex,
        NodeIndex,
    },
};
use std::{
    fmt::{
        self,
        Debug,
    },
};
use std::default::Default;
use crate::{
    graph::{
        Graph,
        edge::{
            EdgeData,
        },
    },
};

pub type EdgeMappingMatrix =
    nalgebra::Matrix<
        bool,
        nalgebra::Dynamic,
        nalgebra::Dynamic,
        nalgebra::VecStorage<
            bool,
            nalgebra::Dynamic,
            nalgebra::Dynamic
        >
    >;

#[derive(PartialEq, Clone, Debug)]
pub struct EdgeMapping {
    pub matrix: EdgeMappingMatrix,
    pub outgoing: Vec<EdgeIndex>,
    pub incoming: Vec<EdgeIndex>,
}
impl<'a> EdgeMapping {
    /// New EdgeMapping
    pub fn new() -> Self {
        Self {
            matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }
    /// Add an incoming edge
    fn add_incoming_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| *e == edge) {
            i
        } else {
            self.incoming.push(edge);
            self.matrix = self.matrix.clone().insert_column(self.matrix.ncols(), false.into());
            self.incoming.len() - 1
        }
    }
    /// Add an outgoing edge
    fn add_outgoing_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| *e == edge) {
            i
        } else {
            self.outgoing.push(edge);
            self.matrix = self.matrix.clone().insert_row(self.matrix.nrows(), false.into());
            self.outgoing.len() - 1
        }
    }
    /// Add a transition between two edges
    pub fn add_transition(&mut self, left_edge: EdgeIndex, right_edge: EdgeIndex) {
        let left_index = self.add_incoming_edge(left_edge);
        let right_index = self.add_outgoing_edge(right_edge);
        self.matrix[(right_index, left_index)] = true.into();
    }
    /// Get weights and sources of incoming edges
    pub fn incoming_sources<N: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<N, E>
        ) -> impl Iterator<Item=(E, NodeIndex)> + 'a {
        graph.edge_weights(self.incoming.iter().cloned())
            .zip(graph.edge_sources(self.incoming.iter().cloned()))
    }
    /// Get weights and targets of outgoing edges
    pub fn outgoing_targets<N: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<N, E>) -> impl Iterator<Item=(E, NodeIndex)> + 'a {
        graph.edge_weights(self.outgoing.iter().cloned())
            .zip(graph.edge_targets(self.outgoing.iter().cloned()))
    }

    /// Get distance groups for incoming edges
    pub fn incoming_distance_groups<N: NodeData>(&self, graph: &Graph<N, usize>) -> Vec<Vec<NodeWeight<N>>> {
        graph.group_weights_by_distance(self.incoming_sources(graph))
    }
    /// Get distance groups for outgoing edges
    pub fn outgoing_distance_groups<N: NodeData>(&self, graph: &Graph<N, usize>) -> Vec<Vec<NodeWeight<N>>> {
        graph.group_weights_by_distance(self.outgoing_targets(graph))
    }
}
impl Default for EdgeMapping {
    fn default() -> Self {
        Self::new()
    }
}

pub trait NodeData : Debug + PartialEq + Clone {}
impl<T: Debug + PartialEq + Clone> NodeData for T {}

#[derive(PartialEq, Clone)]
pub struct NodeWeight<N: NodeData>  {
    pub data: N,
    pub mapping: EdgeMapping,
}
impl<N: NodeData> Debug for NodeWeight<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
impl<N: NodeData> NodeWeight<N> {
    pub fn new(data: N) -> Self {
        Self {
            data,
            mapping: Default::default(),
        }
    }
}
