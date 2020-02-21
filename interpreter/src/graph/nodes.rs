use crate::graph::edges::*;
use crate::graph::*;

#[derive(Debug, Clone)]
pub struct GraphNodes<'a>  {
    graph: &'a TextGraph,
    indices: HashSet<NodeIndex>,
}


impl<'a> GraphNodes<'a> {
    pub fn new(graph: &'a TextGraph) -> Self {
        Self {
            graph,
            indices: HashSet::new()
        }
    }
    pub fn add(&mut self, node: NodeIndex)  {
        self.indices.insert(node);
    }
}
