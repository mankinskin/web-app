use crate::{
    hypergraph::{
        r#match::*,
        Child,
        Hypergraph,
        Pattern,
        PatternId,
    },
    token::Tokenize,
};
mod searcher;
pub use searcher::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound {
    pub index: Child,
    pub parent_match: ParentMatch,
    pub pattern_id: PatternId,
    pub sub_index: usize,
}
// found range of search pattern in vertex at index
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoundRange {
    Complete,                // Full index found
    Prefix(Pattern),         // found prefix (remainder)
    Postfix(Pattern),        // found postfix (remainder)
    Infix(Pattern, Pattern), // found infix
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NotFound {
    EmptyPatterns,
    Mismatch(PatternMismatch),
    NoChildPatterns,
    NoMatchingParent,
    SingleIndex,
    UnknownTokens,
}
pub type SearchResult = Result<SearchFound, NotFound>;
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
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl Into<Child>>,
    ) -> SearchResult {
        Searcher::search_right(self).find_pattern(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl Into<T>>,
    ) -> SearchResult {
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
            match $exp {
                Ok((a, c)) => {
                    let a: &Child = a;
                    if let Ok(SearchFound {
                        index,
                        parent_match,
                        ..
                        }) = $in {
                        assert_eq!(index, *a, $name);
                        assert_eq!(parent_match.parent_range, c, $name);
                    } else {
                        assert_eq!($in, Ok(SearchFound {
                            index: *a,
                            pattern_id: 0,
                            sub_index: 0,
                            parent_match: ParentMatch {
                                parent_range: c,
                                remainder: None,
                            },
                        }), $name);
                    }
                },
                Err(err) => assert_eq!($in, Err(err), $name),
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
        assert_match!(graph.find_pattern(bc_pattern), Err(NotFound::SingleIndex));
        assert_match!(
            graph.find_pattern(b_c_pattern),
            Ok((bc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_bc_pattern),
            Ok((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(ab_c_pattern),
            Ok((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_bc_d_pattern),
            Ok((abcd, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(a_b_c_pattern),
            Ok((abc, FoundRange::Complete))
        );
        let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern =
            &[*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        assert_match!(
            graph.find_pattern(a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern),
            Ok((ababababcdefghi, FoundRange::Complete))
        );
        let a_b_c_c_pattern = &[&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_eq!(graph.find_pattern(a_b_c_c_pattern), Err(NotFound::NoMatchingParent));
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
        //assert_match!(graph.find_sequence("a".chars()), Err(NotFound::SingleIndex), "a");
        assert_match!(
            graph.find_sequence("abc".chars()),
            Ok((abc, FoundRange::Complete)),
            "abc"
        );
        assert_match!(
            graph.find_sequence("ababababcdefghi".chars()),
            Ok((ababababcdefghi, FoundRange::Complete)),
            "ababababcdefghi"
        );
    }
}
