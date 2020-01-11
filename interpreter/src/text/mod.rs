use super::parse::*;
mod text;
pub use text::*;
mod word;
pub use word::*;
mod punctuation;
pub use punctuation::*;
mod textelement;
pub use textelement::*;

use std::fmt::{Debug, Display, self};
struct ElementContext {
    before: Text,
    after: Text,
}
impl From<(Text, Text)> for ElementContext {
    fn from(ctx: (Text, Text)) -> Self {
        Self {
            before: ctx.0,
            after: ctx.1,
        }
    }
}
impl Debug for ElementContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} _ {}", self.before, self.after)
    }
}
impl Display for ElementContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
fn context_patterns(e: TextElement, pattern: Text) -> Vec<ElementContext> {
    let mut parts = Vec::new();
    let mut buf: Text = Text::new();
    for elem in pattern {
        if elem == e {
            parts.push(buf.clone());
            buf.clear();
        } else {
            buf.push(elem);
        }
    }
    parts.push(buf);
    let mut res = Vec::new();
    for window in (&parts).windows(2) {
        res.push((window[0].clone(), window[1].clone()).into());
    }
    res
}
use std::collections::HashMap;
fn all_context_patterns(pattern: Text) -> HashMap<TextElement, Vec<ElementContext>> {
    let mut map = HashMap::new();
    for word in pattern.unique_elements() {
        map.insert(word.clone(), context_patterns(word, pattern.clone()));
    }
    map
}
mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn parse_textelement() {
        assert_eq!(TextElement::parse(",").unwrap().1, TextElement::Punctuation(Punctuation::Comma));
        assert_eq!(TextElement::parse(".").unwrap().1, TextElement::Punctuation(Punctuation::Dot));
        assert_eq!(TextElement::parse(":").unwrap().1, TextElement::Punctuation(Punctuation::Colon));
        assert_eq!(TextElement::parse(";").unwrap().1, TextElement::Punctuation(Punctuation::Semicolon));
        assert_eq!(TextElement::parse("\'").unwrap().1, TextElement::Punctuation(Punctuation::Quote));
        assert_eq!(TextElement::parse("\"").unwrap().1, TextElement::Punctuation(Punctuation::DoubleQuote));
        let words = vec![
            "Hello",
            "Hi",
            "yes",
            "aha",
            "Mathematik",
            "mathmatical"
        ];
        for word in words {
            assert_eq!(TextElement::parse(word).unwrap().1, TextElement::Word(Word::from(word)));
        }
    }
    #[test]
    fn parse_punctuation() {
        assert_eq!(Punctuation::parse(",").unwrap().1, Punctuation::Comma);
        assert_eq!(Punctuation::parse(".").unwrap().1, Punctuation::Dot);
        assert_eq!(Punctuation::parse(":").unwrap().1, Punctuation::Colon);
        assert_eq!(Punctuation::parse(";").unwrap().1, Punctuation::Semicolon);
        assert_eq!(Punctuation::parse("\'").unwrap().1, Punctuation::Quote);
        assert_eq!(Punctuation::parse("\"").unwrap().1, Punctuation::DoubleQuote);
    }
    #[test]
    fn parse_word() {
        //let words = vec![
        //    "hello",
        //    "Hello",
        //    "Hi",
        //    "yes",
        //    "aha",
        //    "Mathematik",
        //    "mathmatical",
        //    "erfuellen"
        //];
        //for word in words {
        //    assert_eq!(Word::parse(word).unwrap().1, Word::from(word));
        //}
        println!("{}", nom::AsChar::is_alpha('à'))
    }
    #[test]
    fn parse_text() {
        let text = "Als Klasse gilt in der Mathematik, Klassenlogik und \
                    Mengenlehre eine Zusammenfassung beliebiger Objekte, \
                    definiert durch eine logische Eigenschaft, die alle \
                    Objekte der Klasse erfuellen.";
        assert_eq!(Text::parse(text).unwrap().1,
                   Text::from(vec![
                           TextElement::Word(Word::from("Als")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("gilt")),
                           TextElement::Word(Word::from("in")),
                           TextElement::Word(Word::from("der")),
                           TextElement::Word(Word::from("Mathematik")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("Klassenlogik")),
                           TextElement::Word(Word::from("und")),
                           TextElement::Word(Word::from("Mengenlehre")),
                           TextElement::Word(Word::from("eine")),
                           TextElement::Word(Word::from("Zusammenfassung")),
                           TextElement::Word(Word::from("beliebiger")),
                           TextElement::Word(Word::from("Objekte")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("definiert")),
                           TextElement::Word(Word::from("durch")),
                           TextElement::Word(Word::from("eine")),
                           TextElement::Word(Word::from("logische")),
                           TextElement::Word(Word::from("Eigenschaft")),
                           TextElement::Punctuation(Punctuation::Comma),
                           TextElement::Word(Word::from("die")),
                           TextElement::Word(Word::from("alle")),
                           TextElement::Word(Word::from("Objekte")),
                           TextElement::Word(Word::from("der")),
                           TextElement::Word(Word::from("Klasse")),
                           TextElement::Word(Word::from("erfuellen")),
                           TextElement::Punctuation(Punctuation::Dot)
                               ]
                   ))
    }
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
                    Klassenschreibweise notiert.
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
                    über die Existenz einer Klasse! Die Klassenlogik ist daher\
                     nur ein syntaktisch reichhaltiger logischer Rahmen, der \
                     eine bequemere optimierte Darstellung erlaubt und es \
                     gestattet, beliebige Klassen ohne die Gefahr eines \
                     Widerspruchs in jeden Kontext einzusetzen. \
                     Klassenvariablen sind hier freie Variablen; in gebundene \
                     Variablen können dagegen nur Elemente eingesetzt werden, \
                     speziell auch alle Mengen, die das Kriterium im \
                     Komprehensionsprinzip erfüllen müssen.";
        let text: Text = Text::parse(text).unwrap().1;
        //let mut occurrences = text.element_occurrences().into_iter().collect::<Vec<_>>();
        //occurrences.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        //let occurrences: Vec<String> =
        //    occurrences.into_iter()
        //               .map(|(k, v)| k.to_string() + ": " + &v.to_string())
        //               .collect();
        println!("{:#?}", text);


        //let pattern_map = all_context_patterns(text.clone());
        //println!("{:#?}", pattern_map);
    }
}
