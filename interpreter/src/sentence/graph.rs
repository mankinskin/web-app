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
    fn build_succ_graph(g: &mut InternalSentenceGraph<'a>, path: TextPath<'a>) -> NodeIndex {
        println!("{}", path);
        let succs = path.successors();
        println!("Successors {:#?}", succs);
        if succs.len() == 1 {

            let succ = succs.iter().next().unwrap().clone();
            println!("Successor {:#?}", succ);

            if (succ.weight().element() == &TextElement::Empty) {
                // stop here
                return g.add_node(path.clone());
            }
            let next_node = TextPath::from_node(&succ);
            let new_sentence = TextPath::try_merge(path.clone(), next_node).unwrap();

            return Self::build_succ_graph(g, new_sentence); // skip node

        } else {
            // if 0 or more than 1 successor
            // add path as node
            let root = g.add_node(path.clone());
            for s in succs {
                println!("Successor {:#?}", s);

                if (s.weight().element() != &TextElement::Empty) {
                    let next_node = TextPath::from_node(&s);
                    let new_sentence = TextPath::try_merge(path.clone(), next_node).unwrap();

                    let index = Self::build_succ_graph(g, new_sentence);
                    g.add_edge(root, index, SentenceGraphEdgeWeight::Empty);
                }
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
    use crate::graph::path::tests::*;
    use crate::text::*;
    use pretty_assertions::{assert_eq};
    #[test]
    fn from_path() {
        let start_a_graph = SentenceGraph::from(START_A_PATH.clone());
        start_a_graph.write_to_file("graphs/start_a_graph");
    }

    #[test]
    fn text() {
        let mut tg = TextGraph::new();
        tg.read_text(crate::graph::tests::gehen_text.clone());
        tg.write_to_file("gehen_graph");

        let stack = tg
            .get_text_path(vec![EMPTY.clone()])
            .unwrap();
        let stack_graph = SentenceGraph::from(stack);
        //stack_graph.write_to_file("graphs/sentence_empty");
    }
}
