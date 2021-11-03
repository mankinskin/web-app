use crate::{
    hypergraph::{
        split::*,
        Indexed,
        VertexIndex,
    },
};
use std::{
    cmp::PartialEq,
    num::NonZeroUsize,
};
mod single;
pub use single::*;
mod range;
pub use range::*;

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

#[derive(Debug, Clone, PartialEq)]
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
        self.pattern().expect("called SplitSegment::unwrap_pattern on a `Child` value")
    }
    pub fn unwrap_child(self) -> Child {
        self.child().expect("called SplitSegment::unwrap_child on a `Pattern` value")
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Child(_) => 1,
            Self::Pattern(p) => {
                let l = p.len();
                assert!(l != 1, "SplitSegment with len = 1 should be a Child!");
                l
            },
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
            Self::Pattern(p) => p.as_slice()
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct IndexSplitter;
impl IndexSplitter {
}

#[cfg(test)]
mod tests {
}