use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternId,
        Pattern,
        PatternView,
        VertexData,
        Parent,
        TokenPosition,
        ChildPatterns,
        Child,
        r#match::PatternMatch,
        prefix,
        postfix,
        search::FoundRange,
    },
    token::{
        Tokenize,
    },
};
use itertools::{
    Itertools,
    EitherOrBoth,
};
use std::collections::{
    HashSet,
    HashMap,
};

fn to_matching_iterator<'a>(a: impl Iterator<Item=&'a Child>, b: impl Iterator<Item=&'a Child>) -> impl Iterator<Item=(usize, EitherOrBoth<&'a Child, &'a Child>)> {
    a.zip_longest(b)
        .enumerate()
        .skip_while(|(_, eob)|
            match eob {
                EitherOrBoth::Both(a, b) => a == b,
                _ => false,
            }
        )
}
pub trait MatchDirection {
    fn get_match_parent(vertex: &VertexData, sup: VertexIndex) -> Option<&'_ Parent>;
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item=&'a Child>,
        b: impl DoubleEndedIterator<Item=&'a Child>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)>;
    // get remaining pattern in matching direction including index
    fn split_end<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern;
    fn split_end_normalized<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    // get remaining pattern in matching direction excluding index
    fn front_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern;
    fn front_context_normalized<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        Self::front_context(pattern, Self::normalize_index(pattern, index))
    }
    // get remaining pattern agains matching direction excluding index
    fn back_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern;
    fn back_context_normalized<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        Self::back_context(pattern, Self::normalize_index(pattern, index))
    }
    fn pattern_tail<'a>(pattern: PatternView<'a>) -> PatternView<'a>;
    fn pattern_head<'a>(pattern: PatternView<'a>) -> Option<&'a Child>;
    fn index_next(index: usize) -> Option<usize>;
    fn normalize_index<'a>(pattern: PatternView<'a>, index: usize) -> usize;
    fn merge_remainder_with_context<'a>(rem: PatternView<'a>, context: PatternView<'a>) -> Pattern;
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
        ) -> HashSet<(PatternId, usize)>;
    fn to_found_range(p: PatternMatch, context: Pattern) -> Option<FoundRange>;
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchRight;
impl MatchDirection for MatchRight {
    fn get_match_parent(vertex: &VertexData, sup: VertexIndex) -> Option<&'_ Parent> {
        vertex.get_parent_at_prefix_of(sup)
    }
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item=&'a Child>,
        b: impl DoubleEndedIterator<Item=&'a Child>,
        ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)> {
        to_matching_iterator(a, b).next()
    }
    fn split_end<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        postfix(pattern, index)
    }
    fn front_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        postfix(pattern, index+1)
    }
    fn back_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        prefix(pattern, index)
    }
    fn pattern_tail<'a>(pattern: PatternView<'a>) -> PatternView<'a> {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head<'a>(pattern: PatternView<'a>) -> Option<&'a Child> {
        pattern.first()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn normalize_index<'a>(_pattern: PatternView<'a>, index: usize) -> usize {
        index
    }
    fn merge_remainder_with_context<'a>(rem: PatternView<'a>, context: PatternView<'a>) -> Pattern {
        [rem, context].concat()
    }
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        _: &HashMap<PatternId, Pattern>,
        ) -> HashSet<(PatternId, usize)> {
        parent.filter_pattern_indicies_at_prefix().cloned().collect()
    }
    fn to_found_range(p: PatternMatch, context: Pattern) -> Option<FoundRange> {
        p.prepend_prefix(context)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchLeft;
impl MatchDirection for MatchLeft {
    fn get_match_parent(vertex: &VertexData, sup: VertexIndex) -> Option<&'_ Parent> {
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item=&'a Child>,
        b: impl DoubleEndedIterator<Item=&'a Child>,
        ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn split_end<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        prefix(pattern, index + 1)
    }
    fn front_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        prefix(pattern, index)
    }
    fn back_context<'a>(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> Pattern {
        postfix(pattern, index + 1)
    }
    fn pattern_tail<'a>(pattern: PatternView<'a>) -> PatternView<'a> {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head<'a>(pattern: PatternView<'a>) -> Option<&'a Child> {
        pattern.last()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn normalize_index<'a>(pattern: PatternView<'a>, index: usize) -> usize {
        pattern.len() - index - 1
    }
    fn merge_remainder_with_context<'a>(rem: PatternView<'a>, context: PatternView<'a>) -> Pattern {
        [context, rem].concat()
    }
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
        ) -> HashSet<(PatternId, usize)> {
        parent.filter_pattern_indicies_at_end_in_patterns(child_patterns).cloned().collect()
    }
    fn to_found_range(p: PatternMatch, context: Pattern) -> Option<FoundRange> {
        p.prepend_prefix(context).map(FoundRange::reverse)
    }
}

#[derive(Clone, Debug)]
pub struct Matcher<'g, T: Tokenize, D: MatchDirection>{
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::Deref for Matcher<'a, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<'a, T: Tokenize, D: MatchDirection> Matcher<'a, T, D> {
    pub fn new(graph: &'a Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    /// match sub_pat against children in parent with an optional offset.
    pub fn compare_parent_with_context(
        &self,
        context: PatternView<'a>,
        parent_index: VertexIndex,
        parent: &Parent,
        ) -> Option<FoundRange> {
        self.compare_parent_context(
            context,
            parent_index,
            parent,
            )
            .and_then(|(other_context, p)|
                D::to_found_range(p, other_context)
            )
    }

    fn compare_parent_matching_context_below_width(
        &self,
        context: PatternView<'a>,
        vertex: &VertexData,
        width_ceiling: Option<TokenPosition>,
        ) -> Option<(VertexIndex, Parent, PatternMatch)> {
        vertex.get_parents_below_width(width_ceiling)
        .find_map(|(&index, parent)|
            self.compare_parent_context(context, index, parent)
                .map(|(_pre, m)| (index, parent.clone(), m))
        )
    }
    fn pick_best_matching_child_pattern(
        child_patterns: &'a ChildPatterns,
        candidates: impl Iterator<Item=(usize, PatternId)>,
        sub_context: PatternView<'a>,
        ) -> Option<(Pattern, Pattern)> {
        candidates.find_or_first(|(pattern_index, sub_index)|
            D::pattern_head(sub_context).and_then(|next_sub|
                D::index_next(*sub_index).and_then(|i|
                    child_patterns.get(pattern_index)
                        .and_then(|pattern|
                            pattern.get(i).map(|next_sup| next_sub == next_sup)
                        )
                )
            ).unwrap_or(false)
        ).and_then(|(pattern_index, sub_index)|
            child_patterns.get(&pattern_index)
                .map(|pattern| (
                    D::back_context(pattern, sub_index),
                    D::split_end(pattern, sub_index)
                ))
        )
    }
    /// match sub_pat against children context in parent.
    pub fn compare_parent_context(
        &'a self,
        context: PatternView<'a>,
        parent_index: VertexIndex,
        parent: &Parent,
        ) -> Option<(Pattern, PatternMatch)> {
        // find pattern where sub is at offset
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub offset
        let candidates = D::candidate_parent_pattern_indices(parent, &child_patterns);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        let best_match = Self::pick_best_matching_child_pattern(
            &child_patterns,
            candidates.into_iter(),
            context,
        );
        best_match.and_then(|(back_context, match_pattern)| {
            //println!("comparing post sub pattern with remaining children of parent");
            let tail = D::pattern_tail(&match_pattern[..]);
            self.compare(context, tail).map(|pm| (back_context, pm))
            // returns result of matching sub with parent's children
        })
    }
    /// find an index at the prefix of a pattern
    fn match_sub_and_context_with_index(&self,
            sub: VertexIndex,
            context: PatternView<'_>,
            sup_index: VertexIndex,
            width: TokenPosition,
        ) -> Option<PatternMatch> {
        //println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub == sup_index {
            return if context.is_empty() {
                Some(PatternMatch::Matching)
            } else {
                Some(PatternMatch::SubRemainder(context.into()))
            }
        }
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().len() < 1 {
            return None;
        }
        let sup_parent = D::get_match_parent(vertex, sup_index);
        if let Some(parent) = sup_parent {
            // parents contain sup
            //println!("sup found in parents");
            let r = self.compare_parent_context(context, sup_index, parent).map(|(_rem, pm)| pm);
            r
        } else {
            // sup is no direct parent, search upwards
            //println!("matching available parents");
            // search sup in parents
            let (parent_index, _parent, index_match) = self.compare_parent_matching_context_below_width(
                context,
                &vertex,
                Some(width),
            )?;
            //println!("found parent matching");
            let new_context = match index_match {
                // found index for complete pattern, tr
                PatternMatch::Matching => Some(Vec::new()),
                // found matching parent larger than the pattern, not the one we were looking for
                PatternMatch::SupRemainder(_) => None,
                // found parent matching with prefix of pattern, continue
                PatternMatch::SubRemainder(rem) => Some(rem),
            }?;
            // TODO: faster way to handle empty new_post
            //println!("matching on parent with remainder");
            let r = self.match_sub_and_context_with_index(parent_index, &new_context, sup_index, width);
            r
        }
    }
    fn match_sub_and_sup_indices(&self, a: PatternView<'a>, ai: &'a Child, b: PatternView<'a>, bi: &'a Child, pos: TokenPosition) -> Option<PatternMatch> {
        //println!("matching \"{}\" and \"{}\"", self.sub_index_string(index_a), self.sub_index_string(index_b));
        // Note: depending on sizes of a, b it may be differently efficient
        // to search for children or parents, large patterns have less parents,
        // small patterns have less children
        // search larger in parents of smaller
        let sub_context;
        let sup_context;
        let sub;
        let sup;
        let sup_width;
        // remember if sub and sup were switched
        let rotate = if ai.width == bi.width {
            // relatives can not have same sizes
            return None;
        } else if ai.width < bi.width {
            //println!("right super");
            sub_context = D::front_context_normalized(a, pos);
            sup_context = D::front_context_normalized(b, pos);
            sub = ai.index;
            sup = bi.index;
            sup_width = bi.width;
            false
        } else {
            //println!("left super");
            sub_context = D::front_context_normalized(b, pos);
            sup_context = D::front_context_normalized(a, pos);
            sub = bi.index;
            sup = ai.index;
            sup_width = ai.width;
            true
        };
        let result = self.match_sub_and_context_with_index(
            sub,
            &sub_context[..],
            sup,
            sup_width,
        );
        // left remainder: sub remainder
        // right remainder: sup remainder
        // matching: sub & sup finished
        //println!("return {:#?}", result);
        let result = match result? {
            PatternMatch::SubRemainder(rem) =>
                self.compare(
                    &rem,
                    &sup_context,
                )?,
            PatternMatch::SupRemainder(rem) => PatternMatch::SupRemainder(
                D::merge_remainder_with_context(&rem, &sup_context)
            ),
            PatternMatch::Matching => {
                let rem = sup_context;
                if rem.is_empty() {
                    PatternMatch::Matching
                } else {
                    PatternMatch::SupRemainder(rem)
                }
            },
        };
        Some(if rotate {
            result.flip_remainder()
        } else {
            result
        })
    }
    pub fn compare(&self, a: PatternView<'_>, b: PatternView<'_>) -> Option<PatternMatch> {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        Some(if let Some((pos, eob)) = D::skip_equal_indices(a.iter(), b.iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(ai, bi) => self.match_sub_and_sup_indices(a, ai, b, bi, pos)?,
                EitherOrBoth::Left(_) => PatternMatch::SubRemainder(D::split_end_normalized(a, pos)),
                EitherOrBoth::Right(_) => PatternMatch::SupRemainder(D::split_end_normalized(b, pos)),
            }
        } else {
            PatternMatch::Matching
        })
    }
}