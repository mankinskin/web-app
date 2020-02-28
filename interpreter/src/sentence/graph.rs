use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::fmt::{self, Debug, Display, Formatter};
use crate::sentence::*;

pub enum SentenceGraphWeight {
    Empty,
}
impl<'a> Debug for SentenceGraphWeight {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "")
    }
}
pub type InternalSentenceGraph<'a> = DiGraph<Sentence<'a>, SentenceGraphWeight>;
pub struct SentenceGraph<'a> {
    graph: InternalSentenceGraph<'a>,
    root: NodeIndex,
}

impl<'a> SentenceGraph<'a> {
    pub fn from_sentence(sentence: Sentence<'a>) -> Self {
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
    fn build_succ_graph(g: &mut InternalSentenceGraph<'a>, sentence: Sentence<'a>) -> NodeIndex {
        println!("{}", sentence);
        let succs = sentence.successors();
        if succs.len() == 1 {
            let mut new_sentence = sentence.clone();
            new_sentence.push(succs.iter().next().unwrap().clone().into());
            return Self::build_succ_graph(g, new_sentence); // skip node
        } else {
            // if 0 or more than 1 successor
            let root = g.add_node(sentence.clone());
            for s in succs {
                let mut new_sentence = sentence.clone();
                new_sentence.push(s.into());

                let index = Self::build_succ_graph(g, new_sentence);
                g.add_edge(root, index, SentenceGraphWeight::Empty);
            }
            return root;
        }
    }
    pub fn write_to_file<S: Into<String>>(&self, name: S) -> std::io::Result<()> {
        std::fs::write(
            name.into() + ".dot",
            format!("{:?}", Dot::new(&self.graph)))
    }
}

impl<'a> From<Sentence<'a>> for SentenceGraph<'a> {
    fn from(s: Sentence<'a>) -> Self {
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

