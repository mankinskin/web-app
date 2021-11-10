
use crate::{
        pattern::*,
        r#match::*,
    token::Tokenize,
};
use itertools::EitherOrBoth;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct AsyncPatternMatch<T: Tokenize> {
    pub left: Option<Box<dyn ReturnedPatternStream<T>>>,
    pub right: Option<Box<dyn ReturnedPatternStream<T>>>,
    _ty_t: std::marker::PhantomData<T>,
}
impl<T: Tokenize> AsyncPatternMatch<T> {
    pub fn flip_remainder(self) -> Self {
        Self {
            left: self.right,
            right: self.left,
            _ty_t: Default::default(),
        }
    }
    pub fn is_matching(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}
impl<T: Tokenize> Default for AsyncPatternMatch<T> {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
            _ty_t: Default::default(),
        }
    }
}
#[derive(Debug)]
pub struct AsyncParentMatch<T: Tokenize> {
    pub parent_range: FoundRange,
    pub remainder: Option<Box<dyn ReturnedPatternStream<T>>>,
}
pub type AsyncParentMatchResult<T> = Result<AsyncParentMatch<T>, PatternMismatch>;
pub type AsyncPatternMatchResult<T> = Result<AsyncPatternMatch<T>, PatternMismatch>;

#[derive(Clone, Debug)]
pub struct AsyncMatcher<T: Tokenize + Send, D: AsyncMatchDirection<T>> {
    graph: HypergraphHandle<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'t, T: Tokenize + Send + 'static, D: AsyncMatchDirection<T>> AsyncMatcher<T, D> {
    pub fn new(graph: HypergraphHandle<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn searcher(&self) -> AsyncSearcher<T, D> {
        AsyncSearcher::new(self.graph)
    }
    // Outline:
    // matching two patterns of indices and
    // returning the remainder. Starting from left or right.
    // - skip equal indices
    // - once unequal, pick larger and smaller index
    // - search for larger in parents of smaller
    // - otherwise: try to find parent with best matching children
    pub async fn compare(&self,
        a: impl PatternStream<Child, Token<T>> + 'static,
        b: impl PatternStream<Child, Token<T>> + 'static,
        ) -> Box<AsyncPatternMatchResult<T>> {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        if let Some((back, eob)) = <D as AsyncMatchDirection<T>>::skip_equal_indices(a, b).await {
            match eob {
                // different elements on both sides
                EitherOrBoth::Both((ac, ap), (bc, bp)) => {
                    self.match_unequal_indices_in_context(ap, ac, bp, bc).await
                },
                // one pattern ended, return remainder
                EitherOrBoth::Left((c, p)) => {
                    Box::new(Ok(AsyncPatternMatch {
                        left: Some(p),
                        ..Default::default()
                    }))
                },
                EitherOrBoth::Right((c, p)) => {
                    Box::new(Ok(AsyncPatternMatch {
                        right: Some(p),
                        ..Default::default()
                    }))
                },
            }
        } else {
            Box::new(Ok(AsyncPatternMatch::default()))
        }
    }
    async fn match_unequal_indices_in_context(
        &'t self,
        a_pattern: Box<dyn ReturnedPatternStream<T>>,
        a: Child,
        b_pattern: Box<dyn ReturnedPatternStream<T>>,
        b: Child,
    ) -> Box<AsyncPatternMatchResult<T>> {
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
            Ordering::Equal => return Box::new(Err(PatternMismatch::Mismatch)),
            Ordering::Less => {
                //println!("right super");
                sub_context = a_pattern;
                sup_context = b_pattern;
                sub = a;
                sup = b;
                false
            }
            Ordering::Greater => {
                //println!("left super");
                sub_context = b_pattern;
                sup_context = a_pattern;
                sub = b;
                sup = a;
                true
            }
        };
        // left remainder: sub remainder
        // right remainder: sup remainder
        // matching: sub & sup finished
        let parent_match = match *self.match_sub_and_context_with_index(sub, sub_context, sup).await {
            Ok(ok) => ok,
            Err(err) => return Box::new(Err(err)),
        };
        let rem = parent_match.remainder;
        let mut boxed_res = match parent_match.parent_range {
            FoundRange::Complete => self.compare(rem.unwrap_or(Box::new(tokio_stream::iter([]))), sup_context).await,
            found_range => {
                let post = D::get_remainder(found_range);
                Box::new(Ok(AsyncPatternMatch {
                    left: rem,
                    right: if let Some(post) = post {
                            Some(<D as AsyncMatchDirection<T>>::merge_remainder_with_context(post, sup_context).await)
                        } else {
                            None
                        },
                    ..Default::default()
                }))
            }
        };
        *boxed_res = boxed_res.map(|result| {
            if rotate {
                result.flip_remainder()
            } else {
                result
            }
        });
        boxed_res
    }
    /// match sub index and context with sup index with max width
    async fn match_sub_and_context_with_index(
        &'t self,
        sub: impl Indexed,
        sub_context: Box<dyn ReturnedPatternStream<T>>,
        sup: Child,
    ) -> Box<AsyncParentMatchResult<T>> {
        //println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub.index() == sup.index() {
            return Box::new(Ok(AsyncParentMatch {
                parent_range: FoundRange::Complete,
                remainder: Some(sub_context),
            }));
        }
        let vertex = self.graph.read().await.expect_vertex_data(sub).clone();
        if vertex.get_parents().is_empty() {
            return Box::new(Err(PatternMismatch::NoParents));
        }
        // get parent where vertex is at relevant position (prefix or postfix)
        if let Some(parent) = D::get_match_parent(&vertex, sup) {
            // found vertex in sup at relevant position
            //println!("sup found in parents");
            // compare context after vertex in parent
            Box::new(self.compare_with_parent_children(sub_context, sup, parent)
                .await
                .map(|(parent_match, _, _)| parent_match))
        } else {
            // sup is no direct parent, search upwards
            //println!("matching available parents");
            // search sup in parents
            if let Ok(SearchFound {
                    index: parent_index,
                    parent_match,
                    ..
                }) = self.searcher()
                .find_largest_matching_parent_below_width(vertex, sub_context, Some(sup.width))
                .await {
                    match (
                        D::found_at_start(parent_match.parent_range),
                        parent_match.remainder,
                    ) {
                        (true, rem) => {
                            let new_context = rem.unwrap_or_default();
                            self.match_sub_and_context_with_index(
                                parent_index,
                                Box::new(tokio_stream::iter(new_context.into_iter().map(|c| Ok(c)))),
                                sup,
                            ).await
                        },
                        // parent not matching at beginning
                        (false, _) => Box::new(Err(PatternMismatch::NoParents)),
                    }
                } else {
                    Box::new(Err(PatternMismatch::NoMatchingParent))
                }
        }
    }
    /// match context against child context in parent.
    pub async fn compare_with_parent_children(
        &'t self,
        context: impl PatternStream<Child, Token<T>>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Box<Result<(AsyncParentMatch<T>, PatternId, usize), PatternMismatch>> {
        //println!("compare_parent_context");
        let vertex = self.graph.read().await.expect_vertex_data(parent_index).clone();
        let child_patterns = vertex.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub offset
        let candidates = D::candidate_parent_pattern_indices(parent, child_patterns);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        Box::new(if let Ok((pattern_index, sub_index)) = AsyncSearcher::<T, D>::find_best_child_pattern(
            child_patterns,
            candidates.into_iter(),
            context,
        )
        .await {
            match *self.compare_child_pattern_at_offset(
                    child_patterns,
                    context,
                    pattern_index,
                    sub_index,
                )
                .await {
                Ok(parent_match) => Ok((parent_match, pattern_index, sub_index)),
                Err(err) => Err(err),
            }
        } else {
            Err(PatternMismatch::NoChildPatterns)
        })
    }
    /// comparison on child pattern and context
    pub async fn compare_child_pattern_at_offset(
        &'t self,
        child_patterns: &'t ChildPatterns,
        context: impl PatternStream<Child, Token<T>>,
        pattern_index: PatternId,
        sub_index: usize,
    ) -> Box<AsyncParentMatchResult<T>> {
        let child_pattern = child_patterns
            .get(&pattern_index)
            .expect("non existent pattern found as best match!");
        let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
        let child_tail = D::pattern_tail(&rem[..]);
        // match search context with child tail
        // back context is from child pattern
        let mut res = self.compare(context, tokio_stream::iter(child_tail.into_iter().map(|c| Ok(*c)))).await;
        Box::new(match *res {
            Ok(pm) => Ok(AsyncParentMatch {
                    parent_range: <D as AsyncMatchDirection<T>>::to_found_range(pm.right, tokio_stream::iter(back_context)).await,
                    remainder: pm.left,
                }),
            Err(err) => Err(err),
        })
        // returns result of matching sub with parent's children
    }
}
