use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternId,
        Pattern,
        PatternView,
        VertexData,
        Parent,
        r#match::PatternMismatch,
        pattern_width,
    },
    token::{
        Tokenize,
    },
};

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
    where T: Tokenize + 't,
{
    pub(crate) fn find_parent_matching_context(
        &self,
        postfix: PatternView<'a>,
        vertex: &VertexData,
        ) -> Option<(VertexIndex, Parent, FoundRange)> {
        //println!("find_parent_matching_pattern");
        // find matching parent
        let width = pattern_width(postfix) + vertex.width;
        self.right_matcher()
            .find_parent_matching_context_below_width(postfix, vertex, Some(width + 1))
            .map(|(index, parent, pre, m)| (index, parent, m.prepend_prefix(pre)))
    }
    pub(crate) fn find_postfix_for(
        &self,
        index: VertexIndex,
        postfix: PatternView<'a>,
    ) -> Option<(VertexIndex, FoundRange)> {
        let vertex = self.expect_vertex_data(index);
        let (index, _parent, found_range) = self.find_parent_matching_context(
            postfix,
            vertex,
        )?;
        match found_range {
            FoundRange::Complete => Some((index, found_range)),
            FoundRange::Prefix(post) => self.find_postfix_for(index, &post[..]),
            // todo: match prefixes
            _ => Some((index, found_range)),
        }
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: PatternView<'a>,
        ) -> Option<(VertexIndex, FoundRange)> {
        let index = pattern.get(0)?.get_index();
        if pattern.len() == 1 {
            // pattern is the response
            return Some((pattern[0].get_index(), FoundRange::Complete));
        }
        let postfix = &pattern[1..];
        self.find_postfix_for(index, postfix)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item=impl Into<T>>,
        ) -> Option<(VertexIndex, FoundRange)> {
        let pattern = T::tokenize(pattern.into_iter());
        let pattern = self.to_token_children(pattern)?;
        self.find_pattern(&pattern)
    }
}
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::hypergraph::{
        Child,
        tests::context,
    };
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
        assert_eq!(graph.find_pattern(bc_pattern), Some((*bc, FoundRange::Complete)));
        assert_eq!(graph.find_pattern(b_c_pattern), Some((*bc, FoundRange::Complete)));
        assert_eq!(graph.find_pattern(a_bc_pattern), Some((*abc, FoundRange::Complete)));
        assert_eq!(graph.find_pattern(ab_c_pattern), Some((*abc, FoundRange::Complete)));
        assert_eq!(graph.find_pattern(a_bc_d_pattern), Some((*abcd, FoundRange::Complete)));
        assert_eq!(graph.find_pattern(a_b_c_pattern), Some((*abc, FoundRange::Complete)));
        let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern = &[
            Child::new(a, 1), Child::new(b, 1), Child::new(a, 1), Child::new(b, 1),
            Child::new(a, 1), Child::new(b, 1), Child::new(a, 1), Child::new(b, 1),
            Child::new(c, 1), Child::new(d, 1), Child::new(e, 1), Child::new(f, 1),
            Child::new(g, 1), Child::new(h, 1), Child::new(i, 1)
        ];
        assert_eq!(graph.find_pattern(a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern), Some((*ababababcdefghi, FoundRange::Complete)));
        let a_b_c_c_pattern = &[&a_b_c_pattern[..], &[Child::new(*c, 1)]].concat();
        assert_eq!(graph.find_pattern(a_b_c_c_pattern), None);
    }
    #[test]
    fn find_sequence() {
        let (
            graph,
            a,
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
        assert_eq!(graph.find_sequence("a".chars()), Some((*a, FoundRange::Complete)), "a");
        assert_eq!(graph.find_sequence("abc".chars()), Some((*abc, FoundRange::Complete)), "abc");
        assert_eq!(graph.find_sequence("ababababcdefghi".chars()), Some((*ababababcdefghi, FoundRange::Complete)), "ababababcdefghi");
    }
}