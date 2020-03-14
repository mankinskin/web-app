use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::fmt::{self, Debug, Display, Formatter};
use crate::sentence::*;
use crate::graph::*;

#[derive(PartialEq)]
pub enum SentenceGraphEdgeWeight {
    Empty,
}
impl<'a> Debug for SentenceGraphEdgeWeight {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "")
    }
}
pub type InternalSentenceGraph<'a> = DiGraph<TextPath<'a>, SentenceGraphEdgeWeight>;
pub struct SentenceGraph<'a> {
    graph: InternalSentenceGraph<'a>,
    root: NodeIndex,
}
use std::path::PathBuf;
impl<'a> SentenceGraph<'a> {
    pub fn from_sentence(sentence: TextPath<'a>) -> Self {
        let mut graph = DiGraph::new();
        let root = Self::build_succ_graph(&mut graph, sentence);
        Self {
            graph,
            root,
        }
    }

    //fn build_pred_graph(g: &mut InternalSentenceGraph<'a>, sentence: Sentence<'a>) -> NodeIndex {
    //    let preds = sentence.predecessors();
    //    if preds.len() == 1 {
    //        let mut new_sentence = sentence.clone();
    //        new_sentence.push_front(preds.first().unwrap().clone().into());
    //        Self::build_pred_graph(g, new_sentence) // skip node
    //    } else {
    //        let root = g.add_node(sentence.clone());
    //        for p in preds {
    //            let mut new_sentence = sentence.clone();
    //            new_sentence.push_front(p.into());

    //            let index = Self::build_pred_graph(g, new_sentence);
    //            g.add_edge(index, root, SentenceGraphWeight::Empty);
    //        }
    //        root
    //    }
    //}
    fn build_succ_graph(g: &mut InternalSentenceGraph<'a>, stack: TextPath<'a>) -> NodeIndex {
        let succs = stack.successors();
        if succs.len() == 1 {
            let mut new_sentence = stack.clone();
            new_sentence.push(succs.iter().next().unwrap().clone().into());
            return Self::build_succ_graph(g, new_sentence); // skip node
        } else {
            //println!("{}", stack);
            // if 0 or more than 1 successor
            let root = g.add_node(stack.clone());
            for s in succs {
                let mut new_sentence = stack.clone();
                new_sentence.push(s.into());

                let index = Self::build_succ_graph(g, new_sentence);
                g.add_edge(root, index, SentenceGraphEdgeWeight::Empty);
            }
            return root;
        }
    }
    pub fn write_to_file<S: Into<PathBuf>>(&self, name: S) -> std::io::Result<()> {
        let mut path: PathBuf = name.into();
        path.set_extension("dot");
        path.canonicalize();
        path.parent().map(|p|
            std::fs::create_dir_all(p.clone())
        );
        std::fs::write(path, format!("{:?}", Dot::new(&self.graph)))
    }
}

impl<'a> From<TextPath<'a>> for SentenceGraph<'a> {
    fn from(s: TextPath<'a>) -> Self {
        Self::from_sentence(s)
    }
}

impl<'a> std::ops::Deref for SentenceGraph<'a> {
    type Target = InternalSentenceGraph<'a>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<'a> std::ops::DerefMut for SentenceGraph<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

mod tests {
    use super::*;
    use crate::*;
    use crate::graph::*;
    use crate::text::*;
    use pretty_assertions::{assert_eq};
    #[test]
    fn successors() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();

        let a_stack = tg.get_text_path(vec![
            TextElement::Empty,
            Word::from("A").into(),
        ]).unwrap();
        let b_stack = tg.get_text_path(vec![
            Word::from("B").into(),
        ]).unwrap();
        let ab = tg.get_text_path(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
        let bc = tg.get_text_path(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let bcd = tg.get_text_path(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();

        let a_graph = SentenceGraph::from(a_stack);
        a_graph.write_to_file("graphs/a_graph");
    }

    #[test]
    fn text() {
        let mut tg = TextGraph::new();
        tg.read_text(crate::graph::tests::gehen_text.clone());
        //tg.write_to_file("gehen_graph");

        let stack = tg
            .get_text_path(vec![TextElement::Empty])
            .unwrap();
        //let stack_graph = SentenceGraph::from(stack);
        //stack_graph.write_to_file("graphs/sentence_empty");
    }
}
