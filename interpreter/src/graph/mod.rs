pub mod edge;
pub use crate::graph::edge::*;
pub mod edges;
pub use crate::graph::edges::*;
pub mod node;
pub use crate::graph::node::*;
pub mod nodes;
pub use crate::graph::nodes::*;
pub mod path;
pub use crate::graph::path::*;
pub mod edge_mapping;
pub use crate::graph::edge_mapping::*;

use crate::text::*;
use crate::sentence::*;

use petgraph::{
    Direction,
    graph::{
        DiGraph,
        EdgeIndex,
        NodeIndex,
        EdgeReference,
    },
    dot::{
        Dot,
    },
    visit::{
        EdgeRef
    },
};
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;


type InternalTextGraph = DiGraph<TextGraphNodeWeight, TextGraphEdgeWeight>;
#[derive(Debug)]
pub struct TextGraph {
    graph: InternalTextGraph,
}
impl std::ops::Deref for TextGraph {
    type Target = InternalTextGraph;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl std::ops::DerefMut for TextGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}
impl Into<InternalTextGraph> for TextGraph {
    fn into(self) -> InternalTextGraph {
        self.graph
    }
}
impl<'a> TextGraph {
    pub fn new() -> Self {
        let mut n = Self {
            graph: DiGraph::new(),
        };
        // TODO should use enum_iterator with is_stop()
        // All stop symbols could be followed by empty
        //n.add_edge(
        //    &TextElement::Punctuation(Punctuation::Dot),
        //    &TextElement::Empty,
        //    1
        //);
        n
    }
    pub fn read_text(&'a mut self, text: Text) {
        let mut text = text;
        let len = text.len();
        let sentences = text.to_sentences();
        //println!("sentences = {:#?}", sentences);
        for sentence in sentences {
            self.read_sentence(sentence);
        }
    }
    pub fn read_sentence(&'a mut self, sentence: Text) {
        let mut sentence = sentence;
        sentence.push_front(TextElement::Empty);
        sentence.push(TextElement::Empty);
        let len = sentence.len();
        //println!("reading: {}", sentence);
        for word_index in 1..(len-1) {
            // i starts at 1 because it should always be between two other
            // elements
            //println!("word = {}, len = {}", word_index, len);
            self.read_word_context(&sentence, word_index);
        }
    }
    pub fn read_word_context(&'a mut self, text: &Text, word_index: usize) {
        let word = &text[word_index];
        let limit = text.len()-1;
        for pre in 0..word_index {
            // for all predecessors of word
            let left = &text[pre];
            //println!("pre = {}, word_index = {}", pre, word_index);
            let left_distance = word_index-pre;
            // for all successors of word
            for succ in (word_index+1)..=limit {
                //println!("pre = {}, succ = {}", pre, succ);
                let right = &text[succ];
                let right_distance = succ-word_index;
                self.insert_sequence(left, left_distance, word, right_distance, right);
            }
        }
    }
    pub fn insert_sequence(&'a mut self,
        left: &TextElement,
        left_distance: usize,
        word: &TextElement,
        right_distance: usize,
        right: &TextElement) {
        //println!("Inserting \"{}\" \"{}\" \"{}\"", left, word, right);
        let li = self.add_node(left);
        let wordi = self.add_node(word);
        let ri = self.add_node(right);
        let left_edge = self.add_edge(li, wordi, left_distance);
        let right_edge = self.add_edge(wordi, ri, right_distance);
        self.node_weight_mut(wordi)
            .unwrap()
            .add_transition(left_edge, right_edge);
    }
    pub fn add_edge_for_elements(&'a mut self, left: &TextElement, right: &TextElement, distance: usize) -> EdgeIndex {
        //println!("inserting \"{}\" and \"{}\"", left, right);
        let li = self.add_node(left);
        let ri = self.add_node(right);
        self.add_edge(li, ri, distance)
    }
    pub fn add_edge(&'a mut self, left: NodeIndex, right: NodeIndex, distance: usize) -> EdgeIndex {
        let l = self.get_node(left);
        let r = self.get_node(right);
        let edge = self.find_edge_index(&l, &r, distance);
        let edge_index = if let Some(i) = edge {
            i
        } else {
            self.graph.add_edge(left, right, TextGraphEdgeWeight::new(distance))
        };
        edge_index
    }
    fn find_node_index(&'a self, element: &TextElement) -> Option<NodeIndex> {
        self.graph.node_indices()
            .find(|i| self.graph[*i].element() == element)
            .map(|i| i.clone())
    }
    fn find_edge_index(&'a self, a: &GraphNode<'a>, b: &GraphNode<'a>, distance: usize) -> Option<EdgeIndex> {
        self.graph
            .edges_connecting(a.index(), b.index())
            .find(|e| *e.weight() == distance)
            .map(|e| e.id())
    }
    pub fn find_node(&'a self, element: &TextElement) -> Option<GraphNode<'a>> {
        self.find_node_index(element).map(|n| self.get_node(n))
    }
    pub fn find_edge(&'a self, a: &GraphNode<'a>, b: &GraphNode<'a>, distance: usize) -> Option<GraphEdge<'a>> {
        self.find_edge_index(a, b, distance)
            .map(|i| GraphEdge::new(self, i))
    }
    pub fn add_node(&'a mut self, element: &TextElement) -> NodeIndex {
        if let Some(i) = self.find_node_index(element) {
            i
        } else {
            self.graph.add_node(
                TextGraphNodeWeight::new(element.clone())
            )
        }
    }
    pub fn contains(&self, element: &TextElement) -> bool {
        self.find_node(element).is_some()
    }
    pub fn get_edges_directed(&'a self, index: NodeIndex, d: Direction) -> GraphEdges<'a> {
        GraphEdges::new(
            self.graph
                .edges_directed(index, d)
                .map(|e| self.get_edge(e.id()))
                .collect::<HashSet<_>>().iter().cloned()
        )
    }
    pub fn get_edges_incoming(&'a self, index: NodeIndex) -> GraphEdges<'a> {
        self.get_edges_directed(index, Direction::Incoming)
    }
    pub fn get_edges_outgoing(&'a self, index: NodeIndex) -> GraphEdges<'a> {
        self.get_edges_directed(index, Direction::Outgoing)
    }
    pub fn get_edge(&'a self, index: EdgeIndex) -> GraphEdge<'a> {
        GraphEdge::new(
            self,
            index
        )
    }
    pub fn get_edges(&'a self, index: NodeIndex) -> GraphEdges<'a> {
        let edges = self.get_edges_incoming(index).into_iter()
            .chain(self.get_edges_outgoing(index));
        GraphEdges::new(edges)
    }
    pub fn get_node(&'a self, index: NodeIndex) -> GraphNode<'a> {
        GraphNode::new(
            self,
            index
        )
    }
    pub fn get_text_path(&'a self, nodes: Vec<GraphNode<'a>>) -> Option<TextPath<'a>> {
        TextPath::from_nodes(nodes)
    }
    pub fn find_text_path(&'a self, nodes: Vec<TextElement>) -> Option<TextPath<'a>> {
        TextPath::from_elements(self, nodes)
    }

    pub fn element_info(&self, element: &TextElement) {
        match self.find_node(element) {
            Some(n) => println!("{}", n),
            None => {}
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

pub(crate) mod tests {
    #![allow(non_upper_case_globals)]
    use super::*;
    use crate::text::*;
    use crate::parse::*;
    lazy_static! {
        pub static ref gehen_text: Text = Text::parse(
        "Ich gehe.
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
        ").unwrap().1;

        pub static ref dornroeschen_text: Text = Text::parse("
Vor Zeiten war ein König und eine Königin, die sprachen jeden Tag 'ach, wenn wir doch ein Kind hätten!'
und kriegten immer keins. Da trug sich zu, als die Königin einmal im Bade saß, daß ein Frosch aus dem
Wasser ans Land kroch und zu ihr sprach, 'dein Wunsch wird erfüllt werden, ehe ein Jahr vergeht, wirst
du eine Tochter zur Welt bringen.' Was der Frosch gesagt hatte, das geschah, und die Königin gebar ein
Mädchen, das war so schön, daß der König vor Freude sich nicht zu lassen wußte und ein großes Fest
anstellte. Er ladete nicht blos seine Verwandte, Freunde und Bekannte, sondern auch die weisen Frauen
dazu ein, damit sie dem Kind hold und gewogen wären. Es waren ihrer dreizehn in seinem Reiche, weil er
aber nur zwölf goldene Teller hatte, von welchen sie essen sollten, so mußte eine von ihnen daheim
bleiben. Das Fest ward mit aller Pracht gefeiert, und als es zu Ende war, beschenkten die weisen Frauen
das Kind mit ihren Wundergaben: die eine mit Tugend, die andere mit Schönheit, die dritte mit Reichthum,
und so mit allem, was auf der Welt zu wünschen ist. Als elfe ihre Sprüche eben gethan hatten, trat
plötzlich die dreizehnte herein. Sie wollte sich dafür rächen daß sie nicht eingeladen war, und ohne
jemand zu grüßen oder nur anzusehen, rief sie mit lauter Stimme 'die Königstochter soll sich in ihrem
fünfzehnten Jahr an einer Spindel stechen und todt hinfallen.' Und ohne ein Wort weiter zu sprechen kehrte
sie sich um und verließ den Saal. Alle waren erschrocken, da trat die zwölfte hervor, die ihren Wunsch
noch übrig hatte und weil sie den bösen Spruch nicht aufheben, sondern nur ihn mildern konnte, so sagte
sie 'es soll aber kein Tod sein, sondern ein hundertjähriger tiefer Schlaf, in welchen die Königstochter
fällt.' Der König, der sein liebes Kind vor dem Unglück gern bewahren wollte, ließ den Befehl ausgehen,
daß alle Spindeln im ganzen Königreiche sollten verbrannt werden. An dem Mädchen aber wurden die Gaben der
weisen Frauen sämmtlich erfüllt, denn es war so schön, sittsam, freundlich und verständig, daß es
jedermann, der es ansah, lieb haben mußte. Es geschah, daß an dem Tage, wo es gerade fünfzehn Jahr alt
ward, der König und die Königin nicht zu Haus waren, und das Mädchen ganz allein im Schloß zurückblieb.
Da gieng es aller Orten herum, besah Stuben und Kammern, wie es Lust hatte, und kam endlich auch an einen
alten Thurm. Es stieg die enge Wendeltreppe hinauf, und gelangte zu einer kleinen Thüre. In dem Schloß
steckte ein verrosteter Schlüssel, und als es umdrehte, sprang die Thüre auf, und saß da in einem kleinen
Stübchen eine alte Frau mit einer Spindel und spann emsig ihren Flachs. 'Guten Tag, du altes Mütterchen,'
sprach die Königstochter, 'was machst du da?' 'Ich spinne,' sagte die Alte und nickte mit dem Kopf. 'Was
ist das für ein Ding, das so lustig herumspringt?' sprach das Mädchen, nahm die Spindel und wollte auch
spinnen. Kaum hatte sie aber die Spindel angerührt, so gieng der Zauberspruch in Erfüllung, und sie stach
sich damit, in den Finger.  In dem Augenblick aber, wo sie den Stich empfand, fiel sie auf das Bett
nieder, das da stand, und lag in einem tiefen Schlaf. Und dieser Schlaf verbreitete sich über das ganze
Schloß: der König und die Königin, die eben heim gekommen waren und in den Saal getreten waren, fiengen
an einzuschlafen, und der ganze Hofstaat mit ihnen. Da schliefen auch die Pferde m Stall, die Hunde im
Hofe, die Tauben auf dem Dache, die Fliegen an der Wand, ja, das Feuer, das auf dem Herde flackerte, ward
still und schlief ein, und der Braten hörte auf zu brutzeln, und der Koch, der den Küchenjungen, weil er
etwas versehen hatte, in den Haaren ziehen wollte, ließ ihn los und schlief. Und der Wind legte sich, und
auf den Bäumen vor dem Schloß regte sich kein Blättchen mehr.
Rings um das Schloß aber begann eine Dornenhecke zu wachsen, die jedes Jahr höher ward, und endlich das
ganze Schloß umzog, und darüber hinaus wuchs, daß gar nichts mehr davon zu sehen war, selbst nicht die
Fahne auf dem Dach. Es gieng aber die Sage in dem Land von dem schönen schlafenden Dornröschen, denn so
ward die Königstochter genannt, also daß von Zeit zu Zeit Königssöhne kamen und durch die Hecke in das
Schloß dringen wollten. Es war ihnen aber nicht möglich, denn die Dornen, als hätten sie Hände, hielten
fest zusammen, und die Jünglinge blieben darin hängen, konnten sich nicht wieder los machen und starben
eines jämmerlichen Todes. Nach langen langen Jahren kam wieder ein mal ein Königssohn in das Land, und
hörte wie ein alter Mann von der Dornhecke erzählte, es sollte ein Schloß dahinter stehen, in welchem eine
wunderschöne Königstochter, Dornröschen genannt, schon seit hundert Jahren schliefe, und mit ihr schliefe
der König und die Königin und der ganze Hofstaat. Er wußte auch von seinem Großvater daß schon viele
Königssöhne gekommen wären und versucht hätten durch die Dornenhecke zu dringen, aber sie wären darin
hängen geblieben und eines traurigen Todes gestorben. Da sprach der Jüngling 'ich fürchte mich nicht, ich
will hinaus und das schöne Dornröschen sehen.' Der gute Alte mochte ihm abrathen, wie er wollte, er hörte
nicht auf seine Worte.
Nun waren aber gerade die hundert Jahre verflossen, und der Tag war gekommen, wo Dornröschen wieder
erwachen sollte. Als der Königssohn sich der Dornenhecke näherte, waren es lauter große schöne Blumen,
die thaten sich von selbst auseinander und ließen ihn unbeschädigt hindurch, und hinter ihm thaten sie
sich wieder als eine Hecke zusammen. Im Schloßhof sah er die Pferde und scheckigen Jagdhunde liegen und
schlafen, auf dem Dache saßen die Tauben und hatten das Köpfchen unter den Flügel gesteckt. Und als er
ins Haus kam, schliefen die Fliegen an der Wand, der Koch in der Küche hielt noch die Hand, als wollte er
den Jungen anpacken, und die Magd saß vor dem schwarzen Huhn, das sollte gerupft werden. Da gieng er
weiter, und sah im Saale den ganzen Hofstaat liegen und schlafen, und oben bei dem Throne lag der König
und die Königin. Da gieng er noch weiter, und alles war so still, daß einer seinen Atem hören konnte, und
endlich kam er zu dem Thurm und öffnete die Thüre zu der kleinen Stube, in welcher Dornröschen schlief. Da
lag es und war so schön, daß er die Augen nicht abwenden konnte, und er bückte sich und gab ihm einen Kuß.
Wie er es mit dem Kuß berührt hatte, schlug Dornröschen die Augen auf, erwachte, und blickte ihn ganz
freundlich an. Da giengen sie zusammen herab, und der König erwachte und die Königin, und der ganze
Hofstaat, und sahen einander mit großen Augen an. Und die Pferde im Hof standen auf und rüttelten sich:
die Jagdhunde sprangen und wedelten: die Tauben auf dem Dache zogen das Köpfchen unterm Flügel hervor,
sahen umher und flogen ins Feld: die Fliegen an den Wänden krochen weiter: das Feuer in der Küche erhob
sich, flackerte: und kochte das Essen: der Braten fieng wieder an zu brutzeln: und der Koch gab dem Jungen
eine Ohrfeige daß er schrie: und die Magd rupfte das Huhn fertig. Und da wurde die Hochzeit des
Königssohns mit dem Dornröschen in aller Pracht gefeiert, und sie lebten vergnügt bis an ihr Ende.
").unwrap().1;
    }
    #[test]
    fn read_text() {
        let mut tg = TextGraph::new();
        tg.read_text(gehen_text.clone());
        tg.write_to_file("graphs/gehen_graph");
    }
}
