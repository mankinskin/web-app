use crate::{
    split::*,
    token::Tokenize,
    Hypergraph,
    Indexed,
};
use std::num::NonZeroUsize;
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
                            let (left, inner, right) =
                                Self::double_split_from_indices(hypergraph, root, indices);
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
    pub(crate) fn resolve_perfect_split_range<T: Tokenize>(
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
    // build intermediate split kind for multiple patterns
    fn build_double_split_kinds(
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
    fn double_split_from_indices<T: Tokenize>(
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
        hypergraph.add_pattern_to_node(
            parent,
            left.clone().into_iter().chain(inner).chain(right.clone()),
        );
        (left, inner, right)
    }
    fn verify_range_split_indices(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;
    #[test]
    fn split_range_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, w, x, y, z] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let wx = graph.insert_pattern([w, x]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let wxab = graph.insert_patterns([vec![wx, ab], vec![w, xab]]);
            let wxaby = graph.insert_patterns([vec![w, xaby], vec![wx, a, by], vec![wxab, y]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let wxabyz = graph.insert_patterns([vec![w, xabyz], vec![wxaby, z], vec![wx, ab, yz]]);

            let _ = graph.index_subrange(wxabyz, 0..);
            let _ = graph.index_subrange(wxabyz, 1..);
            let _ = graph.index_subrange(wxabyz, 1..3);
            let _ = graph.index_subrange(wxabyz, 2..5);
            let _ = graph.index_subrange(wxabyz, 3..);
        } else {
            panic!();
        }
    }
}
