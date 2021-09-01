use crate::{
    hypergraph::{
        VertexIndex,
        Pattern,
        PatternView,
        Hypergraph,
        split::Split,
        Child,
        search::FoundRange,
        index_splitter::{
            IndexSplit,
        },
    },
    token::Tokenize,
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SplitMinimizer;
impl SplitMinimizer {
    pub fn minimize_index_split<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, index_split: IndexSplit) -> IndexSplit  {
        index_split
    }
    //fn perform_child_splits<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &mut Hypergraph<T>, child_splits: Vec<SplitContext>) -> IndexSplit {
    //    // todo: create new index for multiple patterns in splits or minimize patterns
    //    let left = Self::merge_contexts_with_splits(
    //        hypergraph,
    //        left,
    //        |context, split| [context, split].concat()
    //        );
    //    let right = Self::merge_contexts_with_splits(
    //        hypergraph,
    //        right,
    //        |context, split| [split, context].concat()
    //        );
    //    (left, right)
    //}
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
    fn create_replace_child<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        left: &Child,
        right: &Child
    ) -> Child {
        //println!("{},{} dont't match {}: {:#?}",
        //    hypergraph.index_string(left.index),
        //    hypergraph.index_string(right.index),
        //    hypergraph.index_string(index),
        //    index_match
        //);
        // create new index and replace in parent
        let width = left.width + right.width;
        let new_index = hypergraph.insert_pattern([left, right]);
        // TODO: replace new index in parent!
        Child::new(new_index, width)
    }
    fn get_or_create_replace_child<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        found: &Option<(VertexIndex, FoundRange)>,
        left: &Child,
        right: &Child,
        ) -> Child {
        let width = left.width + right.width;
        if let Some((found_index, found_range)) = found {
            if found_range.is_matching() {
                //println!("{},{} match {}",
                //    hypergraph.index_string(left.index),
                //    hypergraph.index_string(right.index),
                //    hypergraph.index_string(index),
                //);
                // since left and right half are minimal, any pattern perfectly matching
                // first left and right indices from merge pos must be the maximum fitting
                return Child::new(*found_index, width);
            }
        } else {
            //println!("{},{} dont't have a common parent",
            //    hypergraph.index_string(left.index),
            //    hypergraph.index_string(right.index),
            //);
        }
        let replace = Self::create_replace_child(hypergraph, left, right);
        if let Some((found_index, found_range)) = found {
            let found_index = *found_index;
            match found_range {
                FoundRange::Postfix(pre) => {
                    hypergraph.add_pattern_to_node(found_index, &[&pre[..], &[replace]].concat());
                },
                FoundRange::Prefix(post) => {
                    hypergraph.add_pattern_to_node(found_index, &[&[replace], &post[..]].concat());
                },
                FoundRange::Infix(pre, post) => {
                    hypergraph.add_pattern_to_node(found_index, &[&pre[..], &[replace], &post[..]].concat());
                },
                FoundRange::Complete => unreachable!(),
            }
        }
        replace
    }
    fn replace_in_pattern(
        pattern: PatternView<'_>,
        pos: usize,
        replace: Child
    ) -> Pattern {
        let after = (pos+1).min(pattern.len());
        [&pattern[..pos-1], &[replace], &pattern[after..]].concat()
    }
    /// minimize a pattern which has been merged at pos
    fn minimize_merged_pattern<T: Tokenize + std::fmt::Display>(
        hypergraph: &mut Hypergraph<T>,
        pattern: Pattern,
        pos: usize,
        ) -> Pattern {
        if !(1..pattern.len()).contains(&pos) {
            return pattern;
        }
        //println!("pos: {}, len: {}", pos, pattern.len());
        let left = &pattern[pos - 1];
        let right = &pattern[pos];
        // find pattern over merge position
        let found = hypergraph.find_pattern(&pattern[pos-1..pos+1]);
        let replace = Self::get_or_create_replace_child(hypergraph, &found, left, right);
        Self::replace_in_pattern(&pattern, pos, replace)
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
        //println!("before: {:#?}", splits);
        splits.sort_unstable();
        splits.dedup();
        //println!("after: {:#?}", splits);
        if splits.len() > 1 {
            //splits.iter().for_each(|pat|
            //println!("{}",
            //    hypergraph.pattern_string(pat),
            //));
            let index = hypergraph.insert_patterns(splits);
            //println!("Created index for: {}",
            //    hypergraph.index_string(index),
            //);
            let width = hypergraph.index_width(&index);
            let child = Child::new(index, width);
            vec![child]
        } else {
            splits.into_iter().next().unwrap_or_default()
        }
    }
}
#[cfg(test)]
mod tests {
}