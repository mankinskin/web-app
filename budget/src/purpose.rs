#![allow(unused)]
use daggy::{
	petgraph::algo::astar,
	Dag,
	NodeIndex,
};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Purpose {
	name: String,
}
impl std::fmt::Display for Purpose {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}
impl From<&str> for Purpose {
	fn from(name: &str) -> Self {
		Self { name: name.into() }
	}
}

#[derive(Debug)]
pub enum GraphError {
	PurposeDoesNotExist(Purpose),
	WouldCycle,
}

pub struct PurposeGraph {
	graph: Dag<Purpose, usize>,
	purposes: HashMap<Purpose, NodeIndex>,
}
impl PurposeGraph {
	pub fn new() -> Self {
		Self {
			graph: Dag::new(),
			purposes: HashMap::new(),
		}
	}
	pub fn add_purpose<P: Into<Purpose>>(&mut self, p: P) -> NodeIndex {
		let p = p.into();
		let id = self.graph.add_node(p.clone());
		self.purposes.insert(p, id);
		id
	}
	pub fn is_related_to<P: Into<Purpose>, B: Into<Purpose>>(
		&self,
		a: P,
		b: B,
	) -> Result<bool, GraphError> {
		let a = a.into();
		let b = b.into();
		let a = self
			.purposes
			.get(&a)
			.ok_or(GraphError::PurposeDoesNotExist(a))?;
		let b = self
			.purposes
			.get(&b)
			.ok_or(GraphError::PurposeDoesNotExist(b))?;
		match astar(self.graph.graph(), *a, |n| n == *b, |_e| 0, |_| 0) {
			Some(_) => Ok(true),
			None => Ok(false),
		}
	}

	pub fn link<P: Into<Purpose>, B: Into<Purpose>>(
		&mut self,
		a: P,
		b: B,
	) -> Result<(), GraphError> {
		let a = a.into();
		let b = b.into();
		let a = self
			.purposes
			.get(&a)
			.ok_or(GraphError::PurposeDoesNotExist(a))?;
		let b = self
			.purposes
			.get(&b)
			.ok_or(GraphError::PurposeDoesNotExist(b))?;
		self.graph
			.add_edge(*a, *b, 0)
			.map_err(|_e| GraphError::WouldCycle)
			.map(|_| ())
	}
}

use crate::interpreter::parse::*;
impl<'a> Parse<'a> for Purpose {
	named!(parse(&'a str) -> Self, map!(alpha1, |s| Self::from(s)));
}

#[derive(Debug, Clone, PartialEq)]
pub struct Purposes(Vec<Purpose>);
impl Purposes {
	pub fn new() -> Self {
		Self(Vec::new())
	}
	pub fn push<P: Into<Purpose>>(&mut self, p: P) {
		self.0.push(p.into());
	}
}
impl From<Vec<Purpose>> for Purposes {
	fn from(ps: Vec<Purpose>) -> Self {
		Self(ps)
	}
}
impl Into<Vec<Purpose>> for Purposes {
	fn into(self) -> Vec<Purpose> {
		self.0
	}
}
impl std::fmt::Display for Purposes {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"{}",
			self.0
				.iter()
				.fold(String::new(), |acc, x| acc + ", " + &x.to_string())
		)
	}
}

mod tests {
	#[test]
	fn relations() {
		use super::PurposeGraph;
		let mut pg = PurposeGraph::new();
		pg.add_purpose("Käse");
		pg.add_purpose("Brot");
		pg.add_purpose("Essen");
		pg.add_purpose("Gesundheit");
		assert!(!pg.is_related_to("Käse", "Brot").unwrap());
		assert!(!pg.is_related_to("Käse", "Essen").unwrap());
		assert!(!pg.is_related_to("Brot", "Käse").unwrap());
		assert!(!pg.is_related_to("Brot", "Essen").unwrap());
		pg.link("Käse", "Essen").unwrap();
		assert!(!pg.is_related_to("Käse", "Brot").unwrap());
		assert!(pg.is_related_to("Käse", "Essen").unwrap());
		assert!(!pg.is_related_to("Brot", "Käse").unwrap());
		assert!(!pg.is_related_to("Brot", "Essen").unwrap());
		pg.link("Brot", "Essen").unwrap();
		assert!(!pg.is_related_to("Käse", "Brot").unwrap());
		assert!(pg.is_related_to("Käse", "Essen").unwrap());
		assert!(!pg.is_related_to("Brot", "Käse").unwrap());
		assert!(pg.is_related_to("Brot", "Essen").unwrap());
		pg.link("Essen", "Gesundheit").unwrap();
		assert!(pg.is_related_to("Käse", "Gesundheit").unwrap());
		assert!(pg.is_related_to("Brot", "Gesundheit").unwrap());
		assert!(pg.is_related_to("Essen", "Gesundheit").unwrap());
		assert!(!pg.is_related_to("Gesundheit", "Essen").unwrap());
	}
}
