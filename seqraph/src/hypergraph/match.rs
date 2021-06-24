use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternIndex,
        Pattern,
        PatternView,
        VertexData,
        Parent,
        TokenPosition,
        Child,
        ChildPatterns,
    },
    token::{
        Tokenize,
    },
};
use either::Either;
use itertools::{
    Itertools,
    EitherOrBoth,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternMatch {
    Remainder(Either<Pattern, Pattern>),
    Matching,
}
impl PatternMatch {
    pub fn flip_remainder(self) -> Self {
        match self {
            Self::Remainder(e) => Self::Remainder(e.flip()),
            _ => self,
        }
    }
}
//impl From<SearchFound> for PatternMatch {
//    fn from(SearchFound(range, index, offset): SearchFound) -> Self {
//        match (offset, range) {
//            (0, FoundRange::Complete) => Self::Matching,
//            (0, FoundRange::Prefix(remainder)) => Self::Remainder(Either::Left(remainder)),
//            _ => Self::Mismatch,
//        }
//    }
//}
//impl From<SearchFound> for IndexMatch {
//    fn from(SearchFound(range, index, offset): SearchFound) -> Self {
//        match (offset, range) {
//            (0, FoundRange::Complete) => Self::Matching,
//            (0, FoundRange::Prefix(remainder)) => Self::SubRemainder(remainder),
//            _ => Self::Mismatch,
//        }
//    }
//}
impl From<IndexMatch> for PatternMatch {
    fn from(r: IndexMatch) -> Self {
        match r {
            IndexMatch::SubRemainder(p) => Self::Remainder(Either::Left(p)),
            IndexMatch::SupRemainder(p) => Self::Remainder(Either::Right(p)),
            IndexMatch::Matching => Self::Matching,
        }
    }
}
impl From<PatternMatch> for IndexMatch {
    fn from(r: PatternMatch) -> Self {
        match r {
            PatternMatch::Remainder(e) => match e {
                Either::Left(p) => Self::SubRemainder(p),
                Either::Right(p) => Self::SupRemainder(p),
            },
            PatternMatch::Matching => Self::Matching,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndexMatch {
    SupRemainder(Pattern),
    SubRemainder(Pattern),
    Matching,
}

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    fn pick_best_matching_child_pattern(
        child_patterns: &'a ChildPatterns,
        candidates: impl Iterator<Item=&'a (usize, PatternIndex)>,
        post_sub_pat: PatternView<'a>,
        ) -> Option<PatternView<'a>> {
        candidates.find_or_first(|(pattern_index, sub_index)|
            post_sub_pat.get(0).and_then(|post_sub|
                    child_patterns.get(pattern_index)
                        .and_then(|pattern|
                            pattern.get(sub_index+1).map(|b| post_sub == b)
                        )
            ).unwrap_or(false)
        ).and_then(|(pattern_index, sub_index)|
            child_patterns.get(pattern_index).and_then(|pattern| pattern.get(*sub_index..))
        )
    }
    /// match sub_pat against children in parent with an optional offset.
    pub fn compare_parent_at_offset(
        &self,
        post_pattern: PatternView<'a>,
        parent_index: VertexIndex,
        parent: &Parent,
        offset: Option<PatternIndex>,
        ) -> Option<IndexMatch> {
        // find pattern where sub is at offset
        println!("compare_parent_at_offset");
        let vert = self.expect_vertex_data(parent_index);
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.sub_index_string(parent.index));
        // optionally filter by sub offset
        let candidates = parent.get_pattern_index_candidates(offset);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // find child pattern with matching successor or pick first candidate
        let best_match = Self::pick_best_matching_child_pattern(
            &child_patterns,
            candidates,
            post_pattern,
        );
        best_match.and_then(|child_pattern|
            //println!("comparing post sub pattern with remaining children of parent");
            self.compare_pattern_prefix(
                post_pattern,
                child_pattern.get(1..).unwrap_or(&[])
                ).map(Into::into)
            // returns result of matching sub with parent's children
        )
    }
    fn get_direct_vertex_parent_with_offset(
        vertex: &'a VertexData,
        parent_index: VertexIndex,
        offset: Option<PatternIndex>,
        ) -> Option<&'a Parent> {
        vertex.get_parent(&parent_index)
            .filter(|parent|
                offset.map(|offset|
                    parent.exists_at_pos(offset)
                ).unwrap_or(true)
            )
    }
    fn get_direct_vertex_parent_at_prefix(
        vertex: &'a VertexData,
        index: VertexIndex,
        ) -> Option<&'a Parent> {
        Self::get_direct_vertex_parent_with_offset(&vertex, index, Some(0))
    }
    /// find an index at the prefix of a pattern
    fn match_sub_and_post_with_index(&self,
            sub: VertexIndex,
            post_pattern: PatternView<'a>,
            sup_index: VertexIndex,
            width: TokenPosition,
        ) -> Option<IndexMatch> {
        println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub == sup_index {
            return if post_pattern.is_empty() {
                Some(IndexMatch::Matching)
            } else {
                Some(IndexMatch::SubRemainder(post_pattern.into()))
            }
        }
        let vertex = self.expect_vertex_data(sub);
        if vertex.get_parents().len() < 1 {
            return None;
        }
        let sup_parent = Self::get_direct_vertex_parent_at_prefix(&vertex, sup_index);
        if let Some(parent) = sup_parent {
            // parents contain sup
            println!("sup found in parents");
            self.compare_parent_at_offset(post_pattern, sup_index, parent, Some(0))
        } else {
            // sup is no direct parent, search upwards
            println!("matching available parents");
            // search sup in parents
            let (parent_index, parent, index_match) = self.find_parent_matching_pattern_at_offset_below_width(
                post_pattern,
                &vertex,
                Some(0),
                Some(width),
            )?;
            println!("found parent matching");
            let new_post = match index_match {
                // found index for complete pattern, tr
                IndexMatch::Matching => Some(Vec::new()),
                // found matching parent larger than the pattern, not the one we were looking for
                IndexMatch::SupRemainder(_) => None,
                // found parent matching with prefix of pattern, continue
                IndexMatch::SubRemainder(rem) => Some(rem),
            }?;
            // TODO: faster way to handle empty new_post
            println!("matching on parent with remainder");
            self.match_sub_and_post_with_index(parent_index, &new_post, sup_index, width)
        }
    }
    pub(crate) fn match_pattern_with_index(
        &self,
        sub_pattern: PatternView<'a>,
        index: VertexIndex,
        width: TokenPosition,
        ) -> Option<IndexMatch> {
        println!("match_sub_pattern_to_super");
        let sub = sub_pattern.get(0)?;
        let post_pattern = sub_pattern.get(1..);
        if let None = post_pattern {
            return if sub.get_index() == index {
                Some(IndexMatch::Matching)
            } else {
                None
            };
        }
        let post_pattern = post_pattern?;
        self.match_sub_and_post_with_index(sub.get_index(), post_pattern, index, width)
    }
    fn compare_pattern_prefix(
            &self,
            pattern_a: PatternView<'a>,
            pattern_b: PatternView<'a>,
        ) -> Option<PatternMatch> {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let pattern_a_iter = pattern_a.iter();
        let pattern_b_iter = pattern_b.iter();
        let mut zipped = pattern_a_iter
            .zip_longest(pattern_b_iter)
            .enumerate()
            .skip_while(|(_, eob)|
                match eob {
                    EitherOrBoth::Both(a, b) => a == b,
                    _ => false,
                }
            );
        let (pos, eob) = if let Some(next) = zipped.next() {
            next
        } else {
            return Some(PatternMatch::Matching);
        };
        Some(match eob {
            // different elements on both sides
            EitherOrBoth::Both(a, b) => {
                //println!("matching \"{}\" and \"{}\"", self.sub_index_string(index_a), self.sub_index_string(index_b));
                // Note: depending on sizes of a, b it may be differently efficient
                // to search for children or parents, large patterns have less parents,
                // small patterns have less children
                // search larger in parents of smaller
                let post_sub_pattern;
                let post_sup;
                let sub;
                let sup;
                let sup_width;
                let rotate = if a.get_width() == b.get_width() {
                    // relatives can not have same sizes
                    return None;
                } else if a.get_width() < b.get_width() {
                    println!("right super");
                    post_sub_pattern = &pattern_a[pos+1..];
                    post_sup = &pattern_b[pos+1..];
                    sub = a.get_index();
                    sup = b.get_index();
                    sup_width = b.get_width();
                    false
                } else {
                    println!("left super");
                    post_sub_pattern = &pattern_b[pos+1..];
                    post_sup = &pattern_a[pos+1..];
                    sub = b.get_index();
                    sup = a.get_index();
                    sup_width = a.get_width();
                    true
                };
                let result = self.match_sub_and_post_with_index(
                    sub,
                    post_sub_pattern,
                    sup,
                    sup_width,
                );
                // left remainder: sub remainder
                // right remainder: sup remainder
                // matching: sub & sup finished
                println!("return {:#?}", result);
                let result = match result? {
                    IndexMatch::SubRemainder(rem) =>
                        self.compare_pattern_prefix(
                            &rem,
                            post_sup,
                        )?,
                    IndexMatch::SupRemainder(rem) => PatternMatch::Remainder(Either::Right([&rem, post_sup].concat())),
                    IndexMatch::Matching => {
                        let rem: Vec<_> = post_sup.iter().cloned().collect();
                        if rem.is_empty() {
                            PatternMatch::Matching
                        } else {
                            PatternMatch::Remainder(Either::Right(rem))
                        }
                    },
                };
                if rotate {
                    result.flip_remainder()
                } else {
                    result
                }
            },
            EitherOrBoth::Left(_) => PatternMatch::Remainder(Either::Left(pattern_a[pos..].iter().cloned().collect())),
            EitherOrBoth::Right(_) => PatternMatch::Remainder(Either::Right(pattern_b[pos..].iter().cloned().collect())),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::hypergraph::tests::CONTEXT;
    #[test]
    fn match_simple() {
        let (
            graph,
            a,
            b,
            c,
            d,
            _e,
            _f,
            _g,
            _h,
            _i,
            ab,
            bc,
            cd,
            _bcd,
            abc,
            abcd,
            _cdef,
            ) = &*CONTEXT;
        let a_bc_pattern = &[Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = &[Child::new(ab, 2), Child::new(c, 1)];
        let abc_d_pattern = &[Child::new(abc, 3), Child::new(d, 1)];
        let a_bc_d_pattern = &[Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let ab_c_d_pattern = &[Child::new(ab, 2), Child::new(c, 1), Child::new(d, 1)];
        let abcd_pattern = &[Child::new(abcd, 4)];
        let b_c_pattern = &[Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = &[Child::new(bc, 2)];
        let a_d_c_pattern = &[Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
        let a_b_c_pattern = &[Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_eq!(graph.compare_pattern_prefix(a_bc_pattern, ab_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(ab_c_pattern, a_bc_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(a_b_c_pattern, a_bc_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(a_b_c_pattern, a_b_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(ab_c_pattern, a_b_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(a_bc_d_pattern, ab_c_d_pattern), Some(PatternMatch::Matching));

        assert_eq!(graph.compare_pattern_prefix(abc_d_pattern, a_bc_d_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(bc_pattern, abcd_pattern), None);
        assert_eq!(graph.compare_pattern_prefix(b_c_pattern, a_bc_pattern), None);
        assert_eq!(graph.compare_pattern_prefix(b_c_pattern, a_d_c_pattern), None);

        assert_eq!(graph.compare_pattern_prefix(a_bc_d_pattern, abc_d_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(a_bc_d_pattern, abcd_pattern), Some(PatternMatch::Matching));
        assert_eq!(graph.compare_pattern_prefix(abcd_pattern, a_bc_d_pattern), Some(PatternMatch::Matching));

        assert_eq!(graph.compare_pattern_prefix(a_b_c_pattern, abcd_pattern), Some(PatternMatch::Remainder(Either::Right(vec![Child::new(*d, 1)]))));

        assert_eq!(graph.compare_pattern_prefix(ab_c_d_pattern, a_bc_pattern), Some(PatternMatch::Remainder(Either::Left(vec![Child::new(*d, 1)]))));
        assert_eq!(graph.compare_pattern_prefix(a_bc_pattern, ab_c_d_pattern), Some(PatternMatch::Remainder(Either::Right(vec![Child::new(*d, 1)]))));
    }
}