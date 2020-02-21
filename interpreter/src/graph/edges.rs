use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use std::collections::{HashSet, HashMap};

use crate::graph::node::*;

pub type Edge<'a> = EdgeReference<'a, HashSet<usize>>;
pub type Edges<'a> = Vec<Edge<'a>>;

#[derive(Clone, Debug)]
pub struct GraphEdge<'a>  {
    edge: EdgeReference<'a, HashSet<usize>>,
}
impl<'a> std::ops::Deref for GraphEdge<'a> {
    type Target = Edge<'a>;
    fn deref(&self) -> &Self::Target {
        &self.edge
    }
}
impl<'a> From<Edge<'a>> for GraphEdge<'a>  {
    fn from(edge: Edge<'a>) -> Self {
        Self {
            edge
        }
    }
}
impl<'a> GraphEdge<'a>  {
    fn contains_weight(&self, w: &usize) -> bool {
        self.weight().contains(w)
    }
    fn max_weight(&self) -> Option<&'a usize> {
        self.weight().iter().max()
    }
}

#[derive(Clone, Debug)]
pub struct GraphEdges<'a>  {
    edges: Vec<GraphEdge<'a>>,
}
impl<'a> std::ops::Deref for GraphEdges<'a> {
    type Target = Vec<GraphEdge<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.edges
    }
}
impl<'a> From<Vec<GraphEdge<'a>>> for GraphEdges<'a>  {
    fn from(edges: Vec<GraphEdge<'a>>) -> Self {
        Self {
            edges
        }
    }
}
impl<'a> From<Vec<Edge<'a>>> for GraphEdges<'a>  {
    fn from(edges: Vec<Edge<'a>>) -> Self {
        Self {
            edges: edges.iter().map(|e|GraphEdge::from(e.clone())).collect()
        }
    }
}
use std::iter::Iterator;
impl<'a> From<petgraph::graph::Edges<'a, HashSet<usize>, petgraph::Directed>> for GraphEdges<'a>  {
    fn from(edges: petgraph::graph::Edges<'a, HashSet<usize>, petgraph::Directed>) -> Self {
        Self {
            edges: edges.map(|e| GraphEdge::from(e.clone())).collect()
        }
    }
}

impl<'a> GraphEdges<'a>  {
    pub fn max_edge(&self) -> Option<&GraphEdge<'a>> {
        self.edges.iter().fold(None,
            |res: Option<(&GraphEdge<'a>, usize)>, edge: &GraphEdge<'a>| {
                res.map(|(e, max)| {
                        let w = edge.max_weight().unwrap();
                        if *w > max {
                            Some((edge, w.clone()))
                        } else {
                            res
                        }
                    })
                .unwrap_or(Some((edge, edge.max_weight().unwrap().clone())))
            }
        )
        .map(|(e, max)| e)
    }
    pub fn max_weight(&self) -> Option<usize> {
        self.max_edge()
            .map(|e| e.max_weight())
            .flatten()
            .map(Clone::clone)
    }
    pub fn group_by_weight(&self) -> Vec<Vec<GraphEdge<'a>>> {
        let max = self.max_weight().unwrap_or(0);
        let mut r: Vec<Vec<GraphEdge<'a>>> = vec![Vec::new(); max];
        for edge in self.iter() {
            let w = edge.max_weight();
            match w {
                Some(i) => r[i-1].push(edge.clone()),
                None => {}
            }
        }
        r
    }
    pub fn sort_by_weight(&mut self) {
        let mut edge_max_weights: Vec<&usize> = self.iter().flat_map(GraphEdge::max_weight).collect();
        edge_max_weights.sort_by(|b, a| {
            a.cmp(&b)
        });
    }
    pub fn filter_by<P: FnMut(&&GraphEdge<'a>) -> bool>(&mut self, p: P) {
        self.edges = self.edges.iter()
            .filter(p)
            .map(|e| e.clone())
            .collect();
    }
    pub fn filter_by_weight(&mut self, w: &usize) {
        self.filter_by(|e| e.contains_weight(w));
    }
}
