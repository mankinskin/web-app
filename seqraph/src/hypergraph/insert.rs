use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        VertexData,
        VertexKey,
        TokenPosition,
        Child,
    },
    token::{
        Token,
        Tokenize,
    },
};
use std::borrow::Borrow;

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    /// insert single token node
    pub fn insert_vertex(&mut self, key: VertexKey<T>, data: VertexData) -> VertexIndex {
        // TODO: return error if exists (don't overwrite by default)
        self.graph.insert_full(key, data).0
    }
    /// insert single token node
    pub fn insert_token(&mut self, token: Token<T>) -> VertexIndex {
        self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
    }
    /// insert multiple token nodes
    pub fn insert_tokens(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Vec<VertexIndex> {
        tokens.into_iter()
            .map(|token|
                self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
            )
            .collect()
    }
    /// utility, builds total width, indices and children for pattern
    fn to_width_indices_children(
        &self,
        indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>,
        ) -> (TokenPosition, Vec<VertexIndex>, Vec<Child>) {
        let mut width = 0;
        let (a, b) = indices.into_iter()
            .map(|index| {
                let index = *index.borrow();
                let w = self.expect_vertex_data(index).width.clone();
                width += w;
                (index, Child::new(index, w))
            })
            .unzip();
        (width, a, b)
    }
    /// adds a parent to all nodes in a pattern
    fn add_parents_to_pattern_nodes(&mut self, pattern: Vec<VertexIndex>, parent_index: VertexIndex, width: TokenPosition, pattern_index: usize) {
        for (i, child_index) in pattern.into_iter().enumerate() {
            let node = self.expect_vertex_data_mut(child_index);
            node.add_parent(parent_index, width, pattern_index, i);
        }
    }
    pub fn insert_to_pattern(&mut self, index: VertexIndex, indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>) -> usize {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index);
        let pattern_index = data.add_pattern(&children);
        self.add_parents_to_pattern_nodes(indices, index, width, pattern_index);
        pattern_index
    }
    pub fn insert_pattern(&mut self, indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>) -> VertexIndex {
        // todo check if exists already
        let id = self.next_pattern_id();
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let mut new_data = VertexData::with_width(width);
        let pattern_index = new_data.add_pattern(&children);
        let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
        self.add_parents_to_pattern_nodes(indices, index, width, pattern_index);
        index
    }
    pub fn insert_patterns(&mut self, indices: impl IntoIterator<Item=impl IntoIterator<Item=impl Borrow<VertexIndex>>>) -> usize {
        // todo handle token nodes
        let mut iter = indices.into_iter();
        let first = iter.next().unwrap();
        let node = self.insert_pattern(first);
        for pat in iter {
            self.insert_to_pattern(node, pat);
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::*;
    #[test]
    fn insert_subpattern() {
        let mut graph = Hypergraph::new();
        if let [a, b, c, d] = graph.insert_tokens([
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
            ])[..] {
            let abcd = graph.insert_pattern([a, b, c, d]);
            // read abcd
            // then abe
            // then bce
            // then cde
        } else {
            panic!()
        }

    }
}