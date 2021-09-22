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
    pub fn compare(&self, a: PatternView<'_>, b: PatternView<'_>) -> PatternMatchResult {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        if let Some((pos, eob)) = D::skip_equal_indices(a.iter(), b.iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(ai, bi) => {
                    self.match_unequal_indices_in_context(a, *ai, b, *bi, pos)
                },
                EitherOrBoth::Left(_) => {
                    Ok(PatternMatch(Some(D::split_end_normalized(a, pos)), None))
                },
                EitherOrBoth::Right(_) => {
                    Ok(PatternMatch(None, Some(D::split_end_normalized(b, pos))))
                },
            }
        } else {
            Ok(PatternMatch(None, None))
        }
    }
    fn match_unequal_indices_in_context(
        &self,
        a_pattern: PatternView<'a>,
        a: Child,
        b_pattern: PatternView<'a>,
        b: Child,
        pos: TokenPosition,
    ) -> PatternMatchResult {
        // Note: depending on sizes of a, b it may be differently efficient
        // to search for children or parents, large patterns have less parents,
        // small patterns have less children
        // search larger in parents of smaller
        let sub_context;
        let sup_context;
        let sub;
        let sup;
        // remember if sub and sup were switched
        let rotate = match a.width.cmp(&b.width) {
            // relatives can not have same sizes
            Ordering::Equal => return Err(PatternMismatch::Mismatch),
            Ordering::Less => {
                //println!("right super");
                sub_context = D::front_context_normalized(a_pattern, pos);
                sup_context = D::front_context_normalized(b_pattern, pos);
                sub = a;
                sup = b;
                false
            }
            Ordering::Greater => {
                //println!("left super");
                sub_context = D::front_context_normalized(b_pattern, pos);
                sup_context = D::front_context_normalized(a_pattern, pos);
                sub = b;
                sup = a;
                true
            }
        };
        // left remainder: sub remainder
        // right remainder: sup remainder
        // matching: sub & sup finished
        let parent_match = self.match_sub_and_context_with_index(sub, &sub_context[..], sup)?;
        let rem = parent_match.remainder;
        match parent_match.parent_range {
            FoundRange::Complete => self.compare(&rem.unwrap_or_default(), &sup_context),
            found_range => {
                let post = D::get_remainder(found_range);
                Ok(PatternMatch(rem, post.map(|post| D::merge_remainder_with_context(&post, &sup_context))))
            },
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
        sub_context: PatternView<'_>,
        sup: Child,
    ) -> ParentMatchResult {
        //println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub.index() == sup.index() {
            return Ok(ParentMatch {
                parent_range: FoundRange::Complete,
                remainder: (!sub_context.is_empty()).then(|| sub_context.into()),
            });
        }
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().is_empty() {
            return Err(PatternMismatch::NoParents);
        }
        // get parent where vertex is at relevant position (prefix or postfix)
        if let Some(parent) = D::get_match_parent(vertex, sup) {
            // found vertex in sup at relevant position
            //println!("sup found in parents");
            // compare context after vertex in parent
            self.compare_with_parent_children(sub_context, sup, parent)
                .map(|(parent_match, _, _)| parent_match)
        } else {
            // sup is no direct parent, search upwards
            //println!("matching available parents");
            // search sup in parents
            self.searcher()
                .find_parent_matching_context_below_width(sub_context, vertex, Some(sup.width))
                .or(Err(PatternMismatch::NoMatchingParent))
                .and_then(|SearchFound {
                    index: parent_index,
                    parent_match,
                    ..
                }|
                    match (D::found_at_start(parent_match.parent_range), parent_match.remainder) {
                        (true, rem) => Ok(rem.unwrap_or_default()),
                        // parent not matching at beginning
                        (false, _) => Err(PatternMismatch::NoParents),
                    }
                    // search next parent
                    .and_then(|new_context|
                        self.match_sub_and_context_with_index(parent_index, &new_context, sup)
                    )
                )
        }
    }
    /// match context against child context in parent.
    pub fn compare_with_parent_children(
        &'a self,
        context: PatternView<'a>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<(ParentMatch, PatternId, usize), PatternMismatch> {
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
                self.compare_child_pattern_at_offset(child_patterns, context, pattern_index, sub_index)
                    .map(|parent_match| (parent_match, pattern_index, sub_index))
            )
    }
    /// comparison on child pattern and context
    pub fn compare_child_pattern_at_offset(
        &'a self,
        child_patterns: &'a ChildPatterns,
        context: PatternView<'a>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> ParentMatchResult {
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
        let child_tail = D::pattern_tail(&rem[..]);
        // match search context with child tail
        // back context is from child pattern
        self.compare(context, child_tail).map(|pm|
            ParentMatch {
                parent_range: D::to_found_range(pm.1, back_context),
                remainder: pm.0,
            }
        )
        // returns result of matching sub with parent's children
    }
}
