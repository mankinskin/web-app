use crate::{
    split::*,
    vertex::*,
    *,
};
use std::collections::HashSet;

pub trait MergeDirection {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)>;
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)>;
    fn concat_inner_and_context(
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
    ) -> Pattern;
    fn merge_order(inner: Child, head: Child) -> (Child, Child);
    fn minimize_split<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        context: SplitSegment,
        inner: SplitSegment,
    ) -> SplitSegment {
        if let Some((outer_head, outer_tail)) = Self::split_context_head(context) {
            let (inner_head, inner_tail) =
                Self::split_inner_head(inner).expect("Empty SplitSegment::Pattern");
            let (left, right) = Self::merge_order(inner_head, outer_head);
            // try to find parent matching both
            SplitMinimizer::new(hypergraph)
                .resolve_common_parent(left, right)
                .map_err(|pat| Self::concat_inner_and_context(inner_tail, pat, outer_tail))
                .into()
        } else {
            // context empty
            inner
        }
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        Self::merge_optional_splits(hypergraph, splits.into_iter().map(|(p, c)| (p, Some(c))))
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_optional_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        splits: impl IntoIterator<Item = (Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (context, inner)| {
                if let Some(inner) = inner {
                    match Self::minimize_split(hypergraph, SplitSegment::Pattern(context), inner) {
                        // stop when single child is found
                        SplitSegment::Child(c) => Err(c),
                        SplitSegment::Pattern(pat) => {
                            acc.insert(pat);
                            Ok(acc)
                        }
                    }
                } else {
                    acc.insert(context);
                    Ok(acc)
                }
            }) {
            Err(child) => {
                //println!("adding [\n{}] to {}",
                //    patterns.clone().into_iter().fold(String::new(), |acc, p| {
                //        format!("{}{},\n", acc, hypergraph.pattern_string(p))
                //    }),
                //    hypergraph.index_string(child),
                //);
                //hypergraph.add_patterns_to_node(child, patterns);
                SplitSegment::Child(child)
            }
            Ok(patterns) => SplitSegment::Child(if patterns.len() == 1 {
                let pattern = patterns.into_iter().next().unwrap();
                hypergraph.insert_pattern(pattern)
            } else {
                hypergraph.insert_patterns(patterns)
            }),
        }
    }
}
// context left, inner right
pub struct MergeLeft;
impl MergeDirection for MergeLeft {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(mut p) => {
                let last = p.pop();
                last.map(|last| (last, p))
            }
            SplitSegment::Child(c) => Some((c, vec![])),
        }
    }
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        MergeRight::split_context_head(context)
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (head, inner)
    }
    fn concat_inner_and_context(
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
    ) -> Pattern {
        [outer_context, inner, inner_context].concat()
    }
}
// context right, inner left
pub struct MergeRight;
impl MergeDirection for MergeRight {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(p) => {
                let mut p = p.into_iter();
                let first = p.next();
                first.map(|last| (last, p.collect()))
            }
            SplitSegment::Child(c) => Some((c, vec![])),
        }
    }
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        MergeLeft::split_context_head(context)
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (inner, head)
    }
    fn concat_inner_and_context(
        inner_context: Pattern,
        inner: Pattern,
        outer_context: Pattern,
    ) -> Pattern {
        [inner_context, inner, outer_context].concat()
    }
}
