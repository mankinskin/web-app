use crate::{
    hypergraph::{
        pattern::*,
        search::FoundRange,
        Hypergraph,
    },
    token::Tokenize,
};
use either::Either;
mod matcher;
pub use matcher::*;
mod match_direction;
pub use match_direction::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternMatch {
    SupRemainder(Pattern),
    SubRemainder(Pattern),
    Matching,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternMismatch {
    EmptyPatterns,
    Mismatch,
    NoParents,
    NoChildPatterns,
    NoMatchingParent,
    ParentMatchingPartially,
    SingleIndex,
    UnknownTokens,
}
type MatchResult = Result<PatternMatch, PatternMismatch>;

impl PatternMatch {
    pub fn flip_remainder(self) -> Self {
        match self {
            Self::SubRemainder(p) => Self::SupRemainder(p),
            Self::SupRemainder(p) => Self::SubRemainder(p),
            _ => self,
        }
    }
    pub fn is_matching(&self) -> bool {
        self == &Self::Matching
    }
    pub fn prepend_prefix(self, pattern: Pattern) -> FoundRange {
        if pattern.is_empty() {
            match self {
                Self::Matching => FoundRange::Complete,
                Self::SupRemainder(post) => FoundRange::Prefix(post),
                Self::SubRemainder(post) => FoundRange::Prefix(post),
            }
        } else {
            match self {
                Self::Matching => FoundRange::Prefix(pattern),
                Self::SupRemainder(post) => FoundRange::Infix(pattern, post),
                Self::SubRemainder(post) => FoundRange::Infix(pattern, post),
            }
        }
    }
}
impl From<Either<Pattern, Pattern>> for PatternMatch {
    fn from(e: Either<Pattern, Pattern>) -> Self {
        match e {
            Either::Left(p) => Self::SubRemainder(p),
            Either::Right(p) => Self::SupRemainder(p),
        }
    }
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn right_matcher(&'a self) -> Matcher<'a, T, MatchRight> {
        Matcher::new(self)
    }
    pub fn left_matcher(&'a self) -> Matcher<'a, T, MatchLeft> {
        Matcher::new(self)
    }
    pub fn compare_pattern_postfix(
        &self,
        a: PatternView<'a>,
        b: PatternView<'a>,
    ) -> MatchResult {
        self.left_matcher().compare(a, b)
    }
    pub fn compare_pattern_prefix(
        &self,
        a: PatternView<'a>,
        b: PatternView<'a>,
    ) -> MatchResult {
        self.right_matcher().compare(a, b)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::{
        tests::context,
        Child,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn compare_pattern_prefix() {
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
            _cd,
            _bcd,
            abc,
            abcd,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &*context();
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
        assert_eq!(
            graph.compare_pattern_prefix(a_bc_pattern, ab_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(ab_c_pattern, a_bc_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(a_b_c_pattern, a_bc_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(a_b_c_pattern, a_b_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(ab_c_pattern, a_b_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(a_bc_d_pattern, ab_c_d_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_prefix(abc_d_pattern, a_bc_d_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(bc_pattern, abcd_pattern),
            Err(PatternMismatch::NoParents)
        );
        assert_eq!(
            graph.compare_pattern_prefix(b_c_pattern, a_bc_pattern),
            Err(PatternMismatch::Mismatch)
        );
        assert_eq!(
            graph.compare_pattern_prefix(b_c_pattern, a_d_c_pattern),
            Err(PatternMismatch::Mismatch)
        );

        assert_eq!(
            graph.compare_pattern_prefix(a_bc_d_pattern, abc_d_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(a_bc_d_pattern, abcd_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_prefix(abcd_pattern, a_bc_d_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_prefix(a_b_c_pattern, abcd_pattern),
            Ok(PatternMatch::SupRemainder(vec![Child::new(*d, 1)]))
        );

        assert_eq!(
            graph.compare_pattern_prefix(ab_c_d_pattern, a_bc_pattern),
            Ok(PatternMatch::SubRemainder(vec![Child::new(*d, 1)]))
        );
        assert_eq!(
            graph.compare_pattern_prefix(a_bc_pattern, ab_c_d_pattern),
            Ok(PatternMatch::SupRemainder(vec![Child::new(*d, 1)]))
        );
    }
    #[test]
    fn compare_pattern_postfix() {
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
            _cd,
            _bcd,
            abc,
            abcd,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = &[*a, *bc];
        let ab_c_pattern = &[*ab, *c];
        let abc_d_pattern = &[*abc, *d];
        let a_bc_d_pattern = &[*a, *bc, *d];
        let ab_c_d_pattern = &[*ab, *c, *d];
        let abcd_pattern = &[*abcd];
        let b_c_pattern = &[*b, *c];
        let b_pattern = &[*b];
        let bc_pattern = &[*bc];
        let a_d_c_pattern = &[*a, *d, *c];
        let a_b_c_pattern = &[*a, *b, *c];
        let a_b_pattern = &[*a, *b];
        let bc_d_pattern = &[*bc, *d];
        assert_eq!(
            graph.compare_pattern_postfix(a_bc_pattern, ab_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(ab_c_pattern, a_bc_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_postfix(a_b_pattern, b_pattern),
            Ok(PatternMatch::SubRemainder(vec![Child::new(a, 1)]))
        );
        assert_eq!(
            graph.compare_pattern_postfix(a_b_c_pattern, a_bc_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_postfix(a_b_c_pattern, a_b_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(ab_c_pattern, a_b_c_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(a_bc_d_pattern, ab_c_d_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(abc_d_pattern, a_bc_d_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(bc_pattern, abcd_pattern),
            Err(PatternMismatch::NoParents)
        );
        assert_eq!(
            graph.compare_pattern_postfix(b_c_pattern, a_bc_pattern),
            Ok(PatternMatch::SupRemainder(vec![Child::new(*a, 1)]))
        );
        assert_eq!(
            graph.compare_pattern_postfix(b_c_pattern, a_d_c_pattern),
            Err(PatternMismatch::Mismatch)
        );
        assert_eq!(
            graph.compare_pattern_postfix(a_bc_d_pattern, abc_d_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_postfix(a_bc_d_pattern, abcd_pattern),
            Ok(PatternMatch::Matching)
        );
        assert_eq!(
            graph.compare_pattern_postfix(abcd_pattern, a_bc_d_pattern),
            Ok(PatternMatch::Matching)
        );

        assert_eq!(
            graph.compare_pattern_postfix(a_b_c_pattern, abcd_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(ab_c_d_pattern, a_bc_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(a_bc_pattern, ab_c_d_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(bc_d_pattern, ab_c_d_pattern),
            Ok(PatternMatch::SupRemainder(vec![Child::new(*a, 1)]))
        );
        assert_eq!(
            graph.compare_pattern_postfix(bc_d_pattern, abc_d_pattern),
            Ok(PatternMatch::SupRemainder(vec![Child::new(*a, 1)]))
        );
        assert_eq!(
            graph.compare_pattern_postfix(abcd_pattern, bc_d_pattern),
            Ok(PatternMatch::SubRemainder(vec![Child::new(*a, 1)]))
        );
    }
}
