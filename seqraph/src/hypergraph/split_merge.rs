use crate::{
    hypergraph::{
        VertexIndex,
        Pattern,
        Hypergraph,
        split::Split,
        Child,
        pattern_width,
        search::FoundRange,
    },
    token::Tokenize,
};
use indexmap::{
    IndexMap,
};
use std::num::NonZeroUsize;

/// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContextChild {
    pub child: usize, // child node in the sub graph
    pub index_in_parent: IndexInParent,
}
/// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContextParent {
    pub parent: usize, // parent node in the sub graph
    pub index_in_parent: IndexInParent,
}
/// refers to an index in a hypergraph node
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IndexInParent {
    pub pattern_index: usize, // index of pattern in parent
    pub replaced_index: usize, // replaced index in pattern
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitContext {
    pub context: (Pattern, Pattern),
    pub key: SplitKey,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitKey {
    pub index: VertexIndex, // index in hypergraph
    pub offset: NonZeroUsize,
}
impl SplitKey {
    pub fn new(index: VertexIndex, offset: NonZeroUsize) -> Self {
        Self {
            index,
            offset,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SplitMerge {
    pub cache: IndexMap<SplitKey, Split>,
}
impl SplitMerge {
    pub fn split<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, root: VertexIndex, pos: NonZeroUsize) -> Split  {
        let mut s = Self::default();
        hypergraph.try_perfect_split(root, pos)
            .unwrap_or_else(|split_indices| {
                let splits = s.merge_child_splits(hypergraph, split_indices);
                Self::minimize_split_patterns(hypergraph, splits)
            })
    }
    // merge child splits with
    fn merge_child_splits<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &mut Hypergraph<T>, child_splits: Vec<SplitContext>) -> (Vec<Pattern>, Vec<Pattern>) {
        let (left, right): (Vec<_>, Vec<_>) = child_splits.into_iter().map(|SplitContext {
            context: (left_context, right_context),
            key,
        }| {
            // recurse
            let (left_split, right_split) = self.get_node_split(hypergraph, key);
            ((left_context, left_split), (right_context, right_split))
        })
        .unzip();
        // todo: create new index for multiple patterns in splits or minimize patterns
        let left = Self::merge_contexts_with_splits(
            hypergraph,
            left,
            |context, split| {
                [context, split].concat()
            });
        let right = Self::merge_contexts_with_splits(
            hypergraph,
            right,
            |context, split| [split, context].concat()
            );
        (left, right)
    }
    fn get_node_split<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &mut Hypergraph<T>, key: SplitKey) -> Split {
        let name = hypergraph.index_string(key.index);
        // don't merge existing indices again
        self.cache.get(&key).cloned()
            .map(|r| { println!("got cached split for: {}", name); r })
            .unwrap_or_else(|| {
                // todo: insert remaining patterns if perfect split has more than one index on one side
                let (perfect_split, remaining_splits) = hypergraph.separate_perfect_split(key.index, key.offset);

                let (left, right) = if let Some((pl, pr)) = perfect_split {
                    if (pl.len() <= 1 || pattern_width(&pl) <= 2) && (pr.len() <= 1 || pattern_width(&pr) <= 2) {
                        (vec![pl], vec![pr])
                    } else { 
                        let (mut left, mut right) = self.merge_child_splits(hypergraph, remaining_splits);
                        left.push(pl);
                        right.push(pr);
                        (left, right)
                    }
                } else {
                    self.merge_child_splits(hypergraph, remaining_splits)
                };
                let (left, right) = Self::minimize_split_patterns(hypergraph, (left, right));
                println!("Split: {} =>", name);
                println!("left:\n\t{}", hypergraph.separated_pattern_string(&left));
                println!("right:\n\t{}", hypergraph.separated_pattern_string(&right));
                let split = (left, right);
                self.cache_split_node(key, split.clone());
                split
            })
    }
    fn merge_contexts_with_splits<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, contexts: Vec<(Pattern, Pattern)>, merge_fn: impl Fn(Pattern, Pattern) -> Pattern) -> Vec<Pattern> {
        contexts.into_iter()
            .map(|(context, split)| {
                    let pos = context.len();
                    let merged = merge_fn(context, split);
                    Self::minimize_merged_pattern(hypergraph, merged, pos)
                    //merged
            })
            .collect()
    }
    fn get_or_create_replace_child<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        found_index: VertexIndex,
        found_range: FoundRange,
        left: &Child,
        right: &Child,
        ) -> Child {
        let width = left.width + right.width;
        match found_range {
            FoundRange::Complete => {
                //println!("{},{} match {}",
                //    hypergraph.index_string(left.index),
                //    hypergraph.index_string(right.index),
                //    hypergraph.index_string(index),
                //);
                // since left and right half are minimal, any pattern perfectly matching
                // first left and right indices from merge pos must be the maximum fitting
                Child::new(found_index, width)
            },
            FoundRange::Postfix(_) |
            FoundRange::Prefix(_) |
            FoundRange::Infix(_, _) => {
                //println!("{},{} dont't match {}: {:#?}",
                //    hypergraph.index_string(left.index),
                //    hypergraph.index_string(right.index),
                //    hypergraph.index_string(index),
                //    index_match
                //);
                // create new index and replace in parent
                let new_index = hypergraph.insert_pattern([left, right]);
                // TODO: replace new index in parent!
                Child::new(new_index, width)
            },
        }
    }
    /// minimize a pattern which has been merged at pos
    fn minimize_merged_pattern<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, pattern: Pattern, pos: usize) -> Pattern {
        if !(1..pattern.len()).contains(&pos) {
            return pattern;
        }
        //println!("pos: {}, len: {}", pos, pattern.len());
        let left = &pattern[pos - 1];
        let right = &pattern[pos];
        // find pattern over merge position
        hypergraph.find_pattern(&pattern[pos-1..pos+1])
            .map(|(index, found_range)| {
                let replace = Self::get_or_create_replace_child(hypergraph, index, found_range, left, right);
                let after = (pos+1).min(pattern.len());
                let minimized = [&pattern[..pos-1], &[replace], &pattern[after..]].concat();
                minimized
            })
            .unwrap_or_else(|| {
                println!("{},{} dont't have a common parent",
                    hypergraph.index_string(left.index),
                    hypergraph.index_string(right.index),
                );
                pattern.clone()
            })
    }
    fn minimize_split_patterns<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, (left_splits, right_splits): (Vec<Pattern>, Vec<Pattern>)) -> Split {
        let left = Self::minimize_split_half(hypergraph, left_splits);
        let right = Self::minimize_split_half(hypergraph, right_splits);
        (left, right)
    }
    fn minimize_split_half<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, mut splits: Vec<Pattern>) -> Pattern {
        if splits.len() <= 1 {
            return splits.into_iter().next().unwrap_or_default();
        }
        // todo: filter duplicates, index intersections
        println!("before: {:#?}", splits);
        splits.sort_unstable();
        splits.dedup();
        println!("after: {:#?}", splits);
        if splits.len() > 1 {
            splits.iter().for_each(|pat|
            println!("{}",
                hypergraph.pattern_string(pat),
            ));
            let index = hypergraph.insert_patterns(splits);
            println!("Created index for: {}",
                hypergraph.index_string(index),
            );
            let width = hypergraph.index_width(index);
            let child = Child::new(index, width);
            vec![child]
        } else {
            splits.into_iter().next().unwrap_or_default()
        }
    }
    fn cache_split_node(&mut self, key: SplitKey, split: Split) {
        self.cache.insert(key, split);
    }
}
#[cfg(test)]
mod tests {
}