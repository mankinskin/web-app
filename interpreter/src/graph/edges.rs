use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use std::collections::{HashSet, HashMap};

use crate::graph::node::*;
use crate::graph::edge::{*, EdgeRef};

pub type EdgeIter<'a> = petgraph::graph::Edges<'a, HashSet<usize>, Directed>;

#[derive(Clone)]
pub struct GraphEdges<'a>  {
    iter: EdgeIter<'a>,
}
impl<'a> std::iter::Iterator for GraphEdges<'a> {
    type Item = GraphEdge<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(GraphEdge::from)
    }
}
impl<'a> From<EdgeIter<'a>> for GraphEdges<'a> {
    fn from(iter: EdgeIter<'a>) -> Self {
        Self {
            iter
        }
    }
}

impl<'a> GraphEdges<'a>  {
    pub fn max_edge(&'a self) -> Option<<Self as Iterator>::Item> {
        self.clone().fold(None,
            |res: Option<(GraphEdge<'a>, usize)>, edge: GraphEdge<'a>| {
                Some(res.map(|(e, max)| {
                        let w = edge.max_weight().unwrap().clone();
                        if w > max {
                            (edge.clone(), w.clone())
                        } else {
                            (e, max)
                        }
                    })
                    .unwrap_or((edge.clone(), *edge.max_weight().unwrap()))
                )
            }
        )
        .map(|(e, _)| e)
    }
    pub fn max_weight(&'a self) -> Option<usize> {
        self.max_edge()
            .map(|e| e.max_weight().map(Clone::clone))
            .flatten()
    }
    pub fn group_by_weight(self) -> Vec<impl Iterator<Item=GraphEdge<'a>> + Clone> {
        let max = self.max_weight().unwrap_or(0);
        let mut r: Vec<_> = Vec::new();
        for i in 1..=max {
            r.push(
                self.clone()
                    .filter(move |e| e.weight().contains(&i))
                    )
        }
        r
    }
    pub fn sort_by_weight(&mut self) -> Vec<usize> {
        let mut v: Vec<_> = self.map(|e| *e.max_weight().unwrap()).collect();
        v.sort_by(|b, a| {
            a.cmp(&b)
        });
        v
    }
    pub fn filter_by_weight(self, w: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.filter(move |e| e.contains_weight(w))
    }
}
