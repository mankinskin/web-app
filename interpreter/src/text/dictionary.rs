use std::collections::{HashSet, HashMap};
use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::slice::Windows;

use crate::text::*;
use crate::graph::*;
use crate::sentence::*;

#[derive(Debug)]
struct Dictionary {
    name: String,
    graph: TextGraph,
}

impl std::ops::Deref for Dictionary {
    type Target = TextGraph;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl std::ops::DerefMut for Dictionary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}
impl<'a> Dictionary {
    pub fn new<S: ToString>(name: S) -> Self {
        let mut graph =  TextGraph::new();
        let mut new = Self {
            name: name.to_string(),
            graph: TextGraph::new(),
        };
        Self {
            name: name.to_string(),
            graph,
        }
    }
    pub fn write_to_file(&self) -> std::io::Result<()> {
        self.graph.write_to_file(self.name.clone())
    }
    pub fn print_element_infos(&self) {
        // all nodes
        for node_index in self.graph.node_indices() {
            println!("{}", self.get_node(node_index));
        }
    }
} // impl Dictionary
