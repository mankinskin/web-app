use crate::{
    hypergraph::*,
    token::{
        Token,
        Tokenize,
    },
};
use std::{
    num::NonZeroUsize,
    ops::{RangeFrom, Range},
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    fn next_vertex_id() -> VertexIndex {
        static VERTEX_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        VERTEX_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    /// insert single token node
    pub fn insert_vertex(&mut self, key: VertexKey<T>, data: VertexData) -> Child {
        // TODO: return error if exists (don't overwrite by default)
        let width = data.width;
        Child::new(self.graph.insert_full(key, data).0, width)
    }
    /// insert single token node
    pub fn insert_token(&mut self, token: Token<T>) -> Child {
        self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
    }
    /// insert multiple token nodes
    pub fn insert_tokens(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Vec<Child> {
        tokens.into_iter()
            .map(|token|
                self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
            )
            .collect()
    }
    /// utility, builds total width, indices and children for pattern
    fn to_width_indices_children(
        &self,
        indices: impl IntoIterator<Item=impl Indexed>,
        ) -> (TokenPosition, Vec<VertexIndex>, Vec<Child>) {
        let mut width = 0;
        let (a, b) = indices.into_iter()
            .map(|index| {
                let index = *index.borrow();
                let w = self.expect_vertex_data(index).get_width();
                width += w;
                (index, Child::new(index, w))
            })
            .unzip();
        (width, a, b)
    }
    /// adds a parent to all nodes in a pattern
    fn add_parents_to_pattern_nodes(&mut self, pattern: Vec<VertexIndex>, parent_index: impl Indexed, width: TokenPosition, pattern_id: PatternId) {
        for (i, child_index) in pattern.into_iter().enumerate() {
            let node = self.expect_vertex_data_mut(child_index);
            node.add_parent(parent_index.borrow(), width, pattern_id, i);
        }
    }
    /// add pattern to existing node
    pub fn add_pattern_to_node(&mut self, index: impl Indexed, indices: impl IntoIterator<Item=impl Indexed>) -> PatternId {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index.borrow());
        let pattern_id = data.add_pattern(&children);
        self.add_parents_to_pattern_nodes(indices, index.borrow(), width, pattern_id);
        pattern_id
    }
    /// add pattern to existing node
    pub fn add_patterns_to_node(&mut self, index: impl Indexed, patterns: impl IntoIterator<Item=impl IntoIterator<Item=impl Indexed>>) -> Vec<PatternId> {
        let index = index.index();
        patterns.into_iter()
            .map(|p| self.add_pattern_to_node(index, p))
            .collect()
    }
    /// create new node from a pattern
    pub fn insert_pattern(&mut self, indices: impl IntoIterator<Item=impl Indexed>) -> Child {
        // todo check if exists already
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let mut new_data = VertexData::with_width(width);
        let pattern_index = new_data.add_pattern(&children);
        let id = Self::next_vertex_id();
        let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
        self.add_parents_to_pattern_nodes(indices, index, width, pattern_index);
        index
    }
    /// create new node from multiple patterns
    pub fn insert_patterns(&mut self, patterns: impl IntoIterator<Item=impl IntoIterator<Item=impl Indexed>>) -> Child {
        // todo handle token nodes
        let mut patterns = patterns.into_iter();
        let first = patterns.next().unwrap();
        let node = self.insert_pattern(first);
        for pat in patterns {
            self.add_pattern_to_node(&node, pat);
        }
        node
    }
    pub fn replace_in_pattern(
        &mut self,
        parent: impl Indexed,
        pat: PatternId,
        mut range: impl PatternRangeIndex + Clone,
        replace: impl IntoIterator<Item=Child> + Clone,
    ) {
        let mut peek_range = range.clone().peekable();
        if peek_range.peek().is_none() {
            return;
        }
        let parent = parent.index();
        let (old, width) = {
            let vertex = self.expect_vertex_data_mut(parent);
            (vertex.replace_in_pattern(pat, range.clone(), replace.clone()), vertex.width)
        };
        range.clone().zip(old.iter()).for_each(|(pos, c)| {
            let c = self.expect_vertex_data_mut(c);
            c.remove_parent(parent, pat, pos);
        });
        let start = range.next().unwrap();
        self.add_pattern_parent_with_width(parent, replace, pat, start, width);
    }
    pub(crate) fn add_pattern_parent_with_width(
        &mut self,
        parent: impl Indexed,
        pattern: impl IntoIterator<Item=Child>,
        pattern_id: PatternId,
        start: usize,
        parent_width: usize,
    ) {
        let parent = parent.index();
        pattern.into_iter().enumerate().for_each(|(pos, c)| {
            let pos = start + pos;
            let c = self.expect_vertex_data_mut(c);
            c.add_parent(parent, parent_width, pattern_id, pos);
        });
    }
    #[allow(unused)]
    pub(crate) fn add_pattern_parent(
        &mut self,
        parent: impl Indexed,
        pattern: impl IntoIterator<Item=Child>,
        pattern_id: PatternId,
        start: usize,
    ) {
        let parent = parent.index();
        let width = self.expect_vertex_data(parent).width;
        self.add_pattern_parent_with_width(parent, pattern, pattern_id, start, width)
    }

}
trait ChildUpdate {
    type R: PatternRangeIndex + Clone;
    fn get_context(s: SplitContext) -> Pattern;
    fn replace_range(p: usize) -> Self::R;
    fn replace_new_index_in_index<T: Tokenize>(
        g: &mut Hypergraph<T>,
        index: Child,
        parent: Child,
    ) {
        let (ps, rem) = g.separate_perfect_split(parent, NonZeroUsize::new(index.width).unwrap());
        Self::replace_new_index_in_split_contexts(g, index, rem);
        Self::replace_new_index_in_perfect_split(g, index, parent, ps);
    }
    fn replace_new_index_in_perfect_split<T: Tokenize>(
        g: &mut Hypergraph<T>,
        index: Child,
        parent: Child,
        ps: Option<(Split, IndexInParent)>,
    ) {
        if let Some((_, id)) = ps {
            g.replace_in_pattern(parent, id.pattern_index, Self::replace_range(id.replaced_index), [index]);
        }
    }
    fn replace_new_index_in_split_contexts<T: Tokenize>(
        g: &mut Hypergraph<T>,
        index: Child,
        patterns: impl IntoIterator<Item=SplitContext>,
    ) {
        if let Some(smallest_larger) = patterns.into_iter().filter_map(|s|
            Self::get_context(s).first().filter(|c| c.width > index.width).cloned()
        ).min_by(|a, b| a.width.cmp(&b.width)) {
            Self::replace_new_index_in_index(g, index, smallest_larger);
        }
    }
}
struct UpdateLeft;
impl ChildUpdate for UpdateLeft {
    type R = Range<usize>;
    fn get_context(s: SplitContext) -> Pattern {
        s.prefix
    }
    fn replace_range(p: usize) -> Self::R {
        0..p
    }
}
struct UpdateRight;
impl ChildUpdate for UpdateRight {
    type R = RangeFrom<usize>;
    fn get_context(s: SplitContext) -> Pattern {
        s.postfix
    }
    fn replace_range(p: usize) -> Self::R {
        p..
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_subpattern() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
            ])[..] {
            let _abcd = graph.insert_pattern([a, b, c, d]);
            // read abcd
            // then abe
            // then bce
            // then cde
        } else {
            panic!()
        }

    }
}
