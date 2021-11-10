use crate::{
    pattern::*,
    search::*,
    *,
};
use either::Either;
mod matcher;
pub use matcher::*;
mod match_direction;
pub use match_direction::*;
//mod async_matcher;
//pub use async_matcher::*;
//mod async_match_direction;
//pub use async_match_direction::*;

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
    pub fn compare_pattern_postfix<C: Into<Child> + Tokenize>(
        &self,
        a: impl IntoPattern<Item = C>,
        b: impl IntoPattern<Item = C>,
    ) -> PatternMatchResult {
        self.left_matcher().compare(a, b)
    }
    pub fn compare_pattern_prefix(
        &self,
        a: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        b: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> PatternMatchResult {
        self.right_matcher().compare(a, b)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
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
            e,
            f,
            g,
            h,
            i,
            ab,
            bc,
            _cd,
            _bcd,
            abc,
            abcd,
            ef,
            _cdef,
            efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let abc_d_pattern = vec![Child::new(abc, 3), Child::new(d, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let ab_c_d_pattern = vec![Child::new(ab, 2), Child::new(c, 1), Child::new(d, 1)];
        let abcd_pattern = vec![Child::new(abcd, 4)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_d_c_pattern = vec![Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_eq!(
            graph.compare_pattern_prefix(vec![e, f, g, h, i], vec![efghi]),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(vec![ef], vec![e, f]),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(vec![e, f], vec![ef]),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(vec![efghi], vec![e, f, g, h, i]),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_bc_pattern, &ab_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&ab_c_pattern, &a_bc_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_b_c_pattern, &a_bc_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_b_c_pattern, &a_b_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&ab_c_pattern, &a_b_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_bc_d_pattern, &ab_c_d_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_prefix(&abc_d_pattern, &a_bc_d_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&bc_pattern, &abcd_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_prefix(&b_c_pattern, &a_bc_pattern),
            Err(PatternMismatch::Mismatch)
        );
        assert_eq!(
            graph.compare_pattern_prefix(&b_c_pattern, &a_d_c_pattern),
            Err(PatternMismatch::Mismatch)
        );

        assert_eq!(
            graph.compare_pattern_prefix(&a_bc_d_pattern, &abc_d_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_bc_d_pattern, &abcd_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&abcd_pattern, &a_bc_d_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_prefix(&a_b_c_pattern, &abcd_pattern),
            Ok(PatternMatch(None, Some(vec![Child::new(*d, 1)])))
        );

        assert_eq!(
            graph.compare_pattern_prefix(&ab_c_d_pattern, &a_bc_pattern),
            Ok(PatternMatch(Some(vec![Child::new(*d, 1)]), None))
        );
        assert_eq!(
            graph.compare_pattern_prefix(&a_bc_pattern, &ab_c_d_pattern),
            Ok(PatternMatch(None, Some(vec![Child::new(*d, 1)])))
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
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = vec![*a, *bc];
        let ab_c_pattern = vec![*ab, *c];
        let abc_d_pattern = vec![*abc, *d];
        let a_bc_d_pattern = vec![*a, *bc, *d];
        let ab_c_d_pattern = vec![*ab, *c, *d];
        let abcd_pattern = vec![*abcd];
        let b_c_pattern = vec![*b, *c];
        let b_pattern = vec![*b];
        let bc_pattern = vec![*bc];
        let a_d_c_pattern = vec![*a, *d, *c];
        let a_b_c_pattern = vec![*a, *b, *c];
        let a_b_pattern = vec![*a, *b];
        let bc_d_pattern = vec![*bc, *d];
        assert_eq!(
            graph.compare_pattern_postfix(&a_bc_pattern, &ab_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&ab_c_pattern, &a_bc_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_postfix(&a_b_pattern, &b_pattern),
            Ok(PatternMatch(Some(vec![Child::new(a, 1)]), None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&a_b_c_pattern, &a_bc_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_postfix(&a_b_c_pattern, &a_b_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&ab_c_pattern, &a_b_c_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&a_bc_d_pattern, &ab_c_d_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&abc_d_pattern, &a_bc_d_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&bc_pattern, &abcd_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(&b_c_pattern, &a_bc_pattern),
            Ok(PatternMatch(None, Some(vec![Child::new(*a, 1)])))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&b_c_pattern, &a_d_c_pattern),
            Err(PatternMismatch::Mismatch)
        );
        assert_eq!(
            graph.compare_pattern_postfix(&a_bc_d_pattern, &abc_d_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_postfix(&a_bc_d_pattern, &abcd_pattern),
            Ok(PatternMatch(None, None))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&abcd_pattern, &a_bc_d_pattern),
            Ok(PatternMatch(None, None))
        );

        assert_eq!(
            graph.compare_pattern_postfix(&a_b_c_pattern, &abcd_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(&ab_c_d_pattern, &a_bc_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(&a_bc_pattern, &ab_c_d_pattern),
            Err(PatternMismatch::NoMatchingParent)
        );
        assert_eq!(
            graph.compare_pattern_postfix(&bc_d_pattern, &ab_c_d_pattern),
            Ok(PatternMatch(None, Some(vec![Child::new(*a, 1)])))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&bc_d_pattern, &abc_d_pattern),
            Ok(PatternMatch(None, Some(vec![Child::new(*a, 1)])))
        );
        assert_eq!(
            graph.compare_pattern_postfix(&abcd_pattern, &bc_d_pattern),
            Ok(PatternMatch(Some(vec![Child::new(*a, 1)]), None))
        );
    }
}
