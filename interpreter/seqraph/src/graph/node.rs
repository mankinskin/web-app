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
        Display,
    },
    collections::{
        HashSet,
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
    pub fn new() -> Self {
        Self {
            matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }
    pub fn add_incoming_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| *e == edge) {
            i
        } else {
            self.incoming.push(edge);
            self.matrix = self.matrix.clone().insert_column(self.matrix.ncols(), false.into());
            self.incoming.len() - 1
        }
    }
    pub fn add_outgoing_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| *e == edge) {
            i
        } else {
            self.outgoing.push(edge);
            self.matrix = self.matrix.clone().insert_row(self.matrix.nrows(), false.into());
            self.outgoing.len() - 1
        }
    }
    pub fn add_transition(&mut self, left_edge: EdgeIndex, right_edge: EdgeIndex) {
        let left_index = self.add_incoming_edge(left_edge);
        let right_index = self.add_outgoing_edge(right_edge);
        self.matrix[(right_index, left_index)] = true.into();
    }
    pub fn incoming_targets<N: NodeData, E: EdgeData>(&self, graph: &Graph<N, E>) -> Vec<(E, NodeIndex)> {
        self.incoming
            .iter()
            .filter_map(|i|
                Some((
                    graph.edge_weight(*i).map(Clone::clone)?,
                    graph.edge_endpoints(*i)
                        .map(|(source, _)| source.clone())?
                ))
            )
            .collect()
    }
    pub fn outgoing_targets<N: NodeData, E: EdgeData>(&self, graph: &Graph<N, E>) -> Vec<(E, NodeIndex)> {
        self.outgoing
            .iter()
            .filter_map(|i|
                Some((
                    graph.edge_weight(*i).map(Clone::clone)?,
                    graph.edge_endpoints(*i)
                        .map(|(_, target)| target.clone())?
                ))
            )
            .collect()
    }
    fn group_by_distance(es: Vec<(usize, NodeIndex)>) -> Vec<Vec<NodeIndex>> {
        let max = es.iter().map(|(d, _)| d.clone()).max().unwrap_or(0);
        let mut r = Vec::new();
        for i in 1..=max {
            r.push(
                es.clone()
                    .into_iter()
                    .filter(|(d, _)| *d == i)
                    .map(|(_, n)| n.clone())
                    .collect()
            )
        }
        r
    }
    pub fn incoming_groups<N: NodeData>(&self, graph: &Graph<N, usize>) -> Vec<Vec<NodeWeight<N>>> {
        Self::group_by_distance(self.incoming_targets(graph))
            .iter()
            .map(|set|
                set.iter()
                   .filter_map(|i| graph.node_weight(*i))
                   .cloned()
                   .collect()
            )
            .collect()
    }
    pub fn outgoing_groups<N: NodeData>(&self, graph: &Graph<N, usize>) -> Vec<Vec<NodeWeight<N>>> {
        Self::group_by_distance(self.outgoing_targets(graph))
            .iter()
            .map(|set|
                set.iter()
                   .filter_map(|i| graph.node_weight(*i))
                   .cloned()
                   .collect()
            )
            .collect()
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
