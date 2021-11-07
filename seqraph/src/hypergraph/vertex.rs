use crate::{
    hypergraph::{
        pattern::*,
        Hypergraph,
        search::*,
    },
    token::*,
};
use either::Either;
use std::{borrow::Borrow, collections::{
        HashMap,
        HashSet,
    }, fmt::Debug, hash::Hasher, slice::SliceIndex, sync::atomic::{
        AtomicUsize,
        Ordering,
    }};

use super::NewTokenIndex;

pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type PatternId = usize;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type VertexPatternView<'a> = Vec<&'a VertexData>;

pub trait Indexed {
    fn index(&self) -> &VertexIndex;
    fn vertex<'g, T: Tokenize>(&'g self, graph: &'g Hypergraph<T>) -> &'g VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl<I: Borrow<VertexIndex>> Indexed for I {
    fn index(&self) -> &VertexIndex {
        (*self).borrow()
    }
}

pub trait ToChild: Indexed + Wide {
    fn to_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide> ToChild for T {}

pub trait MaybeIndexed<T: Tokenize> {
    type Inner: Indexed;
    fn into_inner(self) -> Result<Self::Inner, T>;
}
impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for Result<I, T> {
    type Inner = I;
    fn into_inner(self) -> Result<Self::Inner, T> {
        self
    }
}
//impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for I {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        Ok(self)
//    }
//}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex),
}
/// Storage for parent relationship of a child to a parent
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    /// width of the parent
    pub width: TokenPosition,
    /// positions of child in parent patterns
    pub pattern_indices: HashSet<(PatternId, usize)>,
}
impl Parent {
    pub fn new(width: TokenPosition) -> Self {
        Self {
            width,
            pattern_indices: Default::default(),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn add_pattern_index(&mut self, pattern: usize, index: PatternId) {
        self.pattern_indices.insert((pattern, index));
    }
    pub fn remove_pattern_index(&mut self, pattern: usize, index: PatternId) {
        self.pattern_indices.remove(&(pattern, index));
    }
    pub fn exists_at_pos(&self, p: usize) -> bool {
        self.pattern_indices.iter().any(|(_, pos)| *pos == p)
    }
    pub fn exists_at_pos_in_pattern(&self, pat: PatternId, pos: usize) -> bool {
        self.pattern_indices.contains(&(pat, pos))
    }
    /// filter for pattern indices which occur at start of their patterns
    pub fn filter_pattern_indicies_at_prefix(&self) -> impl Iterator<Item = &(PatternId, usize)> {
        self.pattern_indices
            .iter()
            .filter(move |(_pattern_index, sub_index)| *sub_index == 0)
    }
    /// filter for pattern indices which occur at end of given patterns
    pub fn filter_pattern_indicies_at_end_in_patterns<'a>(
        &'a self,
        patterns: &'a HashMap<PatternId, Pattern>,
    ) -> impl Iterator<Item = &'a (PatternId, usize)> {
        self.pattern_indices
            .iter()
            .filter(move |(pattern_index, sub_index)| {
                *sub_index + 1
                    == patterns
                        .get(pattern_index)
                        .expect("Pattern index not in patterns!")
                        .len()
            })
    }
    // filter for pattern indices which occur in given patterns
    //pub fn filter_pattern_indicies_in_patterns<'a>(
    //    &'a self,
    //    patterns: &'a HashMap<PatternId, Pattern>,
    //) -> impl Iterator<Item = &'a (PatternId, usize)> {
    //    self.pattern_indices
    //        .iter()
    //        .filter(move |(pattern_index, sub_index)| {
    //            *sub_index
    //                == patterns
    //                    .get(pattern_index)
    //                    .expect("Pattern index not in patterns!")
    //        })
    //}
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct Child {
    pub index: VertexIndex,   // the child index
    pub width: TokenPosition, // the token width
}
impl Child {
    #[allow(unused)]
    pub(crate) const INVALID: Child = Child { index: 0, width: 0, };
    pub fn new(index: impl Indexed, width: TokenPosition) -> Self {
        Self {
            index: *index.index(),
            width,
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_index(&self) -> VertexIndex {
        self.index
    }
}
impl std::cmp::PartialOrd for Child {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl Wide for Child {
    fn width(&self) -> usize {
        self.width
    }
}
impl std::hash::Hash for Child {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.index.hash(h);
    }
}
impl std::cmp::Ord for Child {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}
impl PartialEq for Child {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl PartialEq<VertexIndex> for Child {
    fn eq(&self, other: &VertexIndex) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ Child {
    fn eq(&self, other: &VertexIndex) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ mut Child {
    fn eq(&self, other: &VertexIndex) -> bool {
        self.index == *other
    }
}
impl Borrow<VertexIndex> for Child {
    fn borrow(&self) -> &VertexIndex {
        &self.index
    }
}
impl Borrow<VertexIndex> for &'_ Child {
    fn borrow(&self) -> &VertexIndex {
        &self.index
    }
}
impl Borrow<VertexIndex> for &'_ mut Child {
    fn borrow(&self) -> &VertexIndex {
        &self.index
    }
}
impl<T: Into<Child> + Clone> From<&'_ T> for Child {
    fn from(o: &'_ T) -> Self {
        (*o).clone().into()
    }
}
impl From<NewTokenIndex> for Child {
    fn from(o: NewTokenIndex) -> Self {
        Self::new(o.index(), 1)
    }
}
impl IntoIterator for Child {
    type Item = Self;
    type IntoIter = std::iter::Once<Child>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
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
        self.parents.get(index).ok_or(NotFound::NoMatchingParent(*index))
    }
    pub fn get_parents(&self) -> &VertexParents {
        &self.parents
    }
    pub fn get_child_pattern_range<R: SliceIndex<[Child]>>(
        &self,
        id: &PatternId,
        range: R,
    ) -> Result<&<R as SliceIndex<[Child]>>::Output, NotFound> {
        self.children.get(id)
            .and_then(|p| p.get(range))
            .ok_or(NotFound::NoChildPatterns)
    }
    pub fn get_child_pattern_position(&self, id: &PatternId, pos: IndexPosition) -> Result<&Child, NotFound> {
        self.children.get(id)
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
    pub fn add_parent(
        &mut self,
        parent: impl ToChild,
        pattern: usize,
        index: PatternId,
    ) {
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
        self.children.iter().find_map(|(id, pat)| {
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