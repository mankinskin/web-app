use crate::{
    r#match::*,
    vertex::*,
    *,
};
mod searcher;
pub use searcher::*;
//mod async_searcher;
//pub use async_searcher::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NotFound {
    EmptyPatterns,
    Mismatch(PatternMismatch),
    NoChildPatterns,
    NoMatchingParent(VertexIndex),
    SingleIndex,
    UnknownKey,
    UnknownIndex,
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn right_searcher(&'a self) -> Searcher<'a, T, MatchRight> {
        Searcher::new(self)
    }
    #[allow(unused)]
    pub(crate) fn left_searcher(&'a self) -> Searcher<'a, T, MatchLeft> {
        Searcher::new(self)
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> SearchResult {
        self.right_searcher().find_pattern(pattern)
    }
    pub fn find_sequence(&self, pattern: impl IntoIterator<Item = impl Into<T>>) -> SearchResult {
        self.right_searcher().find_sequence(pattern)
    }
}
#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        graph::tests::context,
        Child,
    };
    use pretty_assertions::{
        assert_eq,
        assert_matches,
    };
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
                    }) = $in
                    {
                        assert_eq!(index, *a, $name);
                        assert_eq!(parent_match.parent_range, c, $name);
                    } else {
                        assert_eq!(
                            $in,
                            Ok(SearchFound {
                                index: *a,
                                pattern_id: 0,
                                sub_index: 0,
                                parent_match: ParentMatch {
                                    parent_range: c,
                                    remainder: None,
                                },
                            }),
                            $name
                        );
                    }
                }
                Err(err) => assert_eq!($in, Err(err), $name),
            }
        };
    }
    pub(crate) use assert_match;
    #[test]
    fn find_pattern_1() {
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
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            ababababcdefghi,
        ) = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_match!(graph.find_pattern(&bc_pattern), Err(NotFound::SingleIndex));
        assert_match!(
            graph.find_pattern(&b_c_pattern),
            Ok((bc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(&a_bc_pattern),
            Ok((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(&ab_c_pattern),
            Ok((abc, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(&a_bc_d_pattern),
            Ok((abcd, FoundRange::Complete))
        );
        assert_match!(
            graph.find_pattern(&a_b_c_pattern),
            Ok((abc, FoundRange::Complete))
        );
        let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern =
            vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        assert_match!(
            graph.find_pattern(&a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern),
            Ok((ababababcdefghi, FoundRange::Complete))
        );
        let a_b_c_c_pattern = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_matches!(
            graph.find_pattern(a_b_c_c_pattern),
            Err(NotFound::NoMatchingParent(_))
        );
    }
    #[test]
    fn find_pattern_2() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let _xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let byz_found = graph.find_pattern(vec![by, z]);
            let x_a_pattern = vec![x, a];
            assert_matches!(
                byz_found,
                Ok(SearchFound {
                    index: Child { width: 5, .. },
                    parent_match: ParentMatch {
                        parent_range: FoundRange::Postfix(_),
                        ..
                    },
                    ..
                })
            );
            let post = byz_found.unwrap().parent_match.parent_range;
            assert_eq!(post, FoundRange::Postfix(x_a_pattern));
        } else {
            panic!();
        }
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
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            ababababcdefghi,
        ) = &*context();
        assert_match!(
            graph.find_sequence("a".chars()),
            Err(NotFound::SingleIndex),
            "a"
        );

        let abc_found = graph.find_sequence("abc".chars());
        assert_matches!(
            abc_found,
            Ok(SearchFound {
                parent_match: ParentMatch {
                    parent_range: FoundRange::Complete,
                    ..
                },
                ..
            })
        );
        assert_eq!(abc_found.unwrap().index, *abc);
        let ababababcdefghi_found = graph.find_sequence("ababababcdefghi".chars());
        assert_matches!(
            ababababcdefghi_found,
            Ok(SearchFound {
                parent_match: ParentMatch {
                    parent_range: FoundRange::Complete,
                    ..
                },
                ..
            })
        );
        assert_eq!(ababababcdefghi_found.unwrap().index, *ababababcdefghi);
    }
}
