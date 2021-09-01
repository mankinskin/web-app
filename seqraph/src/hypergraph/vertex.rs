use std::{fmt::Debug, hash::Hasher};
use crate::{
    token::{
        Token,
        Tokenize,
    },
    hypergraph::Hypergraph,
};
use std::collections::{
    HashSet,
    HashMap,
};
use std::borrow::Borrow;
use std::slice::SliceIndex;
use either::Either;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type Pattern = Vec<Child>;
pub type PatternId = usize;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type PatternView<'a> = &'a[Child];
pub type VertexPatternView<'a> = Vec<&'a VertexData>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex)
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    pub width: TokenPosition,
    pub pattern_indices: HashSet<(usize, usize)>, // positions of child in parent patterns
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
    pub fn exists_at_pos(&self, p: PatternId) -> bool {
        self.pattern_indices.iter().any(|(_, pos)| *pos == p)
    }
    pub fn filter_pattern_indicies_at_prefix(
        &self,
        ) -> impl Iterator<Item=&(PatternId, usize)> {
        self.pattern_indices.iter()
            .filter(move |(_pattern_index, sub_index)| *sub_index == 0)
    }
    pub fn filter_pattern_indicies_at_end_in_patterns<'a>(
        &'a self,
        patterns: &'a HashMap<PatternId, Pattern>,
        ) -> impl Iterator<Item=&'a (PatternId, usize)> {
        self.pattern_indices.iter()
            .filter(move |(pattern_index, sub_index)|
                *sub_index + 1 == patterns.get(pattern_index).expect("Pattern index not in patterns!").len()
            )
    }
}
#[derive(Debug, Eq, Clone, Copy)]
pub struct Child {
    pub index: VertexIndex, // the child index
    pub width: TokenPosition, // the token width
}
impl Child {
    pub fn new(index: impl Borrow<VertexIndex>, width: TokenPosition) -> Self {
        Self {
            index: *index.borrow(),
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

//impl<T: Borrow<Child>>  Borrow<Child> for &T {
//    fn borrow(&self) -> &VertexIndex {
//        &self.index
//    }
//}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    pub width: TokenPosition,
    pub parents: VertexParents,
    pub children: ChildPatterns,
}
impl VertexData {
    fn next_child_pattern_id() -> PatternId {
        static PATTERN_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        PATTERN_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn with_width(width: TokenPosition) -> Self {
        Self {
            width,
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_parent(&self, index: impl Borrow<VertexIndex>) -> Option<&Parent> {
        self.parents.get(index.borrow())
    }
    pub fn get_parents(&self) -> &VertexParents {
        &self.parents
    }
    pub fn get_child_pattern_range<R: SliceIndex<[Child]>>(&self, id: &PatternId, range: R) -> Option<&<R as SliceIndex<[Child]>>::Output> {
        self.children.get(id)?.get(range)
    }
    pub fn get_child_pattern_position(&self, id: &PatternId, pos: IndexPosition) -> Option<&Child> {
        self.children.get(id)?.get(pos)
    }
    pub fn get_child_pattern(&self, id: &PatternId) -> Option<&Pattern> {
        self.children.get(id)
    }
    pub fn expect_any_pattern(&self) -> &Pattern {
        self.children.values().next()
            .unwrap_or_else(|| panic!(
                "Pattern vertex has no children {:#?}",
                self,
            ))
    }
    pub fn expect_child_pattern(&self, id: &PatternId) -> &Pattern {
        self.children.get(id)
            .unwrap_or_else(|| panic!(
                "Child pattern with id {} does not exist in in vertex {:#?}",
                id, self,
            ))
    }
    pub fn get_children(&self) -> &ChildPatterns {
        &self.children
    }
    pub fn add_pattern<'c, P: IntoIterator<Item=&'c Child>>(&mut self, pat: P) -> PatternId {
        // TODO: detect unmatching pattern
        let id = Self::next_child_pattern_id();
        let pat = pat.into_iter().cloned().collect();
        self.children.insert(id, pat);
        id
    }
    pub fn add_parent(&mut self, vertex: impl Borrow<VertexIndex>, width: TokenPosition, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(vertex.borrow()) {
            parent.add_pattern_index(pattern, index);
        } else {
            let mut parent = Parent::new(width);
            parent.add_pattern_index(pattern, index);
            self.parents.insert(*vertex.borrow(), parent);
        }
    }
    pub fn remove_parent(&mut self, vertex: impl Borrow<VertexIndex>, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(vertex.borrow()) {
            if parent.pattern_indices.len() > 1 {
                parent.remove_pattern_index(pattern, index);
            } else {
                self.parents.remove(vertex.borrow());
            }
        }
    }
    pub fn get_parents_below_width(&self, width_ceiling: Option<TokenPosition>) -> impl Iterator<Item=(&VertexIndex, &Parent)> + Clone {
        let parents = self.get_parents();
        // optionally filter parents by width
        if let Some(ceil) = width_ceiling {
            Either::Left(parents.iter().filter(move |(_, parent)| parent.get_width() < ceil))
        } else {
            Either::Right(parents.iter())
        }
    }
    pub fn to_pattern_strings<T: Tokenize + std::fmt::Display>(&self, g: &Hypergraph<T>) -> Vec<Vec<String>> {
        self.get_children()
            .values()
            .map(|pat|
                pat.iter().map(|c| g.index_string(c.index)).collect::<Vec<_>>()
            )
            .collect::<Vec<_>>()
    }
    pub fn filter_parent(
        &self,
        parent_index: impl Borrow<VertexIndex>,
        cond: impl Fn(&&Parent) -> bool,
        ) -> Option<&'_ Parent> {
        self.get_parent(parent_index.borrow())
            .filter(cond)
    }
    pub fn get_parent_starting_at(
        &self,
        parent_index: impl Borrow<VertexIndex>,
        offset: PatternId,
        ) -> Option<&'_ Parent> {
        self.filter_parent(parent_index, |parent|
            parent.exists_at_pos(offset)
        )
    }
    pub fn get_parent_ending_at(
        &self,
        parent_index: impl Borrow<VertexIndex>,
        offset: PatternId,
        ) -> Option<&'_ Parent> {
        self.filter_parent(parent_index, |parent|
            offset.checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        )
    }
    pub fn get_parent_at_prefix_of(
        &self,
        index: impl Borrow<VertexIndex>,
        ) -> Option<&'_ Parent> {
        self.get_parent_starting_at(index, 0)
    }
    pub fn get_parent_at_postfix_of(
        &self,
        index: impl Borrow<VertexIndex>,
        ) -> Option<&'_ Parent> {
        self.filter_parent(index, |parent|
            parent.width.checked_sub(self.width)
                .map(|p| parent.exists_at_pos(p))
                .unwrap_or(false)
        )
    }
}