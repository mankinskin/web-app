use petgraph::{
	*,
	graph::*,
	graphmap::*,
	dot::*,
	visit::*,
};

use std::collections::{HashSet, HashMap};

use crate::graph::*;
use crate::graph::node::*;
use petgraph::visit::{EdgeRef};

pub type EdgeIter<'a> = petgraph::graph::Edges<'a, TextGraphEdgeWeight, Directed>;

#[derive(Debug, Clone, PartialEq)]
pub struct GraphEdges<'a>  {
	edges: HashSet<GraphEdge<'a>>,
}
impl<'a> std::iter::IntoIterator for GraphEdges<'a> {
	type Item = GraphEdge<'a>;
	type IntoIter = <HashSet<GraphEdge<'a>> as IntoIterator>::IntoIter;
	fn into_iter(self) -> Self::IntoIter {
		self.edges.into_iter()
	}
}
impl<'a> GraphEdges<'a>  {
	pub fn new<I: IntoIterator<Item=GraphEdge<'a>>>(edges: I) -> Self {
		let edges: HashSet<GraphEdge<'a>> = edges.into_iter().collect();
		Self {
			edges,
		}
	}
	pub fn max_edge(&'a self) -> Option<<Self as IntoIterator>::Item> {
		self.clone().into_iter().fold(None,
			|res: Option<(GraphEdge<'a>, usize)>, edge: GraphEdge<'a>| {
				Some(res.map(|(e, max)| {
						let w: usize = edge.weight().distance().clone();
						if w > max {
							(edge.clone(), w)
						} else {
							(e, max)
						}
					})
					.unwrap_or((edge.clone(), edge.weight().distance().clone()))
				)
			}
		)
		.map(|(e, _)| e)
	}
	pub fn max_weight(&'a self) -> Option<usize> {
		self.max_edge()
			.map(|e| e.weight().distance().clone())
	}
	pub fn group_by_distance(self) -> Vec<HashSet<GraphEdge<'a>>> {
		//println!("group_by_weight...");
		let max = self.max_weight().unwrap_or(0);
		let mut r: Vec<HashSet<_>> = Vec::new();
		for i in 1..=max {
			r.push(
				self.clone()
					.into_iter()
					.filter(|e| *e.weight() == i)
					.collect()
					)
		}
		//println!("done");
		r
	}
	pub fn sort_by_distance(&mut self) -> Vec<usize> {
		let mut v: Vec<_> = self.clone()
			.into_iter()
			.map(|e| e.weight().distance().clone()).collect();
		v.sort_by(|b, a| {
			a.cmp(&b)
		});
		v
	}
	pub fn contains(&self, edge: &GraphEdge<'a>) -> bool {
		self.clone().into_iter().find(move |e| e == edge).is_some()
	}
	pub fn filter_by_weight(self, w: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
		self.into_iter().filter(move |e| e.weight() == w)
	}
	pub fn intersection(self, other: &'a Self) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
		self.into_iter()
			.filter(move |edge| {
				other.contains(edge)
			})
			.map(|e| e.clone())
	}
}

pub mod tests {
	use crate::*;
	use crate::graph::*;
	use crate::graph::node::tests::*;
	pub use pretty_assertions::{assert_eq};
	lazy_static!{
		static ref START_A1_EDGE: GraphEdge<'static> = TG.find_edge(&START, &A, 1).unwrap();
		static ref START_A2_EDGE: GraphEdge<'static> = TG.find_edge(&START, &A, 2).unwrap();
		static ref START_A4_EDGE: GraphEdge<'static> = TG.find_edge(&START, &A, 4).unwrap();
		static ref A_STOP1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &STOP, 1).unwrap();
		static ref A_STOP3_EDGE: GraphEdge<'static> = TG.find_edge(&A, &STOP, 3).unwrap();
		static ref A_STOP4_EDGE: GraphEdge<'static> = TG.find_edge(&A, &STOP, 4).unwrap();
		static ref AB1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &B, 1).unwrap();
		static ref AC1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &B, 1).unwrap();
		static ref AC2_EDGE: GraphEdge<'static> = TG.find_edge(&A, &C, 2).unwrap();
		static ref AD3_EDGE: GraphEdge<'static> = TG.find_edge(&A, &D, 3).unwrap();
		static ref EA1_EDGE: GraphEdge<'static> = TG.find_edge(&E, &A, 1).unwrap();
		static ref AA1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &A, 1).unwrap();
		static ref AA2_EDGE: GraphEdge<'static> = TG.find_edge(&A, &A, 2).unwrap();
		static ref AA3_EDGE: GraphEdge<'static> = TG.find_edge(&A, &A, 3).unwrap();
		static ref AF1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &F, 1).unwrap();
		static ref AF2_EDGE: GraphEdge<'static> = TG.find_edge(&A, &F, 2).unwrap();
		static ref FA1_EDGE: GraphEdge<'static> = TG.find_edge(&A, &F, 1).unwrap();
		static ref A_EDGES: GraphEdges<'static> = TG.get_edges(A.index());
	}
	//#[test]
	//fn iter() {
	//	assert_eq!(
	//		A_EDGES.clone(),
	//		GraphEdges::new(set![
	//				START_A1_EDGE.clone(),
	//				A_STOP4_EDGE.clone(),
	//				AB1_EDGE.clone(),
	//				AC1_EDGE.clone(),
	//				AC2_EDGE.clone(),
	//				AD3_EDGE.clone(),
	//				EA1_EDGE.clone(),
	//				AA1_EDGE.clone(),
	//				AA2_EDGE.clone(),
	//				AA3_EDGE.clone(),
	//				AF1_EDGE.clone(),
	//				AF2_EDGE.clone(),
	//				FA1_EDGE.clone(),
	//			]
	//			.iter()
	//			.cloned()
	//		)
	//	);
	//}
}
