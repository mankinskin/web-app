use crate::{
    hypergraph::{
        pattern_width,
        r#match::PatternMismatch,
        Child,
        Hypergraph,
        Indexed,
        Pattern,
        PatternId,
        PatternView,
        VertexData,
        VertexIndex,
    },
    token::Tokenize,
};
mod searcher;
pub use searcher::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound(FoundRange, VertexIndex, PatternId);
// found range of search pattern in vertex at index

impl SearchFound {
    #[allow(unused)]
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        Self(self.0.prepend_prefix(pattern), self.1, self.2)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoundRange {
    Complete,                // Full index found
    Prefix(Pattern),         // found prefix (remainder)
    Postfix(Pattern),        // found postfix (remainder)
    Infix(Pattern, Pattern), // found infix
}
impl FoundRange {
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        if pattern.is_empty() {
            return self;
        }
        match self {
            FoundRange::Complete => FoundRange::Prefix(pattern),
            FoundRange::Prefix(post) => FoundRange::Infix(pattern, post),
            FoundRange::Infix(pre, post) => {
                FoundRange::Infix([&pattern[..], &pre[..]].concat(), post)
            }
            FoundRange::Postfix(pre) => FoundRange::Postfix([&pattern[..], &pre[..]].concat()),
        }
    }
    pub fn is_matching(&self) -> bool {
        self == &FoundRange::Complete
    }
    pub fn reverse(self) -> Self {
        match self {
            Self::Complete => Self::Complete,
            Self::Prefix(post) => Self::Postfix(post),
            Self::Postfix(pre) => Self::Prefix(pre),
            Self::Infix(pre, post) => Self::Infix(post, pre),
        }
    }
}
enum NotFound {
    EmptyPattern,
    Mismatch(PatternMismatch),
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl Into<Child>>,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        Searcher::search_right(self).find_pattern(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl Into<T>>,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        Searcher::search_right(self).find_sequence(pattern)
    }
}
#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {
    use super::*;
    use crate::hypergraph::{
        tests::context,
        Child,
    };
    use pretty_assertions::assert_eq;
    macro_rules! assert_match {
        ($in:expr, $exp:expr) => {
            assert_match!($in, $exp, "")
        };
        ($in:expr, $exp:expr, $name:literal) => {
            if let Some((a, c)) = $exp {
                let a: &Child = a;
                if let Some((ia, _ib, ic)) = $in {
                    assert_eq!(ia, *a, $name);
                    assert_eq!(ic, c, $name);
                } else {
                    assert_eq!($in, Some((*a, (0, 0), c)), $name);
                }
            } else {
                assert_eq!($in, None, $name);
            }
        };
    }
    pub(crate) use assert_match;
    #[test]
    fn find_pattern() {
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
            _cdef,
            _efghi,
            _abab,
            _ababab,
            ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = &[Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = &[Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = &[Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = &[Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = &[Child::new(bc, 2)];
        let a_b_c_pattern = &[Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_match!(graph.find_pattern(bc_pattern), None);
        assert_match!(
            graph.find_pattern(b_c_pattern),
            Some((bc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_bc_pattern),
            Some((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(ab_c_pattern),
            Some((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_bc_d_pattern),
            Some((abcd, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_b_c_pattern),
            Some((abc, FoundRange::Complete))
        );
        let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern =
            &[*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        assert_match!(
            graph.find_pattern(a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern),
            Some((ababababcdefghi, FoundRange::Complete))
        );
        let a_b_c_c_pattern = &[&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_eq!(graph.find_pattern(a_b_c_c_pattern), None);
    }
    #[test]
    fn find_sequence() {
        let (
            graph,
            _a,
            _b,
            _c,
            _d,
            _e,
            _f,
            _g,
            _h,
            _i,
            _ab,
            _bc,
            _cd,
            _bcd,
            abc,
            _abcd,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            ababababcdefghi,
        ) = &*context();
        assert_match!(graph.find_sequence("a".chars()), None, "a");
        assert_match!(
            graph.find_sequence("abc".chars()),
            Some((abc, FoundRange::Complete)),
            "abc"
        );
        assert_match!(
            graph.find_sequence("ababababcdefghi".chars()),
            Some((ababababcdefghi, FoundRange::Complete)),
            "ababababcdefghi"
        );
    }
}
