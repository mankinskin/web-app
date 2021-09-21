use crate::{
    hypergraph::{
        pattern::*,
        r#match::*,
        search::*,
        Child,
        ChildPatterns,
        Hypergraph,
        Indexed,
        Parent,
        PatternId,
        TokenPosition,
    },
    token::Tokenize,
};
use itertools::EitherOrBoth;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Matcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::Deref for Matcher<'a, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize + 'a, D: MatchDirection> Matcher<'a, T, D> {
    pub fn new(graph: &'a Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn searcher(&self) -> Searcher<'a, T, D> {
        Searcher::new(self.graph)
    }
    // Outline:
    // matching two patterns of indices and
    // returning the remainder. Starting from left or right.
    // - skip equal indices
    // - once unequal, pick larger and smaller index
    // - search for larger in parents of smaller
    // - otherwise: try to find parent with best matching children
    pub fn compare(&self, a: PatternView<'_>, b: PatternView<'_>) -> MatchResult {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        if let Some((pos, eob)) = D::skip_equal_indices(a.iter(), b.iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(ai, bi) => {
                    self.match_unequal_indices_in_context(a, ai, b, bi, pos)
                },
                EitherOrBoth::Left(_) => {
                    Ok(PatternMatch::SubRemainder(D::split_end_normalized(a, pos)))
                },
                EitherOrBoth::Right(_) => {
                    Ok(PatternMatch::SupRemainder(D::split_end_normalized(b, pos)))
                },
            }
        } else {
            Ok(PatternMatch::Matching)
        }
    }
    fn match_unequal_indices_in_context(
        &self,
        a_pattern: PatternView<'a>,
        a: &'a Child,
        b_pattern: PatternView<'a>,
        b: &'a Child,
        pos: TokenPosition,
    ) -> MatchResult {
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
        let rotate = match a.width.cmp(&b.width) {
            // relatives can not have same sizes
            Ordering::Equal => return Err(PatternMismatch::Mismatch),
            Ordering::Less => {
                //println!("right super");
                sub_context = D::front_context_normalized(a_pattern, pos);
                sup_context = D::front_context_normalized(b_pattern, pos);
                sub = a.index;
                sup = b.index;
                sup_width = b.width;
                false
            }
            Ordering::Greater => {
                //println!("left super");
                sub_context = D::front_context_normalized(b_pattern, pos);
                sup_context = D::front_context_normalized(a_pattern, pos);
                sub = b.index;
                sup = a.index;
                sup_width = a.width;
                true
            }
        };
        // left remainder: sub remainder
        // right remainder: sup remainder
        // matching: sub & sup finished
        match self.match_sub_and_context_with_index(sub, &sub_context[..], sup, sup_width)? {
            PatternMatch::SubRemainder(rem) => self.compare(&rem, &sup_context),
            PatternMatch::SupRemainder(rem) => {
                Ok(PatternMatch::SupRemainder(D::merge_remainder_with_context(&rem, &sup_context)))
            },
            PatternMatch::Matching => {
                let rem = sup_context;
                Ok(if rem.is_empty() {
                    PatternMatch::Matching
                } else {
                    PatternMatch::SupRemainder(rem)
                })
            }
        }.map(|result| if rotate {
            result.flip_remainder()
        } else {
            result
        })
    }
    /// match sub index and context with sup index with max width
    fn match_sub_and_context_with_index(
        &self,
        sub: impl Indexed,
        context: PatternView<'_>,
        sup_index: impl Indexed,
        sup_width: TokenPosition,
    ) -> MatchResult {
        //println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub.index() == sup_index.index() {
            return if context.is_empty() {
                Ok(PatternMatch::Matching)
            } else {
                Ok(PatternMatch::SubRemainder(context.into()))
            };
        }
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().is_empty() {
            return Err(PatternMismatch::NoParents);
        }
        // get parent where vertex is at relevant position (prefix or postfix)
        if let Some(parent) = D::get_match_parent(vertex, sup_index.index()) {
            // found vertex in sup at relevant position
            //println!("sup found in parents");
            // compare context after vertex in parent
                self.compare_context_with_child_pattern(context, sup_index, parent)
                    .map(|(_pid, _rem, pm)| pm)
        } else {
            // sup is no direct parent, search upwards
            //println!("matching available parents");
            // search sup in parents
            self.searcher().find_parent_matching_context_below_width(context, vertex, Some(sup_width))
                .or(Err(PatternMismatch::NoMatchingParent))
                .and_then(|(parent_index, _parent, _pattern_id, back_context, index_match)|
                    if !back_context.is_empty() {
                        Err(PatternMismatch::NoParents) // todo
                    } else {
                        //println!("found parent matching");
                        match index_match {
                            // found index for complete pattern
                            PatternMatch::Matching => Ok(Vec::new()),
                            // found matching parent larger than the pattern, not the one we were looking for
                            PatternMatch::SupRemainder(_) => Err(PatternMismatch::NoParents),
                            // found parent matching with prefix of pattern, continue
                            PatternMatch::SubRemainder(rem) => Ok(rem),
                        }.and_then(|new_context|
                            // TODO: faster way to handle empty new_post
                            //println!("matching on parent with remainder");
                            self.match_sub_and_context_with_index(parent_index, &new_context, sup_index, sup_width)
                        )
                    }
                )
        }
    }
    /// match context against child context in parent.
    pub fn compare_context_with_child_pattern(
        &'a self,
        context: PatternView<'a>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<((PatternId, usize), Pattern, PatternMatch), PatternMismatch> {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub offset
        let candidates = D::candidate_parent_pattern_indices(parent, child_patterns);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        Searcher::<'a, T, D>::find_best_child_pattern(child_patterns, candidates.into_iter(), context)
            .or(Err(PatternMismatch::NoChildPatterns))
            .and_then(|(pattern_index, sub_index)|
                self.compare_child_pattern_with_remainder(child_patterns, context, pattern_index, sub_index)
                    .map(|(back_context, m)| ((pattern_index, sub_index), back_context, m))
            )
    }
    /// comparison on child pattern and context
    pub fn compare_child_pattern_with_remainder(
        &'a self,
        child_patterns: &'a ChildPatterns,
        context: PatternView<'a>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> Result<(Pattern, PatternMatch), PatternMismatch> {
        let pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, rem) = D::directed_pattern_split(pattern, sub_index);
        let tail = D::pattern_tail(&rem[..]);
        self.compare(context, tail).map(|pm| (back_context, pm))
        // returns result of matching sub with parent's children
    }
}
