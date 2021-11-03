use crate::{
    hypergraph::{
        split::*,
        Hypergraph,
        Indexed,
    },
    token::Tokenize,
};
use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

impl IndexSplitter {
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
    /// Find single split indicies and positions of multiple patterns
    pub(crate) fn find_single_split_indices(
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
    /// find split position in index in pattern
    pub(crate) fn find_pattern_split_index(
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
    fn single_split_from_indices<T: Tokenize>(
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
}
#[cfg(test)]
mod tests {}