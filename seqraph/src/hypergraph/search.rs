use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternId,
        Pattern,
        PatternView,
        VertexData,
        Parent,
        TokenPosition,
        Child,
        pattern_width,
        r#match::{
            IndexMatch,
        },
    },
    token::{
        Tokenize,
    },
};
use either::Either;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound(FoundRange, VertexIndex, PatternId);
// found range of search pattern in vertex at index

impl SearchFound {
    //pub fn from_match_result_on_index_at_offset(result: PatternMatch, index: VertexIndex, offset: Option<PatternId>) -> Self {
    //    let offset = offset.unwrap_or(0);
    //    match result {
    //        PatternMatch::Matching => Self(FoundRange::Complete, index, offset),
    //        PatternMatch::Remainder(Either::Left(rem)) => Self(FoundRange::Prefix(rem), index, offset),
    //    }
    //}
    #[allow(unused)]
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        Self(self.0.prepend_prefix(pattern), self.1, self.2)
    }
}
    #[allow(unused)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoundRange {
    Complete, // Full index found
    Prefix(Pattern), // found prefix (remainder)
    Postfix(Pattern), // found postfix (remainder)
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
            FoundRange::Infix(pre, post) => FoundRange::Infix([&pattern[..], &pre[..]].concat(), post),
            FoundRange::Postfix(pre) => FoundRange::Postfix([&pattern[..], &pre[..]].concat()),
        }
    }
}

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    /// find parent of vertex matching a pattern after the vertex
    /// offset: limit results to a position
    /// width_ceiling: only search parents below this width
    pub(crate) fn find_parent_matching_pattern_at_offset_below_width(
        &self,
        post_pattern: PatternView<'a>,
        vertex: &VertexData,
        offset: Option<usize>,
        width_ceiling: Option<TokenPosition>,
        ) -> Option<(VertexIndex, Parent, IndexMatch)> {
        //println!("find_parent_matching_pattern");
        let parents = vertex.get_parents();
        // optionally filter parents by width
        if let Some(ceil) = width_ceiling {
            Either::Left(parents.iter().filter(move |(_, parent)| parent.get_width() < ceil))
        } else {
            Either::Right(parents.iter())
        }
        // find matching parent
        .find_map(|(&index, parent)|
            self.compare_parent_at_offset(post_pattern, index, parent, offset)
                .map(|m| (index, parent.clone(), m))
        )
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: PatternView<'a>,
        ) -> Option<(VertexIndex, IndexMatch)> {
        let vertex = self.expect_vertex_data(pattern.get(0)?.get_index());
        if pattern.len() == 1 {
            return Some((pattern[0].get_index(), IndexMatch::Matching));
        }
        let width = pattern_width(pattern);
        //let mut pattern_iter = pattern.into_iter().cloned().enumerate();
        // accumulate prefix not found
        //let mut prefix = Vec::with_capacity(pattern_iter.size_hint().0);
        self.find_parent_matching_pattern_at_offset_below_width(
            &pattern[1..],
            vertex,
            Some(0),
             Some(width+1))
            .and_then(|(index, p, m)| match m {
                IndexMatch::SubRemainder(rem) =>
                    self.find_pattern(&[&[Child::new(index, p.get_width())], &rem[..]].concat())
                    .or(Some((index, IndexMatch::SubRemainder(rem)))),
                _ => Some((index, m)),
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::hypergraph::tests::context;
    #[test]
    fn find_simple() {
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
        let a_bc_d_pattern = &[Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = &[Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = &[Child::new(bc, 2)];
        let a_b_c_pattern = &[Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        assert_eq!(graph.find_pattern(bc_pattern), Some((*bc, IndexMatch::Matching)));
        assert_eq!(graph.find_pattern(b_c_pattern), Some((*bc, IndexMatch::Matching)));
        assert_eq!(graph.find_pattern(a_bc_pattern), Some((*abc, IndexMatch::Matching)));
        assert_eq!(graph.find_pattern(ab_c_pattern), Some((*abc, IndexMatch::Matching)));
        assert_eq!(graph.find_pattern(a_bc_d_pattern), Some((*abcd, IndexMatch::Matching)));
        assert_eq!(graph.find_pattern(a_b_c_pattern), Some((*abc, IndexMatch::Matching)));
        let a_b_c_c_pattern = &[&a_b_c_pattern[..], &[Child::new(*c, 1)]].concat();
        assert_eq!(graph.find_pattern(a_b_c_c_pattern), Some((*abc, IndexMatch::SubRemainder(vec![Child::new(*c, 1)]))));
    }
}