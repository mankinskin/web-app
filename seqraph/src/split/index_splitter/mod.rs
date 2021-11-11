use crate::{
    split::*,
    Indexed,
    VertexIndex,
};
use std::{
    cmp::PartialEq,
    num::NonZeroUsize,
};
mod single;
pub use single::*;
mod range;
pub use range::*;

pub type Split = (Pattern, Pattern);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitContext {
    pub prefix: Pattern,
    pub key: SplitKey,
    pub postfix: Pattern,
}
/// refers to an index in a hypergraph node
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IndexInParent {
    pub pattern_index: usize,  // index of pattern in parent
    pub replaced_index: usize, // replaced index in pattern
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SplitIndex {
    pos: TokenPosition,
    index: VertexIndex,
    index_pos: IndexPosition,
}
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct PatternSplit {
    pub(crate) prefix: Pattern,
    pub(crate) inner: IndexSplit,
    pub(crate) postfix: Pattern,
}
impl PatternSplit {
    pub fn new(prefix: Pattern, inner: impl Into<IndexSplit>, postfix: Pattern) -> Self {
        Self {
            prefix,
            inner: inner.into(),
            postfix,
        }
    }
}
#[derive(Debug, Clone, Eq, Ord, PartialOrd, Default)]
pub struct IndexSplit {
    pub(crate) splits: Vec<PatternSplit>,
}
impl IndexSplit {
    pub fn new(inner: impl IntoIterator<Item = impl Into<PatternSplit>>) -> Self {
        Self {
            splits: inner.into_iter().map(Into::into).collect(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.splits.is_empty()
    }
    pub fn add_split<T: Into<PatternSplit>>(&mut self, split: T) {
        self.splits.push(split.into());
    }
}
impl PartialEq for IndexSplit {
    fn eq(&self, other: &Self) -> bool {
        let a: BTreeSet<_> = self.splits.iter().collect();
        let b: BTreeSet<_> = other.splits.iter().collect();
        a == b
    }
}
impl From<Split> for PatternSplit {
    fn from((prefix, postfix): Split) -> Self {
        Self {
            prefix,
            inner: Default::default(),
            postfix,
        }
    }
}
impl<T: Into<IndexSplit>> From<(Pattern, T, Pattern)> for PatternSplit {
    fn from((prefix, inner, postfix): (Pattern, T, Pattern)) -> Self {
        Self::new(prefix, inner, postfix)
    }
}
impl<T: Into<PatternSplit>> From<Vec<T>> for IndexSplit {
    fn from(splits: Vec<T>) -> Self {
        Self {
            splits: splits.into_iter().map(Into::into).collect(),
        }
    }
}
impl<T: Into<PatternSplit>> From<T> for IndexSplit {
    fn from(split: T) -> Self {
        Self::from(vec![split])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct SplitKey {
    pub index: VertexIndex, // index in hypergraph
    pub offset: NonZeroUsize,
}
impl SplitKey {
    pub fn new(index: impl Indexed, offset: NonZeroUsize) -> Self {
        Self {
            index: *index.index(),
            offset,
        }
    }
}
pub enum RangeSplitResult {
    Full(Child),
    Single(SplitSegment, SplitSegment),
    Double(SplitSegment, SplitSegment, SplitSegment),
    None,
}
pub type SingleSplitResult = (SplitSegment, SplitSegment);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SplitSegment {
    Pattern(Pattern),
    Child(Child),
}
impl SplitSegment {
    pub fn pattern(self) -> Option<Pattern> {
        match self {
            Self::Child(_) => None,
            Self::Pattern(p) => Some(p),
        }
    }
    pub fn child(self) -> Option<Child> {
        match self {
            Self::Pattern(_) => None,
            Self::Child(c) => Some(c),
        }
    }
    pub fn map_pattern(self, f: impl FnOnce(Pattern) -> Pattern) -> Self {
        match self {
            Self::Pattern(p) => Self::Pattern(f(p)),
            _ => self,
        }
    }
    pub fn map_child(self, f: impl FnOnce(Child) -> Child) -> Self {
        match self {
            Self::Child(c) => Self::Child(f(c)),
            _ => self,
        }
    }
    pub fn unwrap_pattern(self) -> Pattern {
        self.pattern()
            .expect("called SplitSegment::unwrap_pattern on a `Child` value")
    }
    pub fn unwrap_child(self) -> Child {
        self.child()
            .expect("called SplitSegment::unwrap_child on a `Pattern` value")
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Child(_) => 1,
            Self::Pattern(p) => {
                let l = p.len();
                assert!(l != 1, "SplitSegment with len = 1 should be a Child!");
                l
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Child(_) => false,
            Self::Pattern(p) => p.is_empty(),
        }
    }
}
impl From<Result<Child, Pattern>> for SplitSegment {
    fn from(r: Result<Child, Pattern>) -> Self {
        match r {
            Ok(c) => Self::Child(c),
            Err(p) => Self::Pattern(p),
        }
    }
}
impl From<Child> for SplitSegment {
    fn from(c: Child) -> Self {
        Self::Child(c)
    }
}
impl From<Pattern> for SplitSegment {
    fn from(p: Pattern) -> Self {
        if p.len() == 1 {
            (*p.first().unwrap()).into()
        } else {
            Self::Pattern(p)
        }
    }
}
impl IntoIterator for SplitSegment {
    type Item = Child;
    type IntoIter = std::vec::IntoIter<Child>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Pattern(p) => p.into_iter(),
            Self::Child(c) => vec![c].into_iter(),
        }
    }
}
impl IntoPattern for SplitSegment {
    type Token = Child;
    fn as_pattern_view(&'_ self) -> &'_ [Self::Token] {
        match self {
            Self::Child(c) => std::slice::from_ref(c),
            Self::Pattern(p) => p.as_slice(),
        }
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

pub type DoublePerfectSplitIndex = (PatternId, Pattern, usize, Pattern, usize, Pattern);

pub enum DoubleSplitPositions {
    None,
    Single(NonZeroUsize),
    Double(NonZeroUsize, NonZeroUsize),
}
pub enum DoubleSplitIndex {
    Left(Pattern, usize, Pattern, SplitKey, Pattern),
    Right(Pattern, SplitKey, Pattern, usize, Pattern),
    Infix(Pattern, SplitKey, Pattern, SplitKey, Pattern),
    Inner(Pattern, (VertexIndex, NonZeroUsize, NonZeroUsize), Pattern),
}
pub type DoubleSplitIndices = Result<DoublePerfectSplitIndex, Vec<(PatternId, DoubleSplitIndex)>>;
pub type SingleSplitIndices = Vec<(PatternId, SplitIndex)>;

#[derive(Debug)]
pub struct IndexSplitter<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'g, T: Tokenize + 'g> IndexSplitter<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    /// Get perfect split if it exists and remaining pattern split contexts
    pub(crate) fn separate_perfect_split(
        &'g self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let current_node = self.graph.expect_vertex_data(root);
        let children = current_node.get_children().clone();
        let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
        let split_indices = Self::find_single_split_indices(child_slices, pos);
        Self::separate_single_split_indices(current_node, split_indices)
    }
    // Get perfect split or pattern split contexts
    //pub(crate) fn try_perfect_split(
    //    &self,
    //    root: impl Indexed,
    //    pos: NonZeroUsize,
    //) -> Result<(Split, IndexInParent), Vec<SplitContext>> {
    //    let current_node = self.get_vertex_data(root).unwrap();
    //    let children = current_node.get_children().clone();
    //    let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
    //    let split_indices = IndexSplitter::find_single_split_indices(child_slices, pos);
    //    match IndexSplitter::perfect_split_search(current_node, split_indices)
    //        .into_iter()
    //        .collect()
    //    {
    //        Ok(s) => Err(s),
    //        Err(s) => Ok(s),
    //    }
    //}
}
