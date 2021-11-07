use crate::{
    hypergraph::{
        pattern::*,
        r#match::*,
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
use std::borrow::Borrow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternMatch(pub Option<Pattern>, pub Option<Pattern>);

impl PatternMatch {
    pub fn left(&self) -> &Option<Pattern> {
        &self.0
    }
    pub fn right(&self) -> &Option<Pattern> {
        &self.1
    }
    pub fn flip_remainder(self) -> Self {
        Self(self.1, self.0)
    }
    pub fn is_matching(&self) -> bool {
        self.left().is_none() && self.right().is_none()
    }
}
impl From<Either<Pattern, Pattern>> for PatternMatch {
    fn from(e: Either<Pattern, Pattern>) -> Self {
        match e {
            Either::Left(p) => Self(Some(p), None),
            Either::Right(p) => Self(None, Some(p)),
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParentMatch {
    pub parent_range: FoundRange,
    pub remainder: Option<Pattern>,
}
impl ParentMatch {
    pub fn embed_in_super(self, other: Self) -> Self {
        Self {
            parent_range: self.parent_range.embed_in_super(other.parent_range),
            remainder: other.remainder,
        }
    }
}
pub type PatternMatchResult = Result<PatternMatch, PatternMismatch>;
pub type ParentMatchResult = Result<ParentMatch, PatternMismatch>;

#[derive(Clone, Debug)]
pub struct Matcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize, D: MatchDirection> std::ops::Deref for Matcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Matcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn searcher(&self) -> Searcher<'g, T, D> {
        Searcher::new(self.graph)
    }
    // Outline:
    // matching two patterns of indices and
    // returning the remainder. Starting from left or right.
    // - skip equal indices
    // - once unequal, pick larger and smaller index
    // - search for larger in parents of smaller
    // - otherwise: try to find parent with best matching children
    pub fn compare<A: IntoPattern<Item=impl Into<Child> + Tokenize>, B: IntoPattern<Item=impl Into<Child> + Tokenize>>(
        &self,
        a: A,
        b: B,
        ) -> PatternMatchResult {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let a: Pattern = a.into_pattern();
        let b: Pattern = b.into_pattern();
        if let Some((pos, eob)) = D::skip_equal_indices(a.clone().iter(), b.clone().iter()) {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both(ai, bi) => {
                    self.match_unequal_indices_in_context(a, *ai, b, *bi, pos)
                }
                EitherOrBoth::Left(_) => {
                    Ok(PatternMatch(Some(D::split_end_normalized(&a, pos)), None))
                }
                EitherOrBoth::Right(_) => {
                    Ok(PatternMatch(None, Some(D::split_end_normalized(&b, pos))))
                }
            }
        } else {
            Ok(PatternMatch(None, None))
        }
    }
    fn match_unequal_indices_in_context<C: Into<Child> + Tokenize, A: IntoPattern<Item=impl Into<Child> + Tokenize, Token=C>, B: IntoPattern<Item=impl Into<Child> + Tokenize, Token=C>>(
        &self,
        a_pattern: A,
        a: Child,
        b_pattern: B,
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
                sub_context = D::front_context_normalized(a_pattern.as_pattern_view(), pos);
                sup_context = D::front_context_normalized(b_pattern.as_pattern_view(), pos);
                sub = a;
                sup = b;
                false
            }
            Ordering::Greater => {
                //println!("left super");
                sub_context = D::front_context_normalized(b_pattern.as_pattern_view(), pos);
                sup_context = D::front_context_normalized(a_pattern.as_pattern_view(), pos);
                sub = b;
                sup = a;
                true
            }
        };
        // left remainder: sub remainder
        // right remainder: sup remainder
        // matching: sub & sup finished
        let parent_match = self.match_sub_and_context_with_index(sub, sub_context, sup)?;
        let rem = parent_match.remainder;
        match parent_match.parent_range {
            FoundRange::Complete => self.compare(rem.unwrap_or_default(), sup_context),
            found_range => {
                let post = D::get_remainder(found_range);
                Ok(PatternMatch(
                    rem,
                    post.map(|post| D::merge_remainder_with_context(post, sup_context.into_pattern())),
                ))
            }
        }
        .map(|result| {
            if rotate {
                result.flip_remainder()
            } else {
                result
            }
        })
    }
    /// match sub index and context with sup index with max width
    fn match_sub_and_context_with_index(
        &self,
        sub: impl Indexed,
        sub_context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
        sup: Child,
    ) -> ParentMatchResult {
        //println!("match_sub_pattern_to_super");
        // search parent of sub
        let sub_index = *sub.index();
        if sub_index == *sup.index() {
            return Ok(ParentMatch {
                parent_range: FoundRange::Complete,
                remainder: (!sub_context.is_empty()).then(|| sub_context.into_pattern()),
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
            self.match_parent_children(sub_context, sup, parent)
                .map(|(parent_match, _, _)| parent_match)
        } else {
            // sup is no direct parent, search upwards
            //println!("matching available parents");
            // search sup in parents
            self.searcher()
                .find_largest_matching_parent_below_width(vertex, sub_context, Some(sup.width))
                .or(Err(PatternMismatch::NoMatchingParent))
                .and_then(
                    |SearchFound {
                         index: parent_index,
                         parent_match,
                         ..
                     }| {
                        match (
                            D::found_at_start(parent_match.parent_range),
                            parent_match.remainder,
                        ) {
                            (true, rem) => Ok(rem.unwrap_or_default()),
                            // parent not matching at beginning
                            (false, _) => {
                                //println!("Found index {} not matching at beginning", parent_index.index);
                                Err(PatternMismatch::NoMatchingParent)
                            },
                        }
                        // search next parent
                        .and_then(|new_context| {
                            self.match_sub_and_context_with_index(parent_index, new_context, sup)
                        })
                    },
                )
        }
    }
    /// match context against child context in parent.
    pub fn match_parent_children(
        &'g self,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<(ParentMatch, PatternId, usize), PatternMismatch> {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub index
        let candidates = D::filter_parent_pattern_indices(parent, child_patterns);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        Self::get_best_child_pattern(
            child_patterns,
            candidates.iter(),
            context.as_pattern_view(),
        )
        // todo: check if this is always correct
        // todo: skip tail comparison for non-candidates (they always point to sub-index at end in parent)
        //.or_else(|| parent.pattern_indices.iter().next().cloned())
        //
        .ok_or(PatternMismatch::NoMatchingParent)
        .and_then(|(pattern_index, sub_index)| {
            self.compare_child_pattern_at_offset(child_patterns, context, pattern_index, sub_index)
                .map(|parent_match| (parent_match, pattern_index, sub_index))
        })
    }
    /// try to find child pattern with context matching sub_context
    pub(crate) fn get_best_child_pattern(
        child_patterns: &'_ ChildPatterns,
        candidates: impl Iterator<Item = impl Borrow<(usize, PatternId)>>,
        sub_context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
    ) -> Option<(PatternId, usize)> {
        candidates
            .map(|c| *c.borrow())
            .find_or_first(|(pattern_index, sub_index)| {
                Self::compare_next_index_in_child_pattern(
                    child_patterns,
                    sub_context.as_pattern_view(),
                    pattern_index,
                    *sub_index,
                )
            })
    }
    pub(crate) fn compare_next_index_in_child_pattern(
        child_patterns: &'_ ChildPatterns,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
        pattern_index: &PatternId,
        sub_index: usize,
    ) -> bool {
        D::pattern_head(context.as_pattern_view())
            .and_then(|next_sub| {
                let next_sub: Child = (*next_sub).into();
                D::index_next(sub_index).and_then(|i| {
                    child_patterns
                        .get(pattern_index)
                        .and_then(|pattern| pattern.get(i).map(|next_sup| next_sub == *next_sup))
                })
            })
            .unwrap_or(false)
    }
    /// comparison on child pattern and context
    pub fn compare_child_pattern_at_offset(
        &'g self,
        child_patterns: &'g ChildPatterns,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
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
        self.compare(context, child_tail).map(|pm| ParentMatch {
            parent_range: D::to_found_range(pm.1, back_context),
            remainder: pm.0,
        })
        // returns result of matching sub with parent's children
    }
    pub(crate) fn match_indirect_parent(
        &'g self,
        index: VertexIndex,
        parent: &Parent,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
    ) -> Option<SearchFound> {
        self
            .match_parent_children(context.as_pattern_view(), index, parent)
            .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                index: Child::new(index, parent.width),
                pattern_id,
                sub_index,
                parent_match,
            })
            .ok()
    }
}
