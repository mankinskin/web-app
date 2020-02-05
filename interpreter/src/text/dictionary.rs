use crate::text::*;
use std::collections::{HashSet, HashMap};
use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
};
use std::slice::Windows;

#[derive(Debug)]
struct Dictionary {
    name: String,
    graph: DiGraph<TextElement, HashSet<usize>>,
}

impl Dictionary {
    pub fn new<S: ToString>(name: S) -> Self {
        let mut new = Self {
            name: name.to_string(),
            graph: DiGraph::new(),
        };
        // TODO should use enum_iterator with is_stop()
        // All stop symbols could be followed by empty
        new.add_edge(&TextElement::Punctuation(Punctuation::Dot),
            &TextElement::Empty, 1);
        new
    }
    pub fn contains(&self, element: &TextElement) -> bool {
        self.get_node_index(element).is_some()
    }
    fn get_node_index(&self, element: &TextElement) -> Option<NodeIndex> {
        self.graph.node_indices()
            .find(|i| self.graph[*i] == *element)
            .map(|i| i.clone())
    }
    pub fn add(&mut self, element: &TextElement) -> NodeIndex {
        match self.get_node_index(element) {
            Some(i) => i,
            None => {
                self.graph.add_node(element.clone())
            }
        }
    }
    pub fn add_edge(&mut self, l: &TextElement, r: &TextElement, distance: usize) {
        let li = self.add(l);
        let ri = self.add(r);
        let old_edge = self.graph.find_edge(li, ri);
        match old_edge {
            Some(i) => {
                self.graph.edge_weight_mut(i).unwrap().insert(distance);
            },
            None => {
                let mut new = HashSet::new();
                new.insert(distance);
                self.graph.update_edge(li, ri, new);
            }
        }
    }
    pub fn insert_elements(&mut self, l: &TextElement, r: &TextElement, distance: usize) {
        if l.is_stop() {
            self.add_edge(&TextElement::Empty, r, distance);
        } else {
            self.add_edge(l, r, distance);
        }
    }
    pub fn insert_text(&mut self, text: Text) {
        let len = text.len();
        if len > 0 {
            self.insert_elements(&TextElement::Empty, &text[0], 1);
        }
        let mut next_stop = 0;
        for i in 0..len-1 {
            if i == next_stop {
                // search for next stop symbol
                // to stop counting distance between elements
                while {
                    next_stop += 1;
                    next_stop < len && !text[next_stop].is_stop()
                }
                { }
                //continue;
            }
            for j in (i+1)..=next_stop {
                let left = &text[i];
                let right = &text[j];
                self.insert_elements(left, right, j-i);
            }
        }
        for elements in (text[..]).windows(2) {
            let (l, r) = (elements[0].clone(), elements[1].clone());
        }
    }
    pub fn write_to_file(&self) -> std::io::Result<()> {
        std::fs::write(
            self.name.clone() + ".dot",
            format!("{:?}", Dot::new(&self.graph)))
    }
}

//fn context_patterns(e: TextElement, pattern: Text) -> ElementEntry {
//    let mut parts = Vec::new();
//    let mut buf: Text = Text::new();
//    let mut occurrences = 0;
//    for elem in pattern {
//        if elem == e {
//            parts.push(buf.clone());
//            buf.clear();
//            occurrences += 1;
//        } else {
//            buf.push(elem);
//        }
//    }
//    parts.push(buf);
//    let mut contexts = Vec::new();
//    for window in (&parts).windows(2) {
//        contexts.push((window[0].clone(), window[1].clone()).into());
//    }
//    ElementEntry {
//        element: e.clone(),
//        occurrences,
//        contexts
//    }
//}
//fn all_context_patterns(pattern: Text) -> HashSet<ElementEntry> {
//    let mut set = HashSet::new();
//    for word in pattern.unique_elements() {
//        set.insert(context_patterns(word, pattern.clone()));
//    }
//    set
//}
//
//#[derive(Hash, Eq, PartialEq)]
//struct ElementContext {
//    before: Text,
//    after: Text,
//}
//use std::fmt::{Debug, Display, self};
//impl Debug for ElementContext {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        write!(f, "{} _ {}", self.before, self.after)
//    }
//}
//impl Display for ElementContext {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        write!(f, "{:?}", self)
//    }
//}
//impl From<(Text, Text)> for ElementContext {
//    fn from((before, after): (Text, Text)) -> Self {
//        Self {
//            before,
//            after,
//        }
//    } }
//
//#[derive(Hash, Eq, PartialEq)]
//struct ElementEntry {
//    element: TextElement,
//    occurrences: u32,
//    contexts: Vec<ElementContext>,
//}

mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn examples() {
        let text = "
        Ich gehe.
        Du gehst.
        Er geht.
        Sie geht.
        Es geht.
        Wir gehen.
        Ihr geht.
        Sie gehen.
        Ich ging.

        Du gingst.
        Er ging.
        Sie ging.
        Es ging.
        Wir gingen.
        Ihr gingt.
        Sie gingen.

        Ich bin gegangen.
        Du bist gegangen.
        Er ist gegangen.
        Sie ist gegangen.
        Es ist gegangen.
        Wir sind gegangen.
        Ihr seid gegangen.
        Sie sind gegangen.
        ";
        let text: Text = Text::parse(&text).unwrap().1;
        //let mut occurrences = text.element_occurrences().into_iter().collect::<Vec<_>>();
        //occurrences.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        //let occurrences: Vec<String> =
        //    occurrences.into_iter()
        //               .map(|(k, v)| k.to_string() + ": " + &v.to_string())
        //               .collect();
        //println!("{:#?}", text);
        let mut dictionary: Dictionary = Dictionary::new("test_dictionary");
        dictionary.insert_text(text);

        dictionary.write_to_file();

        //let pattern_map = all_context_patterns(text.clone());
        //println!("{:#?}", pattern_map);
    }
}
