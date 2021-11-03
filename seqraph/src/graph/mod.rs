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
    Direction,
};
use std::ops::{
    Deref,
    DerefMut,
};
use std::{
    fmt::Debug,
    path::PathBuf,
};
use tracing::debug;

pub mod edge;
pub mod node;

#[derive(Debug, Default)]
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
    /// Nodes

    /// Return a NodeIndex for a node with NodeData
    pub fn node(&mut self, element: &N) -> NodeIndex {
        if let Some(i) = self.find_node_index(element.clone()) {
            //debug!("Found node for element {:?} with index {}", element, i.index());
            i
        } else {
            self.graph.add_node(element.clone())
            //debug!("Added element node {:?} with index {}", element, i.index());
        }
    }
    /// Find NodeIndex for NodeData, if any
    pub fn find_node_index<T: PartialEq<N> + Debug>(&self, element: T) -> Option<NodeIndex> {
        //debug!("Finding node index for element {:?}", element);
        self.node_indices().find(|i| element == self.graph[*i])
        //debug!("Found {:?}", i.map(|i| i.index()));
    }
    /// Return NodeWeight for NodeData, if any
    pub fn find_node_weight<T: PartialEq<N> + Debug>(&self, element: T) -> Option<N> {
        debug!("Finding node weight for element {:?}", element);
        let i = self.find_node_index(element)?;
        //debug!("Found index {}", i.index());
        self.graph.node_weight(i).map(Clone::clone)
    }
    /// Return any NodeIndices for NodeDatas
    pub fn find_node_indices<T: PartialEq<N> + Debug>(
        &self,
        es: impl Iterator<Item = T>,
    ) -> Option<Vec<NodeIndex>> {
        es.map(|e| self.find_node_index(e)).collect()
    }
    /// Return any NodeWeights for NodeDatas
    pub fn find_node_weights<T: PartialEq<N> + Debug>(
        &self,
        es: impl Iterator<Item = T>,
    ) -> Option<Vec<N>> {
        es.map(|e| self.find_node_weight(e)).collect()
    }
    /// True if NodeData has node in graph
    pub fn has_node<T: PartialEq<N> + Debug>(&self, element: T) -> bool {
        self.find_node_index(element).is_some()
    }
    /// True if NodeData has all nodes in graph
    pub fn has_nodes<T: PartialEq<N> + Debug>(
        &self,
        mut elements: impl Iterator<Item = T>,
    ) -> bool {
        elements.all(|e| self.find_node_index(e).is_some())
    }
    /// Map NodeIndices to weights
    pub fn node_weights_for(
        &'a self,
        is: impl IntoIterator<Item = NodeIndex> + 'a,
    ) -> impl Iterator<Item = N> + 'a {
        is.into_iter()
            .filter_map(move |i| self.node_weight(i).cloned())
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
    pub fn edge(&mut self, li: NodeIndex, ri: NodeIndex, w: E) -> EdgeIndex {
        let i = self.find_edge_index(li, ri, &w);
        if let Some(i) = i {
            i
        } else {
            self.graph.add_edge(li, ri, w)
        }
    }
    /// Add a new edge with weight between NodeDatas
    pub fn node_edge(&mut self, l: &N, r: &N, w: E) -> EdgeIndex {
        let li = self.node(l);
        let ri = self.node(r);
        self.graph.add_edge(li, ri, w)
    }
    /// Get all edge weights between NodeIndices
    pub fn get_edges(&self, li: NodeIndex, ri: NodeIndex) -> Vec<E> {
        self.edge_weights_for(self.find_edge_indices(li, ri))
            .collect()
    }
    /// Get all edge weights between NodeDatas
    pub fn get_node_edges<T: PartialEq<N> + Debug>(&self, l: T, r: T) -> Vec<E> {
        self.edge_weights_for(self.find_node_edge_indices(l, r))
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
    pub fn find_node_edge_index<T: PartialEq<N> + Debug>(
        &self,
        l: T,
        r: T,
        w: &E,
    ) -> Option<EdgeIndex> {
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
    pub fn find_node_edge_indices<T: PartialEq<N> + Debug>(&self, l: T, r: T) -> Vec<EdgeIndex> {
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
    pub fn has_node_edge<T: PartialEq<N> + Debug>(&self, l: T, r: T, w: &E) -> bool {
        self.find_node_edge_index(l, r, w).is_some()
    }
    /// True if edge with weight between NodeIndices
    pub fn has_edge(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> bool {
        self.find_edge_index(li, ri, w).is_some()
    }
    /// Get endpoint of edge
    pub fn edge_endpoint_directed(&self, i: EdgeIndex, direction: Direction) -> Option<NodeIndex> {
        self.edge_endpoints(i)
            .map(|(source, target)| match direction {
                Direction::Outgoing => source,
                Direction::Incoming => target,
            })
    }
    /// Get source of edge
    pub fn edge_source(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoint_directed(i, Direction::Outgoing)
    }
    /// Get target of edge
    pub fn edge_target(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoint_directed(i, Direction::Incoming)
    }
    /// Get sources of edges
    pub fn edge_sources_for(
        &'a self,
        is: impl IntoIterator<Item = EdgeIndex> + 'a,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        is.into_iter().filter_map(move |i| self.edge_source(i))
    }
    /// Get targets of edges
    pub fn edge_targets_for(
        &'a self,
        is: impl IntoIterator<Item = EdgeIndex> + 'a,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        is.into_iter().filter_map(move |i| self.edge_target(i))
    }
    /// Get weights of edges
    pub fn edge_weights_for(
        &'a self,
        is: impl IntoIterator<Item = EdgeIndex> + 'a,
    ) -> impl Iterator<Item = E> + 'a {
        is.into_iter()
            .filter_map(move |i| self.edge_weight(i).cloned())
    }
    /// Write graph to dot file
    pub fn write_to_file<S: Into<PathBuf>>(&self, name: S) -> std::io::Result<()> {
        let mut path: PathBuf = name.into();
        path.set_extension("dot");
        //path.canonicalize()?;
        path.parent().map(std::fs::create_dir_all);
        std::fs::write(path, format!("{:?}", Dot::new(&self.graph)))
    }
}
impl<'a, N: NodeData> Graph<N, usize> {
    /// Group EdgeIndices by distances
    pub fn group_edges_by_distance(
        &self,
        es: impl Iterator<Item = EdgeIndex>,
    ) -> Vec<Vec<EdgeIndex>> {
        let es = es
            .filter_map(|i| Some((i, self.edge_weight(i).cloned()?)))
            .collect::<Vec<_>>();
        let mut r = Vec::new();
        if let Some(&max) = es.iter().map(|(_, d)| d).max() {
            for d in 1..=max {
                r.push(
                    es.iter()
                        .filter(|(_, dist)| *dist == d)
                        .map(|(i, _)| *i)
                        .collect(),
                )
            }
        }
        r
    }
    /// Map function to groups
    pub fn map_groups<A, T, R: IntoIterator<Item = T> + 'a, B: IntoIterator<Item = A>>(
        gs: impl IntoIterator<Item = B>,
        f: impl Fn(B) -> R,
    ) -> Vec<Vec<T>> {
        gs.into_iter()
            .map(move |group| f(group).into_iter().collect())
            .collect()
    }
    pub fn distance_group_sources(
        &self,
        es: impl Iterator<Item = EdgeIndex>,
    ) -> Vec<Vec<NodeIndex>> {
        Self::map_groups(self.group_edges_by_distance(es), move |group| {
            self.edge_sources_for(group)
        })
    }
    pub fn distance_group_targets(
        &self,
        es: impl Iterator<Item = EdgeIndex>,
    ) -> Vec<Vec<NodeIndex>> {
        Self::map_groups(self.group_edges_by_distance(es), move |group| {
            self.edge_targets_for(group)
        })
    }
    pub fn distance_group_source_weights(
        &self,
        es: impl Iterator<Item = EdgeIndex>,
    ) -> Vec<Vec<N>> {
        Self::map_groups(self.distance_group_sources(es), move |group| {
            self.node_weights_for(group)
        })
    }
    pub fn distance_group_target_weights(
        &self,
        es: impl Iterator<Item = EdgeIndex>,
    ) -> Vec<Vec<N>> {
        Self::map_groups(self.distance_group_targets(es), move |group| {
            self.node_weights_for(group)
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
        static ref G: Mutex<Graph<char, usize>> = Mutex::new(Graph::default());
        static ref NODE_INDICES: Vec<NodeIndex> = {
            let mut g = G.lock().unwrap();
            ELEMS.iter().map(|e| g.node(e)).collect()
        };
        static ref EDGE_INDICES: Vec<EdgeIndex> = {
            let mut g = G.lock().unwrap();
            EDGES
                .iter()
                .map(|(l, r, w)| g.node_edge(l, r, *w))
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
            assert!(g.has_node(*e));
        }
    }
    #[test]
    fn has_node_edge() {
        let g = init();
        for (l, r, w) in EDGES.iter() {
            assert!(g.has_node_edge(*l, *r, w));
        }
    }
    #[test]
    fn write_to_file() {
        let g = init();
        g.write_to_file("test_graph").unwrap();
    }
}
