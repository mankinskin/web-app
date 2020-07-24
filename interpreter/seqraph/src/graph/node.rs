use petgraph::{
    graph::{
        EdgeIndex,
    },
};
use std::fmt::{
    Debug,
};
use std::default::Default;

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
}
impl Default for EdgeMapping {
    fn default() -> Self {
        Self::new()
    }
}

pub trait NodeData : Debug + PartialEq + Clone
{
}
impl<T: Debug + PartialEq + Clone> NodeData for T {}

#[derive(PartialEq, Debug)]
pub struct NodeWeight<N: NodeData>  {
    pub data: N,
    pub mapping: EdgeMapping,
}
impl<N: NodeData> NodeWeight<N> {
    pub fn new(data: N) -> Self {
        Self {
            data,
            mapping: Default::default(),
        }
    }
}
