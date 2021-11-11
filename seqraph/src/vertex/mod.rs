use crate::{
    graph::*,
    read::*,
    search::*,
};
use either::Either;
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    slice::SliceIndex,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

mod indexed;
mod parent_child;
mod pattern;
mod pattern_stream;
mod token;
pub use {
    indexed::*,
    parent_child::*,
    pattern::*,
    pattern_stream::*,
    token::*,
};

pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type PatternId = usize;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type VertexPatternView<'a> = Vec<&'a VertexData>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    pub index: VertexIndex,
    pub width: TokenPosition,
    pub parents: VertexParents,
    pub children: ChildPatterns,
}
impl VertexData {
    fn next_child_pattern_id() -> PatternId {
        static PATTERN_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        PATTERN_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn new(index: VertexIndex, width: TokenPosition) -> Self {
        Self {
            index,
            width,
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_parent(&self, index: impl Indexed) -> Result<&Parent, NotFound> {
        let index = index.index();
        self.parents
            .get(index)
            .ok_or(NotFound::NoMatchingParent(*index))
    }
    pub fn get_parents(&self) -> &VertexParents {
        &self.parents
    }
    pub fn get_child_pattern_range<R: SliceIndex<[Child]>>(
        &self,
        id: &PatternId,
        range: R,
    ) -> Result<&<R as SliceIndex<[Child]>>::Output, NotFound> {
        self.children
            .get(id)
            .and_then(|p| p.get(range))
            .ok_or(NotFound::NoChildPatterns)
    }
    pub fn get_child_pattern_position(
        &self,
        id: &PatternId,
        pos: IndexPosition,
    ) -> Result<&Child, NotFound> {
        self.children
            .get(id)
            .and_then(|p| p.get(pos))
            .ok_or(NotFound::NoChildPatterns)
    }
    pub fn get_child_pattern(&self, id: &PatternId) -> Option<&Pattern> {
        self.children.get(id)
    }
    pub fn get_child_pattern_mut(&mut self, id: &PatternId) -> Result<&mut Pattern, NotFound> {
        self.children.get_mut(id).ok_or(NotFound::NoChildPatterns)
    }
    pub fn expect_any_pattern(&self) -> &Pattern {
        self.children
            .values()
            .next()
            .unwrap_or_else(|| panic!("Pattern vertex has no children {:#?}", self,))
    }
    pub fn expect_child_pattern(&self, id: &PatternId) -> &Pattern {
        self.get_child_pattern(id).unwrap_or_else(|| {
            panic!(
                "Child pattern with id {} does not exist in in vertex {:#?}",
                id, self,
            )
        })
    }
    pub fn expect_child_pattern_mut(&mut self, id: &PatternId) -> &mut Pattern {
        self.get_child_pattern_mut(id)
            .unwrap_or_else(|_| panic!("Child pattern with id {} does not exist in in vertex", id,))
    }
    pub fn get_children(&self) -> &ChildPatterns {
        &self.children
    }
    pub fn add_pattern<P: IntoIterator<Item = impl Into<Child>>>(&mut self, pat: P) -> PatternId {
        // TODO: detect unmatching pattern
        let id = Self::next_child_pattern_id();
        let pat = pat.into_iter().map(Into::into).collect();
        self.children.insert(id, pat);
        id
    }
    pub fn add_parent(&mut self, parent: impl ToChild, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(parent.index()) {
            parent.add_pattern_index(pattern, index);
        } else {
            let mut parent_rel = Parent::new(parent.width());
            parent_rel.add_pattern_index(pattern, index);
            self.parents.insert(*parent.index(), parent_rel);
        }
    }
    pub fn remove_parent(&mut self, vertex: impl Indexed, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(vertex.index()) {
            if parent.pattern_indices.len() > 1 {
                parent.remove_pattern_index(pattern, index);
            } else {
                self.parents.remove(vertex.index());
            }
        }
    }
    pub fn get_parents_below_width(
        &self,
        width_ceiling: Option<TokenPosition>,
    ) -> impl Iterator<Item = (&VertexIndex, &Parent)> + Clone {
        let parents = self.get_parents();
        // optionally filter parents by width
        if let Some(ceil) = width_ceiling {
            Either::Left(
                parents
                    .iter()
                    .filter(move |(_, parent)| parent.get_width() < ceil),
            )
        } else {
            Either::Right(parents.iter())
        }
    }
    pub fn to_pattern_strings<T: Tokenize + std::fmt::Display>(
        &self,
        g: &Hypergraph<T>,
    ) -> Vec<Vec<String>> {
        self.get_children()
            .values()
            .map(|pat| {
                pat.iter()
                    .map(|c| g.index_string(c.index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
    pub fn filter_parent(
        &self,
        parent_index: impl Indexed,
        cond: impl Fn(&&Parent) -> bool,
    ) -> Result<&'_ Parent, NotFound> {
        let index = parent_index.index();
        Some(self.get_parent(index)?)
            .filter(cond)
            .ok_or(NotFound::NoMatchingParent(*index))
    }
    pub fn get_parent_starting_at(
        &self,
        parent_index: impl Indexed,
        offset: PatternId,
    ) -> Result<&'_ Parent, NotFound> {
        self.filter_parent(parent_index, |parent| parent.exists_at_pos(offset))
    }
    pub fn get_parent_ending_at(
        &self,
        parent_index: impl Indexed,
        offset: PatternId,
    ) -> Result<&'_ Parent, NotFound> {
        self.filter_parent(parent_index, |parent| {
            offset
                .checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        })
    }
    pub fn get_parent_at_prefix_of(&self, index: impl Indexed) -> Result<&'_ Parent, NotFound> {
        self.get_parent_starting_at(index, 0)
    }
    pub fn get_parent_at_postfix_of(&self, index: impl Indexed) -> Result<&'_ Parent, NotFound> {
        self.filter_parent(index, |parent| {
            parent
                .width
                .checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        })
    }
    pub fn find_pattern_with_range<T: Tokenize + std::fmt::Display>(
        &self,
        half: Pattern,
        range: impl PatternRangeIndex + Clone,
    ) -> Result<PatternId, NotFound> {
        self.children
            .iter()
            .find_map(|(id, pat)| {
                if pat[range.clone()] == half[..] {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or(NotFound::NoChildPatterns)
    }
    /// replace indices in sub pattern and returns old indices
    /// doesn't modify parents of sub-patterns!
    pub(crate) fn replace_in_pattern(
        &mut self,
        pat: PatternId,
        range: impl PatternRangeIndex + Clone,
        replace: impl IntoIterator<Item = Child>,
    ) -> Pattern {
        let pattern = self.expect_child_pattern_mut(&pat);
        let old = pattern
            .get(range.clone())
            .expect("Replace range out of range of pattern!")
            .to_vec();
        *pattern = replace_in_pattern(pattern.iter().cloned(), range, replace);
        old
    }
}

impl Indexed for VertexData {
    fn index(&self) -> &VertexIndex {
        &self.index
    }
    fn vertex<'g, T: Tokenize>(&'g self, _graph: &'g Hypergraph<T>) -> &'g VertexData {
        self
    }
}
//impl Borrow<VertexIndex> for VertexData {
//    fn borrow(&self) -> &VertexIndex {
//        &self.index
//    }
//}
impl Borrow<VertexIndex> for &VertexData {
    fn borrow(&self) -> &VertexIndex {
        &self.index
    }
}
