use crate::{
    arithmetic_bool::ArithmeticBool,
    token::{
        ContextLink,
        ContextMapping,
    },
};
use petgraph::graph::{
    EdgeIndex,
    NodeIndex,
};
use std::default::Default;
#[allow(unused)]
use tracing::debug;
#[derive(PartialEq, Clone, Debug, Eq)]
pub struct LoadedEdge {
    pub index: EdgeIndex,
    pub node: NodeIndex,
    pub dist: usize,
}
impl LoadedEdge {
    pub fn new(index: EdgeIndex, node: NodeIndex, dist: usize) -> Self {
        Self { index, node, dist }
    }
}
impl ContextLink for LoadedEdge {
    fn index(&self) -> &EdgeIndex {
        &self.index
    }
}

pub type MappingMatrix<T> = nalgebra::DMatrix<T>;

pub type EdgeMapping = MatrixMapping<EdgeIndex>;
pub type LoadedEdgeMapping = MatrixMapping<LoadedEdge>;
pub type EdgeMappingMatrix = MappingMatrix<ArithmeticBool>;

#[derive(PartialEq, Clone, Debug, Eq)]
pub struct MatrixMapping<E: ContextLink> {
    pub matrix: EdgeMappingMatrix,
    pub incoming: Vec<E>,
    pub outgoing: Vec<E>,
}
impl<E: ContextLink> ContextMapping<E> for MatrixMapping<E> {
    fn incoming(&self) -> &Vec<E> {
        &self.incoming
    }
    fn outgoing(&self) -> &Vec<E> {
        &self.outgoing
    }
}
impl<E: ContextLink> MatrixMapping<E> {
    pub fn new() -> Self {
        Self {
            matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
    pub fn remove_zero_columns(&mut self) {
        let (incoming, columns): (_, Vec<_>) = self
            .incoming
            .clone()
            .into_iter()
            .zip(self.matrix.column_iter())
            .filter(|(_, col)| col.iter().any(|b| b.0))
            .unzip();
        self.incoming = incoming;
        self.matrix = EdgeMappingMatrix::from_columns(&columns);
    }
    pub fn remove_zero_rows(&mut self) {
        let (outgoing, rows): (_, Vec<_>) = self
            .outgoing
            .clone()
            .into_iter()
            .zip(self.matrix.row_iter())
            .filter(|(_, row)| row.iter().any(|b| b.0))
            .unzip();
        self.outgoing = outgoing;
        self.matrix = EdgeMappingMatrix::from_rows(&rows);
    }
    /// Add an incoming edge
    fn add_incoming_edge(&mut self, edge: E) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| edge.index() == e.index()) {
            //debug!("Incoming edge already exists {:#?}", edge);
            i
        } else {
            let index = self.incoming.len();
            //debug!("Adding new incoming edge {:#?} with index {}", edge, index);
            self.incoming.push(edge);
            //debug!("{:#?}", self.incoming);
            self.matrix = self
                .matrix
                .clone()
                .insert_column(self.matrix.ncols(), false.into());
            index
        }
    }
    /// Add an outgoing edge
    fn add_outgoing_edge(&mut self, edge: E) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| edge.index() == e.index()) {
            //debug!("Outgoing edge already exists {:#?}", edge);
            i
        } else {
            let index = self.outgoing.len();
            //debug!("Adding new outgoing edge {:#?} with index {}", edge, index);
            self.outgoing.push(edge);
            //debug!("{:#?}", self.outgoing);
            self.matrix = self
                .matrix
                .clone()
                .insert_row(self.matrix.nrows(), false.into());
            index
        }
    }
    /// Add a transition between two edges
    pub fn add_transition(&mut self, l: E, r: E) {
        let li = self.add_incoming_edge(l);
        let ri = self.add_outgoing_edge(r);
        self.matrix[(ri, li)] = true.into();
    }
}
impl<'a> MatrixMapping<LoadedEdge> {
    /// Get weights and sources of incoming edges
    pub fn incoming_sources(&'a self) -> impl Iterator<Item = (usize, NodeIndex)> + 'a {
        self.incoming.iter().map(|e| (e.dist, e.node))
    }
    /// Get weights and targets of outgoing edges
    pub fn outgoing_targets(&'a self) -> impl Iterator<Item = (usize, NodeIndex)> + 'a {
        self.outgoing.iter().map(|e| (e.dist, e.node))
    }
}
impl<T: ContextLink> Default for MatrixMapping<T> {
    fn default() -> Self {
        Self::new()
    }
}
