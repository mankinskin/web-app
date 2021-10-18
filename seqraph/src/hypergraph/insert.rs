use crate::{
    hypergraph::*,
    token::*,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    fn next_pattern_vertex_id() -> VertexIndex {
        static VERTEX_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        VERTEX_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    /// insert single token node
    pub fn insert_vertex(&mut self, key: VertexKey<T>, mut data: VertexData) -> Child {
        // TODO: return error if exists (don't overwrite by default)
        //let index = insert_full(key, data).0;
        let entry = self.graph.entry(key);
        data.index = entry.index();
        let c = Child::new(data.index, data.width);
        entry.or_insert(data);
        c
    }
    /// insert single token node
    pub fn insert_token(&mut self, token: Token<T>) -> Child {
        self.insert_vertex(VertexKey::Token(token), VertexData::new(0, 1))
    }
    /// insert multiple token nodes
    pub fn insert_tokens(&mut self, tokens: impl IntoIterator<Item = Token<T>>) -> Vec<Child> {
        tokens
            .into_iter()
            .map(|token| self.insert_vertex(VertexKey::Token(token), VertexData::new(0, 1)))
            .collect()
    }
    /// utility, builds total width, indices and children for pattern
    fn to_width_indices_children(
        &self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> (TokenPosition, Vec<VertexIndex>, Vec<Child>) {
        let mut width = 0;
        let (a, b) = indices
            .into_iter()
            .map(|index| {
                let index = *index.index();
                let w = self.expect_vertex_data(index).get_width();
                width += w;
                (index, Child::new(index, w))
            })
            .unzip();
        (width, a, b)
    }
    /// adds a parent to all nodes in a pattern
    fn add_parents_to_pattern_nodes(
        &mut self,
        pattern: Vec<VertexIndex>,
        parent: impl ToChild,
        pattern_id: PatternId,
    ) {
        for (i, child_index) in pattern.into_iter().enumerate() {
            let node = self.expect_vertex_data_mut(child_index);
            node.add_parent(parent.to_child(), pattern_id, i);
        }
    }
    /// add pattern to existing node
    pub fn add_pattern_to_node(
        &mut self,
        index: impl Indexed,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> PatternId {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index.index());
        let pattern_id = data.add_pattern(&children);
        self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_id);
        pattern_id
    }
    /// add pattern to existing node
    pub fn add_patterns_to_node(
        &mut self,
        index: impl Indexed,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> Vec<PatternId> {
        let index = index.index();
        patterns
            .into_iter()
            .map(|p| self.add_pattern_to_node(index, p))
            .collect()
    }
    /// create new node from a pattern
    pub fn insert_pattern_with_id(&mut self, indices: impl IntoIterator<Item = impl Indexed>) -> (Child, Option<PatternId>) {
        // todo check if exists already
        // todo handle token nodes
        let indices: Vec<_> = indices.into_iter().collect();
        if indices.len() == 1 {
            (self.to_child(indices.first().unwrap().index()), None)
        } else {
            let (width, indices, children) = self.to_width_indices_children(indices);
            let mut new_data = VertexData::new(0, width);
            let pattern_index = new_data.add_pattern(&children);
            let id = Self::next_pattern_vertex_id();
            let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
            self.add_parents_to_pattern_nodes(indices, Child::new(index, width), pattern_index);
            (index, Some(pattern_index))
        }
    }
    /// create new node from a pattern
    pub fn insert_pattern(&mut self, indices: impl IntoIterator<Item = impl Indexed>) -> Child {
        self.insert_pattern_with_id(indices).0
    }
    /// create new node from multiple patterns
    pub fn insert_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl IntoIterator<Item = impl Indexed>>,
    ) -> Child {
        // todo handle token nodes
        let mut patterns = patterns.into_iter();
        let first = patterns.next().expect("Tired to insert no patterns");
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
        replace: impl IntoIterator<Item = Child> + Clone,
    ) {
        let mut peek_range = range.clone().peekable();
        if peek_range.peek().is_none() {
            return;
        }
        let parent = parent.index();
        let (old, width) = {
            let vertex = self.expect_vertex_data_mut(parent);
            (
                vertex.replace_in_pattern(pat, range.clone(), replace.clone()),
                vertex.width,
            )
        };
        range.clone().zip(old.iter()).for_each(|(pos, c)| {
            let c = self.expect_vertex_data_mut(c);
            c.remove_parent(parent, pat, pos);
        });
        let start = range.next().unwrap();
        self.add_pattern_parent(Child::new(parent, width), replace, pat, start);
    }
    pub(crate) fn add_pattern_parent(
        &mut self,
        parent: impl ToChild,
        pattern: impl IntoIterator<Item = impl ToChild>,
        pattern_id: PatternId,
        start: usize,
    ) {
        pattern.into_iter().enumerate().for_each(|(pos, c)| {
            let pos = start + pos;
            let c = self.expect_vertex_data_mut(c);
            c.add_parent(parent.to_child(), pattern_id, pos);
        });
    }
    pub(crate) fn append_to_pattern(
        &mut self,
        parent: impl ToChild,
        pattern_id: PatternId,
        new: impl IntoIterator<Item = impl ToChild>,
    ) {
        let new: Vec<_> = new.into_iter().map(|c| c.to_child()).collect();
        let offset = {
            let vertex = self.expect_vertex_data_mut(parent.index());
            let pattern = vertex.expect_child_pattern_mut(&pattern_id);
            let offset = pattern.len();
            pattern.extend(new.iter());
            offset
        };
        self.add_pattern_parent(parent, new, pattern_id, offset);
    }
    //pub fn read_sequence(&mut self, sequence: impl IntoIterator<Item = T>) -> Child {
    //    IndexReader::new(self).read_sequence(sequence)
    //}
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
        ])[..]
        {
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
