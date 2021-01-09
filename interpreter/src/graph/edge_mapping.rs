use petgraph::{
	graph::{
		EdgeIndex
	},
};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Mul, Add, MulAssign, AddAssign};
use crate::graph::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EdgeMappingMatrixValue {
	val: bool
}

impl Display for EdgeMappingMatrixValue {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", match self.val {
			true => "I",
			false => "O",
		})
	}
}
impl num_traits::Zero for EdgeMappingMatrixValue {
	fn zero() -> Self {
		Self { val: false }
	}
	fn is_zero(&self) -> bool {
		!self.val
	}
}
impl num_traits::One for EdgeMappingMatrixValue {
	fn one() -> Self {
		Self { val: true }
	}
	fn is_one(&self) -> bool {
		self.val
	}
}
impl From<bool> for EdgeMappingMatrixValue {
	fn from(val: bool) -> Self {
		Self { val }
	}
}
impl Into<bool> for EdgeMappingMatrixValue {
	fn into(self) -> bool {
		self.val
	}
}
impl Mul for EdgeMappingMatrixValue {
	type Output = Self;
	fn mul(self, o: Self) -> Self::Output {
		Self::from(self.val && o.val)
	}
}
impl Add for EdgeMappingMatrixValue {
	type Output = Self;
	fn add(self, o: Self) -> Self::Output {
		Self::from(self.val || o.val)
	}
}
impl AddAssign for EdgeMappingMatrixValue {
	fn add_assign(&mut self, o: Self) {
		*self = *self + o;
	}
}
impl MulAssign for EdgeMappingMatrixValue {
	fn mul_assign(&mut self, o: Self) {
		*self = *self * o;
	}
}

pub type EdgeMappingMatrix = nalgebra::Matrix<EdgeMappingMatrixValue,
	nalgebra::Dynamic,
	nalgebra::Dynamic,
	nalgebra::VecStorage<EdgeMappingMatrixValue, nalgebra::Dynamic, nalgebra::Dynamic>>;

#[derive(PartialEq, Clone)]
pub struct EdgeMapping {
	pub matrix: EdgeMappingMatrix,
	pub outgoing_edges: Vec<EdgeIndex>,
	pub incoming_edges: Vec<EdgeIndex>,
}

impl<'a> EdgeMapping {
	pub fn new() -> Self {
		Self {
			matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
			outgoing_edges: Vec::new(),
			incoming_edges: Vec::new(),
		}
	}
	pub fn add_incoming_edge(&mut self, edge: EdgeIndex) -> usize {
		if let Some(i) = self.incoming_edges.iter().position(|e| *e == edge) {
			i
		} else {
			self.incoming_edges.push(edge);
			self.matrix = self.matrix.clone().insert_column(self.matrix.ncols(), false.into());
			self.incoming_edges.len() - 1
		}
	}
	pub fn add_outgoing_edge(&mut self, edge: EdgeIndex) -> usize {
		if let Some(i) = self.outgoing_edges.iter().position(|e| *e == edge) {
			i
		} else {
			self.outgoing_edges.push(edge);
			self.matrix = self.matrix.clone().insert_row(self.matrix.nrows(), false.into());
			self.outgoing_edges.len() - 1
		}
	}
	pub fn add_transition(&mut self, left_edge: EdgeIndex, right_edge: EdgeIndex) {
		let left_index = self.add_incoming_edge(left_edge);
		let right_index = self.add_outgoing_edge(right_edge);
		self.matrix[(right_index, left_index)] = true.into();
	}
}
impl Debug for EdgeMapping {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "EdgeMapping(\nincoming_edges: {:#?},\noutgoing_edges: {:#?},\nmatrix: {}",
			self.incoming_edges, self.outgoing_edges, self.matrix)
	}
}
impl Display for EdgeMapping {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "matrix: {}", self.matrix)
	}
}
