use crate::hypergraph::VertexIndex;
use std::num::NonZeroUsize;
// describes search position of child in tree
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexPositionDescriptor {
    pub parent: Option<TreeParent>,
    pub node: VertexIndex,
    pub offset: NonZeroUsize,
}
// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeParent {
    pub tree_node: usize,
    pub index_in_parent: IndexInParent,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexInParent {
    pub pattern_index: usize,
    pub replaced_index: usize,
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct PathDescriptor {
    index: VertexIndex,
    index_in_parent: IndexInParent
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PathTree {
    parents: PathTreeParents
}
type PathTreeParents = Vec<(Option<TreeParent>, VertexIndex)>;
impl PathTree {
    pub fn new() -> Self {
        Self {
            parents: Default::default(),
        }
    }
    pub fn get_parents(&self) -> &PathTreeParents {
        &self.parents
    }
    pub fn get_parents_mut(&mut self) -> &mut PathTreeParents {
        &mut self.parents
    }
    pub fn into_parents(self) -> PathTreeParents {
        self.parents
    }
    pub fn add_element(&mut self, parent: Option<TreeParent>, index: VertexIndex) -> usize {
        let id = self.parents.len();
        self.parents.push((parent, index));
        id
    }
    pub fn add_child_of(&mut self, parent: TreeParent, index: VertexIndex) -> usize {
        self.add_element(Some(parent), index)
    }
}
#[cfg(test)]
mod tests {
}