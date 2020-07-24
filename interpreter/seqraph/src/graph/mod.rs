use petgraph::{
    graph::{
        DiGraph,
        EdgeIndex,
        NodeIndex,
    },
    visit::{
        EdgeRef
    },
    dot::{
        Dot,
    },
};
use std::{
    fmt::{
        Debug,
    },
    path::{
        PathBuf,
    },
};
use node::{
    NodeData,
    NodeWeight,
};
use edge::{
    EdgeData,
};
use std::ops::{
    Deref,
    DerefMut,
};

pub mod edge;
pub mod node;

#[derive(Debug)]
pub struct Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    graph: DiGraph<NodeWeight<N>, E>,
}
impl<N, E> Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }

    pub fn add_node(&mut self, element: N) -> NodeIndex {
        if let Some(i) = self.find_node(&element) {
            i
        } else {
            self.graph.add_node(
                NodeWeight::new(element)
            )
        }
    }
    fn find_node(&self, element: &N) -> Option<NodeIndex> {
        self.graph
            .node_indices()
            .find(|i| self.graph[*i].data == *element)
            .map(|i| i.clone())
    }
    pub fn find_nodes(&self, elems: &[N]) -> Option<Vec<NodeIndex>> {
        elems.iter().map(|e| self.find_node(e)).collect()
    }
    pub fn has_node(&self, element: &N) -> bool {
        self.find_node(element).is_some()
    }

    pub fn add_edge(&mut self, li: NodeIndex, ri: NodeIndex, w: E) -> EdgeIndex {
        let e = self.find_edge(li, ri, &w);
        let ei = if let Some(i) = e {
            i
        } else {
            self.graph.add_edge(li, ri, w)
        };
        ei
    }
    pub fn add_node_edge(&mut self, l: N, r: N, w: E) -> EdgeIndex {
        let li = self.add_node(l);
        let ri = self.add_node(r);
        self.graph.add_edge(li, ri, w)
    }
    pub fn find_edge(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> Option<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .find(|e| *e.weight() == *w)
            .map(|e| e.id())
    }
    pub fn find_node_edge(&self, l: &N, r: &N, w: &E) -> Option<EdgeIndex> {
        let li = self.find_node(l)?;
        let ri = self.find_node(r)?;
        self.find_edge(li, ri, w)
    }
    pub fn find_edges(&self, li: NodeIndex, ri: NodeIndex) -> Vec<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .map(|e| e.id())
            .collect()
    }
    pub fn find_node_edges(&self, l: &N, r: &N) -> Vec<EdgeIndex> {
        let li = self.find_node(l);
        let ri = self.find_node(r);
        if let (Some(li), Some(ri)) = (li, ri) {
            self.graph
                .edges_connecting(li, ri)
                .map(|e| e.id())
                .collect()
        } else {
            Vec::new()
        }
    }
    pub fn has_node_edge(&self, l: &N, r: &N, w: &E) -> bool {
        self.find_node_edge(l, r, w).is_some()
    }
    pub fn has_edge(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> bool {
        self.find_edge(li, ri, w).is_some()
    }
    pub fn write_to_file<S: Into<PathBuf>>(&self, name: S) -> std::io::Result<()> {
        let mut path: PathBuf = name.into();
        path.set_extension("dot");
        //path.canonicalize()?;
        path.parent().map(|p|
            std::fs::create_dir_all(p.clone())
        );
        std::fs::write(path, format!("{:?}", Dot::new(&self.graph)))
    }
}
impl<N: NodeData, E: EdgeData> Deref for Graph<N, E> {
    type Target = DiGraph<NodeWeight<N>, E>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData, E: EdgeData> DerefMut for Graph<N, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    lazy_static!{
        static ref ELEMS: Vec<char> = {
            Vec::from(['a', 'b', 'c'])
        };
        static ref EDGES: Vec<(char, char, usize)> = {
            Vec::from([
                ('a', 'b', 1),
                ('b', 'c', 2),
                ('c', 'a', 3)
            ])
        };
        static ref G: Graph<char, usize> = {
            let mut g = Graph::new();
            for e in ELEMS.iter() {
                g.add_node(e.clone());
            }
            for (l, r, w) in EDGES.iter() {
                g.add_node_edge(l.clone(), r.clone(), w.clone());
            }
            g
        };
    }
    #[test]
    fn has_node() {
        for e in ELEMS.iter() {
            assert!(G.has_node(&e));
        }
    }
    #[test]
    fn has_node_edge() {
        for (l, r, w) in EDGES.iter() {
            assert!(G.has_node_edge(l, r, w));
        }
    }
    #[test]
    fn write_to_file() {
        G.write_to_file("test_graph").unwrap();
    }
}
