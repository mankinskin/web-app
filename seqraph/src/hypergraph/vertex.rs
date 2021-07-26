use std::fmt::Debug;
use crate::{
    token::{
        Token,
        Tokenize,
    },
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

pub(crate) type VertexIndex = usize;
pub(crate) type VertexParents = HashMap<VertexIndex, Parent>;
pub(crate) type ChildPatterns = HashMap<PatternId, Pattern>;
pub(crate) type ChildPatternView<'a> = &'a[PatternView<'a>];
pub(crate) type Pattern = Vec<Child>;
pub(crate) type PatternId = usize;
pub(crate) type TokenPosition = usize;
pub(crate) type IndexPosition = usize;
pub(crate) type IndexPattern = Vec<VertexIndex>;
pub(crate) type VertexPattern = Vec<VertexData>;
pub(crate) type PatternView<'a> = &'a[Child];
pub(crate) type VertexPatternView<'a> = Vec<&'a VertexData>;
pub(crate) type VertexPatternViewMut<'a> = Vec<&'a mut VertexData>;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex)
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    width: TokenPosition,
    pattern_indices: HashSet<(usize, PatternId)>, // positions of child in parent patterns
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
    pub fn get_pattern_index_candidates(
        &self,
        offset: Option<PatternId>,
        ) -> impl Iterator<Item=&(usize, PatternId)> {
        if let Some(offset) = offset {
            print!("at offset = {} ", offset);
            Either::Left(self.pattern_indices.iter()
                .filter(move |(_pattern_index, sub_index)| *sub_index == offset))
        } else {
            print!("at offset = 0");
            Either::Right(self.pattern_indices.iter())
        }
    }
}
#[derive(Debug, Eq, Clone, Hash)]
pub struct Child {
    index: VertexIndex, // the child index
    width: TokenPosition, // the token width
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
impl PartialEq for Child {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    width: TokenPosition,
    parents: VertexParents,
    children: ChildPatterns,
}
impl VertexData {
    fn next_child_pattern_id() -> PatternId {
        static PATTERN_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = PATTERN_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        id
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
    pub fn get_parent(&self, index: &VertexIndex) -> Option<&Parent> {
        self.parents.get(index)
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
            .expect(&format!(
                    "Pattern vertex has no children {:#?}",
                    self,
            ))
    }
    pub fn expect_child_pattern(&self, id: &PatternId) -> &Pattern {
        self.children.get(id)
            .expect(&format!(
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
        self.children.insert(id, pat.into_iter().cloned().collect());
        id
    }
    pub fn add_parent(&mut self, vertex: VertexIndex, width: TokenPosition, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(&vertex) {
            parent.add_pattern_index(pattern, index);
        } else {
            let mut parent = Parent::new(width);
            parent.add_pattern_index(pattern, index);
            self.parents.insert(vertex, parent);
        }
    }
    pub fn remove_parent(&mut self, vertex: VertexIndex, pattern: usize, index: PatternId) {
        if let Some(parent) = self.parents.get_mut(&vertex) {
            if parent.pattern_indices.len() > 1 {
                parent.remove_pattern_index(pattern, index);
            } else {
                self.parents.remove(&vertex);
            }
        }
    }
}