use edge::EdgeData;
use node::NodeData;
use petgraph::{
    dot::Dot,
    graph::{
        DiGraph,
        EdgeIndex,
        NodeIndex,
    },
    visit::EdgeRef,
};
use std::ops::{
    Deref,
    DerefMut,
};
use std::{
    fmt::Debug,
    path::PathBuf,
};

pub mod edge;
pub mod node;

#[derive(Debug)]
pub struct Graph<N, E>
where
    N: NodeData,
    E: EdgeData,
{
    graph: DiGraph<N, E>,
}
impl<'a, N, E> Graph<N, E>
where
    N: NodeData,
    E: EdgeData,
{
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }
    /// Nodes

    /// Return a NodeIndex for a node with NodeData
    pub fn add_node<T: PartialEq<N> + Into<N>>(&mut self, element: T) -> NodeIndex {
        if let Some(i) = self.find_node_index(&element) {
            i
        } else {
            self.graph.add_node(element.into())
        }
    }
    /// Get NodeWeight for NodeIndex
    pub fn get_node_weight(&self, i: NodeIndex) -> Option<N> {
        self.graph.node_weight(i).map(Clone::clone)
    }
    /// Find NodeIndex for NodeData, if any
    pub fn find_node_index<T: PartialEq<N>>(&self, element: &T) -> Option<NodeIndex> {
        self.graph
            .node_indices()
            .find(|i| *element == self.graph[*i])
            .map(|i| i.clone())
    }
    /// Return NodeWeight for NodeData, if any
    pub fn find_node_weight<T: PartialEq<N>>(&self, element: &T) -> Option<N> {
        let i = self.find_node_index(element)?;
        self.graph.node_weight(i).map(Clone::clone)
    }
    /// Return any NodeIndices for NodeDatas
    pub fn find_node_indices<T: PartialEq<N> + 'a>(
        &'a self,
        es: impl Iterator<Item = &'a T> + 'a,
    ) -> Option<Vec<NodeIndex>> {
        es.map(|e| self.find_node_index(e)).collect()
    }
    /// Return any NodeWeights for NodeDatas
    pub fn find_node_weights<T: PartialEq<N> + 'a>(
        &'a self,
        es: impl Iterator<Item = &'a T> + 'a,
    ) -> Option<Vec<N>> {
        es.map(|e| self.find_node_weight(e)).collect()
    }
    /// True if NodeData has node in graph
    pub fn has_node<T: PartialEq<N>>(&self, element: &T) -> bool {
        self.find_node_index(element).is_some()
    }
    /// Map NodeIndices to weights
    pub fn node_weights(
        &'a self,
        is: impl Iterator<Item = &'a NodeIndex> + 'a,
    ) -> impl Iterator<Item = N> + 'a {
        is.filter_map(move |i| self.get_node_weight(i.clone()))
    }
    /// Get all NodeIndices in the graph
    pub fn all_node_indices(&self) -> Vec<NodeIndex> {
        self.node_indices().collect()
    }
    /// Get all NodeWeights in the graph
    pub fn all_node_weights(&self) -> Vec<N> {
        self.raw_nodes().iter().map(|n| n.weight.clone()).collect()
    }

    /// Edges

    /// Return an EdgeIndex for an edge with weight between NodeIndices
    pub fn add_edge(&mut self, li: NodeIndex, ri: NodeIndex, w: E) -> EdgeIndex {
        let i = self.find_edge_index(li, ri, &w);
        let i = if let Some(i) = i {
            i
        } else {
            self.graph.add_edge(li, ri, w)
        };
        i
    }
    /// Add a new edge with weight between NodeDatas
    pub fn add_node_edge(&mut self, l: N, r: N, w: E) -> EdgeIndex {
        let li = self.add_node(l);
        let ri = self.add_node(r);
        self.graph.add_edge(li, ri, w)
    }
    /// Return weight of edge with index, if any
    pub fn get_edge_weight(&self, i: EdgeIndex) -> Option<E> {
        self.graph.edge_weight(i).map(Clone::clone)
    }
    /// Get all edge weights between NodeIndices
    pub fn get_edges(&self, li: NodeIndex, ri: NodeIndex) -> Vec<E> {
        self.edge_weights(self.find_edge_indices(li, ri).iter())
            .collect()
    }
    /// Get all edge weights between NodeDatas
    pub fn get_node_edges<T: PartialEq<N>>(&self, l: &T, r: &T) -> Vec<E> {
        self.edge_weights(self.find_node_edge_indices(l, r).iter())
            .collect()
    }
    /// Find edge index of edge with weight between NodeIndices
    pub fn find_edge_index(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> Option<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .find(|e| *e.weight() == *w)
            .map(|e| e.id())
    }
    /// Find edge index of edge with weight between NodeDatas
    pub fn find_node_edge_index<T: PartialEq<N>>(&self, l: &T, r: &T, w: &E) -> Option<EdgeIndex> {
        let li = self.find_node_index(l)?;
        let ri = self.find_node_index(r)?;
        self.find_edge_index(li, ri, w)
    }
    /// Find edge indices connecting NodeIndices
    pub fn find_edge_indices(&self, li: NodeIndex, ri: NodeIndex) -> Vec<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .map(|e| e.id())
            .collect()
    }
    /// Find edge indices between NodeDatas
    pub fn find_node_edge_indices<T: PartialEq<N>>(&self, l: &T, r: &T) -> Vec<EdgeIndex> {
        let li = self.find_node_index(l);
        let ri = self.find_node_index(r);
        if let (Some(li), Some(ri)) = (li, ri) {
            self.find_edge_indices(li, ri)
        } else {
            Vec::new()
        }
    }
    /// Get all EdgeIndices in the graph
    pub fn all_edge_indices(&self) -> Vec<EdgeIndex> {
        self.edge_indices().collect()
    }
    /// True if edge with weight between NodeDatas
    pub fn has_node_edge<T: PartialEq<N>>(&self, l: &T, r: &T, w: &E) -> bool {
        self.find_node_edge_index(l, r, w).is_some()
    }
    /// True if edge with weight between NodeIndices
    pub fn has_edge(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> bool {
        self.find_edge_index(li, ri, w).is_some()
    }
    /// Get source of edge
    pub fn edge_source(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoints(i).map(|t| t.0)
    }
    /// Get sources of edges
    pub fn edge_sources(
        &'a self,
        is: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        is.filter_map(move |i| self.edge_source(*i))
    }
    /// Get target of edge
    pub fn edge_target(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoints(i).map(|t| t.1)
    }
    /// Get targets of edges
    pub fn edge_targets(
        &'a self,
        is: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        is.filter_map(move |i| self.edge_target(*i))
    }
    /// Get weights of edges
    pub fn edge_weights(
        &'a self,
        is: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> impl Iterator<Item = E> + 'a {
        is.filter_map(move |i| self.get_edge_weight(*i))
    }
    /// Write graph to dot file
    pub fn write_to_file<S: Into<PathBuf>>(&self, name: S) -> std::io::Result<()> {
        let mut path: PathBuf = name.into();
        path.set_extension("dot");
        //path.canonicalize()?;
        path.parent().map(|p| std::fs::create_dir_all(p.clone()));
        std::fs::write(path, format!("{:?}", Dot::new(&self.graph)))
    }
}
impl<'a, N: NodeData> Graph<N, usize> {
    /// Group EdgeIndices by distances
    pub fn group_edges_by_distance(
        &'a self,
        es: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> Vec<Vec<EdgeIndex>> {
        let es = es
            .filter_map(|i| Some((i.clone(), self.get_edge_weight(*i)?.clone())))
            .collect::<Vec<_>>();
        let mut r = Vec::new();
        if let Some(&max) = es.iter().map(|(_, d)| d).max() {
            for d in 1..=max {
                r.push(
                    es.iter()
                        .filter(|(_, dist)| *dist == d)
                        .map(|(i, _)| i.clone())
                        .collect(),
                )
            }
        }
        r
    }
    /// Map function to groups
    pub fn map_groups<A: 'a, T: 'a, R: Iterator<Item = T> + 'a>(
        gs: impl Iterator<Item = &'a Vec<A>> + 'a,
        f: impl Fn(&'a Vec<A>) -> R,
    ) -> Vec<Vec<T>> {
        gs.map(move |group| f(group).collect()).collect()
    }
    pub fn distance_group_sources(
        &'a self,
        es: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> Vec<Vec<NodeIndex>> {
        Self::map_groups(self.group_edges_by_distance(es).iter(), move |group| {
            self.edge_sources(group.iter())
        })
    }
    pub fn distance_group_targets(
        &'a self,
        es: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> Vec<Vec<NodeIndex>> {
        Self::map_groups(self.group_edges_by_distance(es).iter(), move |group| {
            self.edge_targets(group.iter())
        })
    }
    pub fn distance_group_source_weights(
        &'a self,
        es: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> Vec<Vec<N>> {
        Self::map_groups(self.distance_group_sources(es).iter(), move |group| {
            self.node_weights(group.iter())
        })
    }
    pub fn distance_group_target_weights(
        &'a self,
        es: impl Iterator<Item = &'a EdgeIndex> + 'a,
    ) -> Vec<Vec<N>> {
        Self::map_groups(self.distance_group_targets(es).iter(), move |group| {
            self.node_weights(group.iter())
        })
    }
}
impl<N: NodeData, E: EdgeData> Deref for Graph<N, E> {
    type Target = DiGraph<N, E>;
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
    use std::sync::{
        Mutex,
        MutexGuard,
    };
    lazy_static::lazy_static! {
        static ref ELEMS: Vec<char> = Vec::from(['a', 'b', 'c']);
        static ref EDGES: Vec<(char, char, usize)> =
            Vec::from([('a', 'b', 1), ('b', 'c', 2), ('c', 'a', 3)]);
        static ref G: Mutex<Graph<char, usize>> = Mutex::new(Graph::new());
        static ref NODE_INDICES: Vec<NodeIndex> = {
            let mut g = G.lock().unwrap();
            ELEMS.iter().map(|e| g.add_node(e.clone())).collect()
        };
        static ref EDGE_INDICES: Vec<EdgeIndex> = {
            let mut g = G.lock().unwrap();
            EDGES
                .iter()
                .map(|(l, r, w)| g.add_node_edge(l.clone(), r.clone(), w.clone()))
                .collect()
        };
    }
    fn init() -> MutexGuard<'static, Graph<char, usize>> {
        format!("{:?}", *NODE_INDICES);
        format!("{:?}", *EDGE_INDICES);
        G.lock().unwrap()
    }
    #[test]
    fn has_node() {
        let g = init();
        for e in ELEMS.iter() {
            assert!(g.has_node(e));
        }
    }
    #[test]
    fn has_node_edge() {
        let g = init();
        for (l, r, w) in EDGES.iter() {
            assert!(g.has_node_edge(l, r, w));
        }
    }
    #[test]
    fn write_to_file() {
        let g = init();
        g.write_to_file("test_graph").unwrap();
    }
}
