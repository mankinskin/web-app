use crate::{
    hypergraph::{
        split::*,
        Hypergraph,
        Indexed,
        VertexIndex,
    },
    token::Tokenize,
};
use std::{
    borrow::Borrow,
    cmp::PartialEq,
    num::NonZeroUsize,
};

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

#[derive(Debug, Clone)]
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
    pub(crate) fn index_subrange<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed + Clone,
        range: impl PatternRangeIndex,
    ) -> RangeSplitResult {
        let root = root.index();
        //println!("splitting {} at {:?}", hypergraph.index_string(root), range);
        let vertex = hypergraph.expect_vertex_data(root).clone();
        // range is a subrange of the index
        let patterns = vertex.get_children().clone();
        match Self::verify_range_split_indices(vertex.width, range) {
            DoubleSplitPositions::Double(lower, higher) => {
                // both positions in position in pattern
                let (left, inner, right) =
                    match Self::build_double_split_kinds(&vertex, patterns, lower, higher) {
                        Ok((pattern_id, pre, left, inner, right, post)) => {
                            // perfect split
                            (
                                Self::resolve_perfect_split_range(
                                    hypergraph,
                                    pre,
                                    root,
                                    pattern_id,
                                    0..left,
                                ),
                                Self::resolve_perfect_split_range(
                                    hypergraph,
                                    inner,
                                    root,
                                    pattern_id,
                                    left..right,
                                ),
                                Self::resolve_perfect_split_range(
                                    hypergraph,
                                    post,
                                    root,
                                    pattern_id,
                                    right..,
                                ),
                            )
                        }
                        Err(indices) => {
                            // unperfect splits
                            let (left, inner, right) = Self::double_split_from_indices(hypergraph, root, indices);
                            (left, SplitSegment::Child(inner), right)
                        }
                    };
                RangeSplitResult::Double(left, inner, right)
            }
            DoubleSplitPositions::Single(single) => {
                // only a single position in pattern
                let single = Self::find_single_split_indices(patterns, single);
                Self::process_single_splits(hypergraph, &vertex, root, single)
            }
            DoubleSplitPositions::None => RangeSplitResult::Full(Child::new(root, vertex.width)),
        }
    }
    // build intermediate split kind for multiple patterns
    pub(crate) fn build_double_split_kinds(
        current_node: &VertexData,
        patterns: impl IntoIterator<Item = (PatternId, impl IntoIterator<Item = Child> + Clone)>,
        left: NonZeroUsize,
        right: NonZeroUsize,
    ) -> DoubleSplitIndices {
        match patterns
            .into_iter()
            .try_fold(vec![], move |mut acc, (pattern_index, pattern)| {
                let left_split = Self::find_pattern_split_index(pattern.clone(), left)
                    .expect("left split not in pattern");
                let right_split = Self::find_pattern_split_index(pattern, right)
                    .expect("right split not in pattern");
                let left = Self::separate_pattern_split(pattern_index, left_split);
                let right = Self::separate_pattern_split(pattern_index, right_split);
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                match (left, right) {
                    (Ok((left, left_ind)), Ok((right, right_ind))) => {
                        // both unperfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let new = match right_index - left_index {
                            0 => {
                                let (prefix, postfix) = split_pattern_at_index(pattern, left_index);
                                (
                                    pattern_index,
                                    DoubleSplitIndex::Inner(
                                        prefix,
                                        (left.index, left.offset, right.offset),
                                        postfix,
                                    ),
                                )
                            }
                            _ => {
                                let (prefix, infix, postfix) =
                                    double_split_context(pattern, left_index, right_index);
                                (
                                    pattern_index,
                                    DoubleSplitIndex::Infix(prefix, left, infix, right, postfix),
                                )
                            }
                        };
                        acc.push(new);
                        Ok(acc)
                    }
                    (Ok((left, left_ind)), Err(right_ind)) => {
                        // only right perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_context(pattern, left_index);
                        let (infix, postfix) =
                            split_pattern_at_index(&rem, right_index - left_index);
                        let new = (
                            pattern_index,
                            DoubleSplitIndex::Right(prefix, left, infix, right_index, postfix),
                        );
                        acc.push(new);
                        Ok(acc)
                    }
                    (Err(left_ind), Ok((right, right_ind))) => {
                        // only left perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_pattern_at_index(pattern, left_index);
                        let (infix, postfix) = split_context(&rem, right_index - left_index);
                        let new = (
                            pattern_index,
                            DoubleSplitIndex::Left(prefix, left_index, infix, right, postfix),
                        );
                        acc.push(new);
                        Ok(acc)
                    }
                    (Err(left_ind), Err(right_ind)) => {
                        // both perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_pattern_at_index(pattern, left_index);
                        let (infix, postfix) =
                            split_pattern_at_index(&rem, right_index - left_index);
                        Err((
                            pattern_index,
                            prefix,
                            left_index,
                            infix,
                            right_index,
                            postfix,
                        ))
                    }
                }
            }) {
            Ok(indices) => Err(indices),
            Err(split) => Ok(split),
        }
    }
    pub(crate) fn double_split_from_indices<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        parent: impl Indexed,
        indices: Vec<(PatternId, DoubleSplitIndex)>,
    ) -> (SplitSegment, Child, SplitSegment) {
        let parent = parent.index();
        // for every child split
        let (left, inner, right) = indices.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut la, mut ia, mut ra), (_pattern_id, split_index)| {
                match split_index {
                    DoubleSplitIndex::Left(pre, _, infix, single, post) => {
                        let (l, r) = Self::split_index(hypergraph, single.index, single.offset);
                        la.push((pre, None));
                        ia.push((None, SplitSegment::Pattern(infix), Some(l)));
                        ra.push((post, Some(r)));
                    }
                    DoubleSplitIndex::Right(pre, single, infix, _, post) => {
                        let (l, r) = Self::split_index(hypergraph, single.index, single.offset);
                        la.push((pre, Some(l)));
                        ia.push((Some(r), SplitSegment::Pattern(infix), None));
                        ra.push((post, None));
                    }
                    DoubleSplitIndex::Infix(pre, left, infix, right, post) => {
                        let (ll, lr) = Self::split_index(hypergraph, left.index, left.offset);
                        let (rl, rr) = Self::split_index(hypergraph, right.index, right.offset);
                        la.push((pre, Some(ll)));
                        ia.push((Some(lr), SplitSegment::Pattern(infix), Some(rl)));
                        ra.push((post, Some(rr)));
                    }
                    DoubleSplitIndex::Inner(pre, (index, left, right), post) => {
                        match Self::index_subrange(hypergraph, index, left.get()..right.get()) {
                            RangeSplitResult::Double(l, i, r) => {
                                la.push((pre, Some(l)));
                                ia.push((None, i, None));
                                ra.push((post, Some(r)));
                            }
                            RangeSplitResult::Single(l, r) => {
                                la.push((pre, Some(l)));
                                ra.push((post, Some(r)));
                            }
                            RangeSplitResult::Full(c) => {
                                la.push((pre, None));
                                ia.push((None, SplitSegment::Child(c), None));
                                ra.push((post, None));
                            }
                            RangeSplitResult::None => {
                                la.push((pre, None));
                                ra.push((post, None));
                            }
                        }
                    }
                }
                (la, ia, ra)
            },
        );
        let mut minimizer = SplitMinimizer::new(hypergraph);
        let left = minimizer.merge_left_optional_splits(left);
        let inner = minimizer.merge_inner_optional_splits(inner);
        let right = minimizer.merge_right_optional_splits(right);
        // split all children and resolve
        //println!(
        //    "adding ({}, {}, {}) to {}",
        //    hypergraph.index_string(left),
        //    hypergraph.index_string(inner),
        //    hypergraph.index_string(right),
        //    hypergraph.index_string(parent),
        //);
        hypergraph.add_pattern_to_node(parent,
            left.clone().into_iter().chain(inner).chain(right.clone())
        );
        (left, inner, right)
    }
    /// Find single split indicies and positions of multiple patterns
    pub fn find_single_split_indices(
        patterns: impl IntoIterator<Item = (PatternId, impl IntoIterator<Item = Child>)>,
        pos: NonZeroUsize,
    ) -> SingleSplitIndices {
        patterns
            .into_iter()
            .map(move |(i, pattern)| {
                let split =
                    Self::find_pattern_split_index(pattern, pos).expect("Split not in pattern");
                (i, split)
            })
            .collect()
    }
    pub(crate) fn process_single_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        vertex: &VertexData,
        root: impl Indexed + Clone,
        single: impl IntoIterator<Item = (PatternId, SplitIndex)>,
    ) -> RangeSplitResult {
        let (perfect_split, remaining_splits) = Self::separate_single_split_indices(vertex, single);
        let (_, (left, right)) = Self::single_split_from_indices(hypergraph, root, perfect_split, remaining_splits);
        RangeSplitResult::Single(left, right)
    }
    pub(crate) fn separate_single_split_indices(
        current_node: &VertexData,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)>,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let len = current_node.get_children().len();
        Self::perfect_split_search(current_node, split_indices)
            .into_iter()
            .fold((None, Vec::with_capacity(len)), |(pa, mut sa), r| match r {
                Ok(s) => {
                    sa.push(s);
                    (pa, sa)
                }
                Err(s) => (Some(s), sa),
            })
    }
    pub(crate) fn verify_range_split_indices(
        width: usize,
        range: impl PatternRangeIndex,
    ) -> DoubleSplitPositions {
        if range.contains(&0) && range.contains(&width) {
            return DoubleSplitPositions::None;
        }
        let lower = if let Bound::Included(&lo) = range.start_bound() {
            lo
        } else if let Bound::Excluded(&lo) = range.start_bound() {
            lo.checked_sub(1).unwrap_or_default()
        } else {
            0
        };
        let higher = if let Bound::Included(&hi) = range.end_bound() {
            hi.checked_add(1).unwrap_or(width)
        } else if let Bound::Excluded(&hi) = range.end_bound() {
            hi
        } else {
            width
        };
        if let Some(lower) = NonZeroUsize::new(lower) {
            match NonZeroUsize::new(higher).ok_or(lower) {
                Ok(higher) => {
                    if higher.get() < width {
                        DoubleSplitPositions::Double(lower, higher)
                    } else {
                        DoubleSplitPositions::Single(lower)
                    }
                }
                Err(lower) => DoubleSplitPositions::Single(lower),
            }
        } else {
            // lower bound out
            DoubleSplitPositions::Single(
                NonZeroUsize::new(higher).expect("upper bound is zero dispite check"),
            )
        }
    }
    pub fn index_prefix<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Child, SplitSegment) {
        let (pid, (l, r)) = Self::split_index_with_pid(hypergraph, root.index(), pos);
        match l {
            SplitSegment::Child(c) => (c, r),
            SplitSegment::Pattern(p) => {
                let len = p.len();
                let c = hypergraph.insert_pattern(p);
                if let Some(pid) = pid {
                    hypergraph.replace_in_pattern(root, pid, 0..len, c);
                }
                (c, r)
            },
        }
    }
    pub fn index_postfix<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (SplitSegment, Child) {
        let (pid, (l, r)) = Self::split_index_with_pid(hypergraph, root.index(), pos);
        match r {
            SplitSegment::Child(c) => (l, c),
            SplitSegment::Pattern(p) => {
                let c = hypergraph.insert_pattern(p);
                if let Some(pid) = pid {
                    hypergraph.replace_in_pattern(root, pid, l.len().., c);
                }
                (l, c)
            },
        }
    }
    pub(crate) fn split_index_with_pid<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<PatternId>, SingleSplitResult) {
        let root = root.index();
        //println!("splitting {} at {}", hypergraph.index_string(root), pos);
        let (perfect_split, remaining_splits) = hypergraph.separate_perfect_split(root, pos);
        Self::single_split_from_indices(hypergraph, root, perfect_split, remaining_splits)
    }
    pub(crate) fn split_index<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> SingleSplitResult {
        Self::split_index_with_pid(hypergraph, root, pos).1
    }
    pub(crate) fn single_split_from_indices<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        perfect_split: Option<(Split, IndexInParent)>,
        remaining_splits: Vec<SplitContext>,
    ) -> (Option<PatternId>, SingleSplitResult) {
        if let Some(ps) = perfect_split {
            let (pid, (left, right)) = Self::perform_perfect_split(hypergraph, ps, root);
            (Some(pid), (left, right))
        } else {
            // split all children and resolve
            let (pid, (left, right)) = Self::perform_child_splits(hypergraph, remaining_splits);
            hypergraph.add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
            (pid, (left, right))
        }
    }
    fn perform_child_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        child_splits: Vec<SplitContext>,
    ) -> (Option<PatternId>, SingleSplitResult) {
        // for every child split
        let (left, right) = child_splits
            .into_iter()
            .map(
                |SplitContext {
                     prefix,
                     key,
                     postfix,
                 }| {
                    // recurse
                    let (l, r) = Self::split_index(hypergraph, key.index, key.offset);
                    ((prefix, l), (postfix, r))
                },
            )
            .unzip();
        let mut minimizer = SplitMinimizer::new(hypergraph);
        let left = minimizer.merge_left_splits(left);
        let right = minimizer.merge_right_splits(right);
        (None, (left, right))
    }
    fn perform_perfect_split<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        ((pl, pr), ind): (Split, IndexInParent),
        parent: impl Indexed,
    ) -> (PatternId, SingleSplitResult) {
        // if other patterns can't add any more overlapping splits
        let parent = parent.index();
        (ind.pattern_index, (
            Self::resolve_perfect_split_range(
                hypergraph,
                pl,
                parent,
                ind.pattern_index,
                0..ind.replaced_index,
            ),
            Self::resolve_perfect_split_range(
                hypergraph,
                pr,
                parent,
                ind.pattern_index,
                ind.replaced_index..,
            ),
        ))
    }
    fn resolve_perfect_split_range<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        pat: Pattern,
        parent: impl Indexed,
        pattern_index: PatternId,
        range: impl PatternRangeIndex + Clone,
    ) -> SplitSegment {
        if pat.len() <= 1 {
            SplitSegment::Child(*pat.first().expect("Empty perfect split half!"))
        } else if parent.vertex(hypergraph).children.len() == 1 {
            SplitSegment::Pattern(pat)
        } else {
            let c = hypergraph.insert_pattern(pat);
            hypergraph.replace_in_pattern(parent, pattern_index, range, [c]);
            SplitSegment::Child(c)
        }
    }
    /// find split position in index in pattern
    pub fn find_pattern_split_index(
        pattern: impl IntoIterator<Item = impl Borrow<Child>>,
        pos: NonZeroUsize,
    ) -> Option<SplitIndex> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or
        pattern.into_iter().enumerate().find_map(|(i, child)| {
            let child = child.borrow();
            if skipped + child.get_width() <= pos {
                skipped += child.get_width();
                None
            } else {
                Some(SplitIndex {
                    index_pos: i,
                    pos: pos - skipped,
                    index: child.index,
                })
            }
        })
    }

    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn perfect_split_search<'a>(
        current_node: &'a VertexData,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)> + 'a,
    ) -> impl IntoIterator<Item = Result<SplitContext, (Split, IndexInParent)>> + 'a {
        split_indices
            .into_iter()
            .map(move |(pattern_index, split_index)| {
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                Self::separate_pattern_split(pattern_index, split_index)
                    .map(
                        move |(
                            key,
                            IndexInParent {
                                replaced_index: split_index,
                                ..
                            },
                        )| {
                            let (prefix, postfix) = split_context(pattern, split_index);
                            SplitContext {
                                prefix,
                                key,
                                postfix,
                            }
                        },
                    )
                    .map_err(
                        |ind
                         @
                         IndexInParent {
                             replaced_index: split_index,
                             ..
                         }| {
                            (split_pattern_at_index(pattern, split_index), ind)
                        },
                    )
            })
    }
    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn separate_pattern_split(
        pattern_index: PatternId,
        split_index: SplitIndex,
    ) -> Result<(SplitKey, IndexInParent), IndexInParent> {
        let SplitIndex {
            index_pos,
            pos,
            index,
        } = split_index;
        let index_in_parent = IndexInParent {
            pattern_index,
            replaced_index: index_pos,
        };
        NonZeroUsize::new(pos)
            .map(|offset| (SplitKey::new(index, offset), index_in_parent.clone()))
            .ok_or(index_in_parent)
    }
}
#[cfg(test)]
mod tests {}
