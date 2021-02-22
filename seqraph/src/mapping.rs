use crate::{
    graph::{
        node::NodeData,
        edge::EdgeData,
        Graph,
    },
    node::{
        Node,
        Edge,
    },
    token::{
        TokenData,
        Token,
        Wide,
    },
    SequenceGraph,
};
use petgraph::graph::{
    NodeIndex,
};
use std::{
    ops::{
        Mul,
        MulAssign,
        Add,
        AddAssign,
    },
    default::Default,
    fmt::{
        self,
        Formatter,
        Debug,
        Display,
    },
};
#[allow(unused)]
use tracing::{
    debug,
};
pub type EdgeMappingMatrix = nalgebra::DMatrix<ArithmeticBool>;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct ArithmeticBool(bool);

impl Display for ArithmeticBool {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self.0 {
            true => "1",
            false => "0",
        })
    }
}
impl num_traits::Zero for ArithmeticBool {
    fn zero() -> Self {
        Self(false)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
}
impl num_traits::One for ArithmeticBool {
    fn one() -> Self {
        Self(true)
    }
    fn is_one(&self) -> bool {
        self.0
    }
}
impl From<bool> for ArithmeticBool {
    fn from(b: bool) -> Self {
        Self(b)
    }
}
impl Into<bool> for ArithmeticBool {
    fn into(self) -> bool {
        self.0
    }
}
impl Mul for ArithmeticBool {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::from(self.0 && other.0)
    }
}
impl Add for ArithmeticBool {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::from(self.0 || other.0)
    }
}
impl AddAssign for ArithmeticBool {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl MulAssign for ArithmeticBool {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
#[derive(PartialEq, Clone, Debug, Eq)]
pub struct EdgeMapping {
    pub matrix: EdgeMappingMatrix,
    pub incoming: Vec<Edge>,
    pub outgoing: Vec<Edge>,
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
    fn add_incoming_edge(&mut self, edge: Edge) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| e.index == edge.index) {
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
    fn add_outgoing_edge(&mut self, edge: Edge) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| e.index == edge.index) {
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
    pub fn add_transition(&mut self, l: Edge, r: Edge) {
        let li = self.add_incoming_edge(l);
        let ri = self.add_outgoing_edge(r);
        self.matrix[(ri, li)] = true.into();
    }
    pub fn remove_zero_columns(&mut self) {
        let (incoming, columns): (_, Vec<_>) = self.incoming
            .iter()
            .cloned()
            .zip(self.matrix.column_iter())
            .filter(|(_, col)| col.iter().any(|b| b.0))
            .unzip();
        self.incoming = incoming;
        self.matrix = EdgeMappingMatrix::from_columns(&columns);
    }
    pub fn remove_zero_rows(&mut self) {
        let (outgoing, rows): (_, Vec<_>) = self.outgoing
            .iter()
            .cloned()
            .zip(self.matrix.row_iter())
            .filter(|(_, row)| row.iter().any(|b| b.0))
            .unzip();
        self.outgoing = outgoing;
        self.matrix = EdgeMappingMatrix::from_rows(&rows);
    }
    /// Get weights and sources of incoming edges
    pub fn incoming_sources(&'a self) -> impl Iterator<Item = (usize, NodeIndex)> + 'a {
        self.incoming.iter().map(|e| (e.dist, e.node))
    }
    /// Get weights and targets of outgoing edges
    pub fn outgoing_targets(&'a self) -> impl Iterator<Item = (usize, NodeIndex)> + 'a {
        self.outgoing.iter().map(|e| (e.dist, e.node))
    }

    /// Get distance groups for incoming edges
    pub fn incoming_distance_groups<T: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<T>,
    ) -> Vec<Vec<Node<T>>> {
        graph.distance_group_source_weights(self.incoming.iter().map(|e| e.index))
    }
    /// Get distance groups for outgoing edges
    pub fn outgoing_distance_groups<T: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<T>,
    ) -> Vec<Vec<Node<T>>> {
        graph.distance_group_target_weights(self.outgoing.iter().map(|e| e.index))
    }
}
impl Default for EdgeMapping {
    fn default() -> Self {
        Self::new()
    }
}
/// Trait for token that can be wrapped in a sequence
pub trait Sequencable: TokenData {
    fn sequenced<T: Into<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Token<Self>> {
        let mut v = vec![Token::Start];
        v.extend(seq.map(|t| Token::Element(t.into())));
        v.push(Token::End);
        v
    }
}
impl<T: TokenData + Into<Token<T>>> Sequencable for T {}
/// Trait for token that can be mapped in a sequence
pub trait Mappable: Sequencable + Wide {}
impl<T: TokenData + Wide> Mappable for T {}

pub trait Mapped: Wide {
    fn mapping(&self) -> &EdgeMapping;
    fn mapping_mut(&mut self) -> &mut EdgeMapping;
}

