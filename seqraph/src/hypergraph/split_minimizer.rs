use crate::{
    hypergraph::{
        Hypergraph,
        Child,
        search::FoundRange,
        PatternId,
        Indexed,
        split::Split,
        index_splitter::{
            IndexSplit,
            SplitHalf,
            IndexInParent,
        },
        pattern::*,
    },
    token::Tokenize,
};
use std::{
    collections::HashSet,
};

trait MergeDirection {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)>;
    fn concat_with_context(context: Pattern, inner: Pattern) -> (Option<usize>, Pattern);
    fn concat_patterns(left: Pattern, right: Pattern) -> (Option<usize>, Pattern) {
        (Some(left.len()), left.into_iter().chain(right.into_iter()).collect())
    }
    // concat single context pattern with each inner pattern
    fn concat_context_with_inner<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        context: Pattern,
        inner: Vec<Pattern>,
        ) -> (Pattern, Vec<(Option<usize>, Pattern)>) {
        if inner.is_empty() {
            (vec![], if context.is_empty() {
                vec![]
            } else {
                vec![(None, context)]
            })
        } else {
            // if inner has multiple patterns, also create an index for those
            let new = if inner.len() > 1 {
                Some(hypergraph.insert_patterns(inner.clone()))
            } else {
                None
            };
            let (context, head, mut merged): (Pattern, Option<Child>, Vec<_>) = if let Some((head, tail)) = Self::split_context_head(context) {
                (tail, Some(head), inner.into_iter()
                    .map(|inner| Self::concat_with_context(vec![head], inner))
                    .collect())
            } else {
                // empty context
                (vec![], None, inner.into_iter()
                    .map(|inner| (None, inner))
                    .collect())
            };
            if let Some(new) = new {
                let new = if let Some(head) = head {
                    Self::concat_with_context(vec![head], vec![new]).1
                } else {
                    vec![new]
                };
                merged.push((None, new));
            }
            (context, merged)
        }
    }
    fn minimize_patterns<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        context: Pattern,
        inner: Vec<Pattern>,
        ) -> Vec<Pattern> {
        let (context, merged) = Self::concat_context_with_inner(hypergraph, context, inner);
        let minimized = SplitMinimizer::minimize_merged_patterns(hypergraph, merged);
        if !context.is_empty() {
            let new = hypergraph.insert_patterns(minimized);
            vec![Self::concat_with_context(context, vec![new]).1]
        } else {
            minimized
        }
    }
    fn merge_split_half<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, half: SplitHalf) -> Vec<Pattern>  {
        if half.inner.is_empty() {
            return vec![half.context];
        }
        let merged = Self::merge_split_halves(hypergraph, half.inner);
        Self::minimize_patterns(hypergraph, half.context, merged)
    }
    /// returns minimal pattern of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_split_halves<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, halves: Vec<SplitHalf>) -> Vec<Pattern>  {
        let mut acc: HashSet<Pattern> = Default::default();
        for half in halves.into_iter() {
            let merged = Self::merge_split_half(hypergraph, half);
            if let Some(first) = merged.first() {
                if first.len() == 1 {
                    return vec![first.clone()];
                }
            }
            acc.extend(merged);
        }
        acc.into_iter().collect()
    }
}
// context left, inner right
struct MergeLeft;
impl MergeDirection for MergeLeft {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)> {
        let mut context = context;
        let last = context.pop();
        last.map(|last| (last, context))
    }
    fn concat_with_context(context: Pattern, inner: Pattern) -> (Option<usize>, Pattern) {
        Self::concat_patterns(context, inner)
    }
}
// context right, inner left
struct MergeRight;
impl MergeDirection for MergeRight {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)> {
        let mut context = context.into_iter();
        let first = context.next();
        first.map(|first| (first, context.collect()))
    }
    fn concat_with_context(context: Pattern, inner: Pattern) -> (Option<usize>, Pattern) {
        Self::concat_patterns(inner, context)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SplitMinimizer;
impl SplitMinimizer {
    /// minimal means:
    /// - no two indicies are adjacient more than once
    /// - no two patterns of the same index share an index border
    /// returns minimal patterns on each side of index split
    pub fn minimize_index_split<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, split: Result<IndexSplit, (Split, IndexInParent)>, root: impl Indexed + Clone) -> (Child, Child)  {
        let split = split.map(|index_split| Self::merge_index_split(hypergraph, index_split));
        Self::resolve_split_halves(hypergraph, split, root)
    }
    fn resolve_split_halves<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        split: Result<(Vec<Pattern>, Vec<Pattern>), (Split, IndexInParent)>,
        parent: impl Indexed + Clone,
    ) -> (Child, Child) {
        match split {
            Ok(split) => Self::resolve_unperfect_split(hypergraph, split, parent),
            Err((split, pattern_id)) => Self::resolve_perfect_split(hypergraph, (split, pattern_id), parent),
        }
    }
    fn resolve_perfect_split<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        ((left, right), index_in_parent): (Split, IndexInParent),
        parent: impl Indexed + Clone,
    ) -> (Child, Child) {
        let left_single = single_child_pattern(left);
        let right_single = single_child_pattern(right);
        match (left_single, right_single) {
            // perfect split between single indices
            (Ok(left), Ok(right)) => (left, right),
            (Ok(left), Err(right)) => {
                let right = Self::resolve_perfect_split_half(
                    hypergraph,
                    right,
                    index_in_parent.pattern_index,
                    index_in_parent.replaced_index..,
                    parent,
                );
                (left, right)
            },
            (Err(left), Ok(right)) => {
                let left = Self::resolve_perfect_split_half(
                    hypergraph,
                    left,
                    index_in_parent.pattern_index,
                    0..index_in_parent.replaced_index,
                    parent,
                );
                (left, right)
            },
            (Err(left), Err(right)) => {
                let left = Self::resolve_perfect_split_half(
                    hypergraph,
                    left,
                    index_in_parent.pattern_index,
                    0..index_in_parent.replaced_index,
                    parent.clone(),
                );
                let right = Self::resolve_perfect_split_half(
                    hypergraph,
                    right,
                    index_in_parent.pattern_index,
                    index_in_parent.replaced_index..,
                    parent,
                );
                (left, right)
            },
        }
    }
    fn resolve_perfect_split_half<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        half: Pattern,
        pattern_id: PatternId,
        range: impl PatternRangeIndex + Clone,
        parent: impl Indexed,
    ) -> Child {
        let new = hypergraph.insert_pattern(half);
        hypergraph.replace_in_pattern(parent, pattern_id, range, [new]);
        new
    }
    fn resolve_unperfect_split_half<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        halves: Vec<Pattern>,
        parent: impl Indexed,
        order: impl Fn(Child) -> [Child; 2],
    ) -> Child {
        let new = hypergraph.insert_patterns(halves);
        let pattern = order(new);
        let parent = parent.index();
        let (pattern_id, width) = {
            let parent = hypergraph.expect_vertex_data_mut(parent);
            let pat = parent.add_pattern(pattern.iter());
            (pat, parent.width)
        };
        hypergraph.add_pattern_parent_with_width(parent, pattern, pattern_id, 0, width);
        new
    }
    fn resolve_unperfect_split<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        (left, right): (Vec<Pattern>, Vec<Pattern>),
        parent: impl Indexed,
    ) -> (Child, Child) {
        let left_single = single_child_patterns(left);
        let right_single = single_child_patterns(right);
        match (left_single, right_single) {
            // parent contains perfect split, no changes needed
            (Ok(left), Ok(right)) => {
                let parent = hypergraph.expect_vertex_data_mut(parent);
                parent.add_pattern([left, right]);
                (left, right)
            },
            (Ok(left), Err(right)) => {
                let right = Self::resolve_unperfect_split_half(
                    hypergraph,
                    right,
                    parent,
                    |right| [left, right]
                );
                (left, right)
            },
            (Err(left), Ok(right)) => {
                let left = Self::resolve_unperfect_split_half(
                    hypergraph,
                    left,
                    parent,
                    |left| [left, right]
                );
                (left, right)
            },
            (Err(left), Err(right)) => {
                let left = hypergraph.insert_patterns(left);
                let right = hypergraph.insert_patterns(right);
                let parent = hypergraph.expect_vertex_data_mut(parent);
                parent.add_pattern([left, right]);
                (left, right)
            },
        }
    }
    pub fn merge_index_split<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, index_split: IndexSplit) -> (Vec<Pattern>, Vec<Pattern>)  {
        let (left, right) = index_split.into_split_halves();
        (
            MergeLeft::merge_split_halves(hypergraph, left),
            MergeRight::merge_split_halves(hypergraph, right),
        )
    }
    /// minimize a pattern which has been merged at pos
    fn minimize_merged_patterns<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        patterns: impl IntoIterator<Item=(Option<usize>, Pattern)>,
        ) -> Vec<Pattern> {
        let patterns: Vec<_> = patterns.into_iter().collect();
        if patterns.len() > 1 {
            // more than one, so we created an index for these earlier, so we can
            // ignore any non-minimizable merged patterns
            patterns.into_iter()
                .filter_map(|(pos, pattern)| pos.and_then(|pos| Self::minimize_merged_pattern(hypergraph, pattern, pos)))
                .collect()
        } else if let Some((pos, first)) = patterns.into_iter().next() {
            vec![pos.and_then(|pos| Self::minimize_merged_pattern(hypergraph, first.clone(), pos))
                .unwrap_or(first)]
        } else {
            vec![]
        }
    }
    /// minimize a pattern which has been merged at pos
    fn minimize_merged_pattern<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        pattern: Pattern,
        pos: usize,
        ) -> Option<Pattern> {
        //println!("pos: {}, len: {}", pos, pattern.len());
        let left = &pattern[pos - 1];
        let right = &pattern[pos];
        // find pattern over merge position
        let found = hypergraph.find_pattern(&pattern[pos-1..pos+1]);
        found.map(|(found, (pattern_id, sub_index), found_range)| {
            let replace = match found_range {
                FoundRange::Postfix(_) |
                FoundRange::Prefix(_) |
                FoundRange::Infix(_, _) =>
                    Self::resolve_duplicate(hypergraph, found, pattern_id, sub_index, left, right),
                FoundRange::Complete => found,
            };
            replace_in_pattern(pattern.clone(), pos-1..pos+1, [replace])
        })
    }
    fn resolve_duplicate<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        parent: Child,
        pattern_id: PatternId,
        pos: usize,
        left: &Child,
        right: &Child,
    ) -> Child {
        // create new index and replace in parent
        let new = hypergraph.insert_pattern([left, right]);
        hypergraph.replace_in_pattern(parent, pattern_id, pos..pos+2, [new]);
        new
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        token::*,
        hypergraph::{
            index_splitter::*,
        },
    };
    use std::{
        num::NonZeroUsize,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn merge_split_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let cd = graph.insert_pattern([c, d]);
            let abc = graph.insert_patterns([
                vec![ab, c],
                vec![a, bc]
            ]);
            let bcd = graph.insert_patterns([
                vec![bc, d],
                vec![b, cd]
            ]);
            let abcd = graph.insert_patterns([
                vec![abc, d],
                vec![a, bcd]
            ]);
            let ab_pattern = vec![ab];
            let cd_pattern = vec![cd];

            let index_split = IndexSplitter::build_index_split(&graph, abcd, NonZeroUsize::new(2).unwrap());
            let (left, right) = SplitMinimizer::merge_index_split(&mut graph, index_split);
            assert_eq!(left, vec![ab_pattern], "left");
            assert_eq!(right, vec![cd_pattern], "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn merge_split_2() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('w'),
                Token::Element('x'),
                Token::Element('y'),
                Token::Element('z'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![x, a, by]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            let x_a_pattern = vec![x, a];
            let by_z_pattern = vec![by, z];

            let (left, right) = IndexSplitter::split(&mut graph, xabyz, NonZeroUsize::new(2).unwrap());
            let xa_found = graph.find_pattern(x_a_pattern);
            if let (xa, _, FoundRange::Complete) = xa_found.expect("xa not found") {
                assert_eq!(left, xa, "left");
            } else { panic!(); };
            let byz_found = graph.find_pattern(by_z_pattern);
            if let (byz, _, FoundRange::Complete) = byz_found.expect("byz not found") {
                assert_eq!(right, byz, "right");
            } else { panic!(); };
        } else {
            panic!();
        }
    }
}