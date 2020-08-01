use std::io::{
    self,
    Write,
    stdin,
    stdout,
    BufRead,
};
use seqraph::{
    SequenceGraph,
    mapping::{
        Symbol,
        EdgeMapping,
    },
    graph::{
        node::{
            NodeData,
        },
    },
};
use itertools::*;
use crate::{
    parse::{
        Parse,
    },
};
use regex_syntax::{
    ast::parse::Parser,
};

pub struct Shell {
    prompt: String,
    exit: bool,
    graph: SequenceGraph<char>,
}

#[derive(Clone)]
enum Command {
    Exit,
    Help,
    Learn(String),
    Match(String),
}

lazy_static!{
    static ref COMMANDS: Vec<(Vec<&'static str>, Command, &'static str)> = vec![
        (vec!["q", "quit", "exit", ":q"], Command::Exit, "Exit shell."),
        (vec!["h", "help", "?"], Command::Help, "Show help.")
    ];
}
impl Shell {
    pub fn new() -> Self {
        Self {
            prompt: "> ".into(),
            exit: false,
            graph: SequenceGraph::new()
        }
    }
    pub fn run(&mut self) -> io::Result<()> {
        self.print_help();
        loop {
            if self.exit { break }
            self.print_prompt()?;
            if let Some(line) = stdin().lock().lines().next() {
                let line = line?;
                if !line.is_empty() {
                    let cmd = self.read_command(&line)?;
                    self.exec_command(cmd)?
                }
            }
            stdout().flush()?;
        }
        Ok(())
    }
    fn read_command(&self, line: &str) -> io::Result<Command> {
        for (ts, cmd, _desc) in COMMANDS.iter() {
            if ts.iter().any(|x| x == &line) {
                return Ok(cmd.clone());
            }
        }
        if line.starts_with("match ") {
            Ok(Command::Match(line.strip_prefix("match ").unwrap().to_string()))
        } else {
            Ok(Command::Learn(line.to_string()))
        }
    }
    fn exec_command(&mut self, cmd: Command) -> io::Result<()> {
        Ok(match cmd {
            Command::Help => self.print_help(),
            Command::Exit => { self.exit = true; },
                // Different actions:
                // - learn new parser
                // - try to parse sequence
                //  - show errors
            Command::Learn(s) => {
                self.graph.learn_sequence(s.chars());
                let info = self.graph.get_node_info(&s.chars().next().unwrap());
                println!("{:#?}", info);
            },
            Command::Match(s) => {
                match Parser::new().parse(&s) {
                    Err(e) => { println!("{:#?}", e); },
                    Ok(re) => {
                        //let captures = re.captures_iter().collect::<Vec<_>>();
                        println!("{:#?}", re);
                    }
                }
            },
        })
    }
    pub fn set_prompt<S: Into<String>>(&mut self, p: S) {
        self.prompt = p.into();
    }
    fn print_prompt(&self) -> io::Result<()> {
        print!("{}", self.prompt);
        stdout().flush()
    }
    fn print_help(&mut self) {
        println!("Natural language interpreter");
        let mut lines = Vec::new();
        let mut max = 0;
        for (ts, _cmd, _desc) in COMMANDS.iter() {
            let strs = ts.iter().join(" | ");
            max = strs.len().max(max);
            lines.push(strs);
        }
        for line in &mut lines {
            let tab_width = 8;
            let d = max - line.len();
            let ts = (d as f32 / tab_width as f32).floor() as usize;
            line.push_str(&std::iter::repeat("\t").take(ts).collect::<String>());
        }
        for (ts, desc) in lines.iter().zip(COMMANDS.iter().map(|(_, _, desc)| desc)) {
            println!("{}\t{}",
                ts,
                desc
            );
        }
    }
}
