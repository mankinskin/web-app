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
    graph: DiGraph<TextElement, usize>,
}

impl Dictionary {
    pub fn new<S: ToString>(name: S) -> Self {
        Self {
            name: name.to_string(),
            graph: DiGraph::new(),
        }
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
    pub fn add_elements(&mut self, l: &TextElement, r: &TextElement, distance: usize) {
        let li = self.add(l);
        let ri = self.add(r);
        let old_edge = self.graph.find_edge(li, ri);
        match old_edge {
            Some(i) => {
                *self.graph.edge_weight_mut(i).unwrap() += 1;
            },
            None => {
                self.graph.update_edge(li, ri, distance);
            }
        }
    }
    pub fn insert_text(&mut self, text: Text) {
        let len = text.len();
        for i in 0..len-1 {
            let left = &text[i];
            //for j in (i+1)..len {
                let right = &text[i+1];
                self.add_elements(left, right, 1);
            //}
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
        let text = "Als Klasse gilt in der Mathematik, Klassenlogik und \
                    Mengenlehre eine Zusammenfassung beliebiger Objekte, \
                    definiert durch eine logische Eigenschaft, die alle \
                    Objekte der Klasse erfuellen. Vom Klassenbegriff ist der \
                    Mengenbegriff zu unterscheiden. Nicht alle Klassen sind \
                    automatisch auch Mengen, weil Mengen zusätzliche \
                    Bedingungen erfüllen müssen. Mengen sind aber stets \
                    Klassen und werden daher auch in der Praxis in \
                    Klassenschreibweise notiert. \
                    In der Mathematik des 19. Jahrhunderts wurden die Begriffe\
                     „Klasse“ und „Menge“ weitgehend synonym verwendet und \
                    waren ungenügend festgelegt, so dass widersprüchliche \
                    Interpretationen möglich waren. Im 20. Jahrhundert wurden \
                    sie im Zuge der Axiomatisierung der Mengenlehre getrennt \
                    und nach und nach präzisiert. Der Begriff „Klasse“ wird \
                    seither oft umfassender als der Begriff „Menge“ verwendet.\
                    Klassen unterliegen keinen Einschränkungen in ihrer \
                    Bildung oder Definition. Sie dürfen aber oft nur \
                    eingeschränkt verwendet werden, damit nicht die \
                    Widersprüche der naiven Mengenlehre entstehen. Zum \
                    Beispiel darf nicht jede Klasse Element von Mengen sein. \
                    Nur ein unsachgemäßer Umgang mit Klassen ist daher \
                    problematisch und erzeugt Widersprüche. Mit diesen drei \
                    Prinzipien können umständliche Formeln der \
                    prädikatenlogischen ZF-Sprache in bequeme und \
                    verständlichere Formeln mit Klassen übersetzt werden. Sie \
                    können als Zusatzaxiome für sogenannte virtuelle Klassen \
                    (s. u.) aufgefasst werden. Sie gelten auch bei der \
                    Verwendung von Klassentermen (s. u.) im Rahmen einer \
                    Klassenlogik; dort besagt aber ein Klassenterm gar nichts \
                    über die Existenz einer Klasse! Die Klassenlogik ist daher \
                    nur ein syntaktisch reichhaltiger logischer Rahmen, der \
                    eine bequemere optimierte Darstellung erlaubt und es \
                    gestattet, beliebige Klassen ohne die Gefahr eines \
                    Widerspruchs in jeden Kontext einzusetzen. \
                    Klassenvariablen sind hier freie Variablen; in gebundene \
                    Variablen können dagegen nur Elemente eingesetzt werden, \
                    speziell auch alle Mengen, die das Kriterium im \
                    Komprehensionsprinzip erfüllen müssen.".to_string();
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
