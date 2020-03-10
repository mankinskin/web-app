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
    pub fn group_by_distance(self) -> Vec<HashSet<GraphEdge<'a>>> {
        //println!("group_by_weight...");
        let max = self.max_weight().unwrap_or(0);
        let mut r: Vec<HashSet<_>> = Vec::new();
        for i in 1..=max {
            r.push(
                self.clone()
                    .into_iter()
                    .filter(move |e| e.weight().contains(&i))
                    .collect()
                    )
        }
        //println!("done");
        r
    }
    pub fn sort_by_distance(&mut self) -> Vec<usize> {
        let mut v: Vec<_> = self.clone()
            .into_iter()
            .map(|e| *e.max_weight().unwrap()).collect();
        v.sort_by(|b, a| {
            a.cmp(&b)
        });
        v
    }
    pub fn contains(&self, edge: &GraphEdge<'a>) -> bool {
        self.clone().into_iter().find(move |e| e == edge).is_some()
    }
    pub fn filter_by_weight(self, w: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.into_iter().filter(move |e| e.contains_weight(w))
    }
    pub fn intersection(self, other: &'a Self) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.into_iter()
            .filter(move |edge| {
                other.contains(edge)
            })
            .map(|e| e.clone())
    }
}

mod tests {
    use crate::*;
    use crate::graph::*;
    #[test]
    fn iter() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("a b c d"));
        tg.write_to_file("graphs/iter_graph");
        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&TextElement::Word(Word::from("a"))).unwrap();
        let b = tg.find_node(&TextElement::Word(Word::from("b"))).unwrap();
        let c = tg.find_node(&TextElement::Word(Word::from("c"))).unwrap();
        let d = tg.find_node(&TextElement::Word(Word::from("d"))).unwrap();
        let empty_a_edge = tg.find_edge(&empty, &a).unwrap();
        let a_empty_edge = tg.find_edge(&a, &empty).unwrap();
        let ab_edge = tg.find_edge(&a, &b).unwrap();
        let ac_edge = tg.find_edge(&a, &c).unwrap();
        let ad_edge = tg.find_edge(&a, &d).unwrap();
        let a_edges = tg.get_edges(a.index());
        assert_eq!(a_edges, GraphEdges::new(set![
                empty_a_edge,
                a_empty_edge,
                ab_edge,
                ac_edge,
                ad_edge
                ].iter().cloned()));
    }
}
