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
    ) -> Vec<Vec<Node<N>>> {
        graph.distance_group_source_weights(self.incoming.iter())
    }
    /// Get distance groups for outgoing edges
    pub fn outgoing_distance_groups<N: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<N>,
    ) -> Vec<Vec<Node<N>>> {
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
pub enum Token<T: NodeData> {
    Element(T),
    Start,
    End,
}
impl<T: NodeData + Display> Display for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Element(t) => t.to_string(),
                Token::Start => "START".to_string(),
                Token::End => "END".to_string(),
            }
        )
    }
}
impl<T: NodeData + Wide> Wide for Token<T> {
    fn width(&self) -> usize {
        match self {
            Token::Element(t) => t.width(),
            Token::Start => 0,
            Token::End => 0,
        }
    }
}
impl<N: NodeData> From<N> for Token<N> {
    fn from(e: N) -> Self {
        Token::Element(e)
    }
}
impl<N: NodeData> PartialEq<Node<N>> for Token<N> {
    fn eq(&self, rhs: &Node<N>) -> bool {
        *self == rhs.token
    }
}

/// Stores sequenced tokens with an edge map
#[derive(PartialEq, Clone)]
pub struct Node<N: NodeData> {
    pub token: Token<N>,
    mapping: EdgeMapping,
}
impl PartialEq<Node<char>> for char {
    fn eq(&self, rhs: &Node<char>) -> bool {
        *self == rhs.token
    }
}
impl PartialEq<Token<char>> for char {
    fn eq(&self, rhs: &Token<char>) -> bool {
        match rhs {
            Token::Element(e) => e == self,
            _ => false,
        }
    }
}
impl<N: NodeData> PartialEq<N> for Node<N> {
    fn eq(&self, rhs: &N) -> bool {
        self.token == *rhs
    }
}
impl<N: NodeData> PartialEq<N> for Token<N> {
    fn eq(&self, rhs: &N) -> bool {
        match self {
            Token::Element(e) => e == rhs,
            _ => false,
        }
    }
}
impl<N: NodeData> Debug for Node<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.token)
    }
}
impl<N: NodeData> Node<N> {
    pub fn new(token: N) -> Self {
        Self::from(Token::from(token))
    }
}
impl<N: NodeData> From<Token<N>> for Node<N> {
    fn from(token: Token<N>) -> Self {
        Self {
            token,
            mapping: Default::default(),
        }
    }
}
impl<N: NodeData> From<N> for Node<N> {
    fn from(e: N) -> Self {
        Node::new(e)
    }
}
impl<T: NodeData + Display> Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}
impl<T: NodeData + Wide> Wide for Node<T> {
    fn width(&self) -> usize {
        self.token.width()
    }
}
/// Trait for token that can be wrapped in a sequence
pub trait Sequencable: NodeData {
    fn sequenced<T: Into<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Token<Self>> {
        let mut v = vec![Token::Start];
        v.extend(seq.map(|t| Token::Element(t.into())));
        v.push(Token::End);
        v
    }
}
impl<T: NodeData + Into<Token<T>>> Sequencable for T {}
/// Trait for token that can be mapped in a sequence
pub trait Mappable: Sequencable + Wide {}
impl<T: NodeData + Wide + Into<Node<T>>> Mappable for T {}

pub trait Mapped: Wide {
    fn mapping(&self) -> &EdgeMapping;
    fn mapping_mut(&mut self) -> &mut EdgeMapping;
}
impl<N: NodeData + Wide> Mapped for Node<N> {
    fn mapping(&self) -> &EdgeMapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut EdgeMapping {
        &mut self.mapping
    }
}
