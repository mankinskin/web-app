use crate::{
    graph::{
        edge::EdgeData,
        node::NodeData,
        Graph,
    },
    SequenceGraph,
};
use petgraph::graph::{
    EdgeIndex,
    NodeIndex,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::default::Default;
use std::fmt::{
    self,
    Debug,
    Display,
};
pub type EdgeMappingMatrix = nalgebra::Matrix<
    bool,
    nalgebra::Dynamic,
    nalgebra::Dynamic,
    nalgebra::VecStorage<bool, nalgebra::Dynamic, nalgebra::Dynamic>,
>;
#[derive(PartialEq, Clone, Debug)]
pub struct EdgeMapping {
    pub matrix: EdgeMappingMatrix,
    pub incoming: Vec<EdgeIndex>,
    pub outgoing: Vec<EdgeIndex>,
}
impl<'a> EdgeMapping {
    /// New EdgeMapping
    pub fn new() -> Self {
        Self {
            matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
    /// Add an incoming edge
    fn add_incoming_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| *e == edge) {
            i
        } else {
            self.incoming.push(edge);
            self.matrix = self
                .matrix
                .clone()
                .insert_column(self.matrix.ncols(), false.into());
            self.incoming.len() - 1
        }
    }
    /// Add an outgoing edge
    fn add_outgoing_edge(&mut self, edge: EdgeIndex) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| *e == edge) {
            i
        } else {
            self.outgoing.push(edge);
            self.matrix = self
                .matrix
                .clone()
                .insert_row(self.matrix.nrows(), false.into());
            self.outgoing.len() - 1
        }
    }
    /// Add a transition between two edges
    pub fn add_transition(&mut self, l: EdgeIndex, r: EdgeIndex) {
        let li = self.add_incoming_edge(l);
        let ri = self.add_outgoing_edge(r);
        self.matrix[(ri, li)] = true.into();
    }
    /// Get weights and sources of incoming edges
    pub fn incoming_sources<N: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<N, E>,
    ) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
        graph
            .edge_weights(self.incoming.iter())
            .zip(graph.edge_sources(self.incoming.iter()))
    }
    /// Get weights and targets of outgoing edges
    pub fn outgoing_targets<N: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<N, E>,
    ) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
        graph
            .edge_weights(self.outgoing.iter())
            .zip(graph.edge_targets(self.outgoing.iter()))
    }

    /// Get distance groups for incoming edges
    pub fn incoming_distance_groups<N: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<N>,
    ) -> Vec<Vec<Symbol<N>>> {
        graph.distance_group_source_weights(self.incoming.iter())
    }
    /// Get distance groups for outgoing edges
    pub fn outgoing_distance_groups<N: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<N>,
    ) -> Vec<Vec<Symbol<N>>> {
        graph.distance_group_target_weights(self.outgoing.iter())
    }
}
impl Default for EdgeMapping {
    fn default() -> Self {
        Self::new()
    }
}
pub trait Wide {
    fn width(&self) -> usize;
}
impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}

/// Type for storing elements of a sequence
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Sequenced<T: NodeData> {
    Element(T),
    Start,
    End,
}
impl<T: NodeData + Display> Display for Sequenced<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Sequenced::Element(t) => t.to_string(),
                Sequenced::Start => "START".to_string(),
                Sequenced::End => "END".to_string(),
            }
        )
    }
}
impl<T: NodeData + Wide> Wide for Sequenced<T> {
    fn width(&self) -> usize {
        match self {
            Sequenced::Element(t) => t.width(),
            Sequenced::Start => 0,
            Sequenced::End => 0,
        }
    }
}
impl<N: NodeData> From<N> for Sequenced<N> {
    fn from(e: N) -> Self {
        Sequenced::Element(e)
    }
}
impl<N: NodeData> PartialEq<Symbol<N>> for Sequenced<N> {
    fn eq(&self, rhs: &Symbol<N>) -> bool {
        *self == rhs.data
    }
}

/// Stores sequenced data with an edge map
#[derive(PartialEq, Clone)]
pub struct Symbol<N: NodeData> {
    pub data: Sequenced<N>,
    mapping: EdgeMapping,
}
impl PartialEq<Symbol<char>> for char {
    fn eq(&self, rhs: &Symbol<char>) -> bool {
        *self == rhs.data
    }
}
impl PartialEq<Sequenced<char>> for char {
    fn eq(&self, rhs: &Sequenced<char>) -> bool {
        match rhs {
            Sequenced::Element(e) => e == self,
            _ => false,
        }
    }
}
impl<N: NodeData> PartialEq<N> for Symbol<N> {
    fn eq(&self, rhs: &N) -> bool {
        self.data == *rhs
    }
}
impl<N: NodeData> PartialEq<N> for Sequenced<N> {
    fn eq(&self, rhs: &N) -> bool {
        match self {
            Sequenced::Element(e) => e == rhs,
            _ => false,
        }
    }
}
impl<N: NodeData> Debug for Symbol<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
impl<N: NodeData> Symbol<N> {
    pub fn new(data: N) -> Self {
        Self::from(Sequenced::from(data))
    }
}
impl<N: NodeData> From<Sequenced<N>> for Symbol<N> {
    fn from(data: Sequenced<N>) -> Self {
        Self {
            data,
            mapping: Default::default(),
        }
    }
}
impl<N: NodeData> From<N> for Symbol<N> {
    fn from(e: N) -> Self {
        Symbol::new(e)
    }
}
impl<T: NodeData + Display> Display for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
impl<T: NodeData + Wide> Wide for Symbol<T> {
    fn width(&self) -> usize {
        self.data.width()
    }
}
/// Trait for data that can be wrapped in a sequence
pub trait Sequencable: NodeData {
    fn sequenced<T: Into<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Sequenced<Self>> {
        let mut v = vec![Sequenced::Start];
        v.extend(seq.map(|t| Sequenced::Element(t.into())));
        v.push(Sequenced::End);
        v
    }
}
impl<T: NodeData + Into<Sequenced<T>>> Sequencable for T {}
/// Trait for data that can be mapped in a sequence
pub trait Mappable: Sequencable + Wide {}
impl<T: NodeData + Wide + Into<Symbol<T>>> Mappable for T {}

pub trait Mapped: Wide {
    fn mapping(&self) -> &EdgeMapping;
    fn mapping_mut(&mut self) -> &mut EdgeMapping;
}
impl<N: NodeData + Wide> Mapped for Symbol<N> {
    fn mapping(&self) -> &EdgeMapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut EdgeMapping {
        &mut self.mapping
    }
}
