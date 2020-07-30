use std::io::{
    self,
    Write,
    stdin,
    Stdout,
    stdout,
    BufRead,
};
use seqraph::{
    SequenceGraph,
    mapping::Sequenced,
};

pub struct Shell {
    prompt: String,
    stdout: Stdout,
    exit: bool,
    graph: SequenceGraph<char>,
}

#[derive(Clone)]
enum Command {
    Exit,
    Help,
    Text(String),
}
#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* $(,)? ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = std::collections::HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
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
            stdout: stdout(),
            exit: false,
            graph: SequenceGraph::new()
        }
    }
    pub fn run(&mut self) -> io::Result<()> {
        self.write_help()?;
        loop {
            self.stdout.flush()?;
            if self.exit { break }
            self.write_prompt()?;
            if let Some(line) = stdin().lock().lines().next() {
                let line = line?;
                if !line.is_empty() {
                    let cmd = self.read_command(&line)?;
                    self.exec_command(cmd)?
                }
            }
            self.stdout.flush()?;
        }
        Ok(())
    }
    fn read_command(&self, line: &str) -> io::Result<Command> {
        for (ts, cmd, _desc) in COMMANDS.iter() {
            if ts.iter().any(|x| x == &line) {
                return Ok(cmd.clone());
            }
        }
        Ok(Command::Text(line.to_string()))
    }
    fn exec_command(&mut self, cmd: Command) -> io::Result<()> {
        Ok(match cmd {
            Command::Help => self.write_help()?,
            Command::Exit => { self.exit = true; },
            Command::Text(s) => {
                self.graph.read_sequence(s.chars());
                let info = self.graph.get_node_info(&s.chars().next().unwrap());
                println!("{:#?}", info);
            },
        })
    }
    pub fn set_prompt<S: Into<String>>(&mut self, p: S) {
        self.prompt = p.into();
    }
    fn write_prompt(&mut self) -> io::Result<()> {
        write!(self.stdout, "{}", self.prompt)?;
        self.stdout.flush()
    }
    fn write_help(&mut self) -> io::Result<()> {
        write!(self.stdout, "Natural language interpreter\n")?;
        let mut lines = Vec::new();
        let mut max = 0;
        for (ts, _cmd, _desc) in COMMANDS.iter() {
            let mut strs =
                   ts.iter()
                     .fold(String::new(),
                     |acc, x| format!("{}{} | ", acc, x));
            strs.pop();
            strs.pop();
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
            write!(self.stdout, "{}\t{}\n",
                ts,
                desc
            )?;
        }
        self.stdout.flush()
    }
}
