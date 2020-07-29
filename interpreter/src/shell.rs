use std::io::{
    self,
    Write,
    stdin,
    Stdout,
    stdout,
    BufRead,
};

//#[derive(Clone)]
pub struct Shell {
    prompt: String,
    //stdout: RawTerminal<Stdout>,
    stdout: Stdout,
    history: Vec<String>,
    history_index: Option<usize>,
    exit: bool,
}

enum Command {
    Exit,
    Help,
    History,
    Text(String),
}

impl Shell {
    pub fn new() -> Self {
        Self {
            prompt: "> ".into(),
            stdout: stdout(),
            history: Vec::new(),
            history_index: None,
            exit: false,
        }
    }
    pub fn run(&mut self) -> io::Result<()> {
        self.write_help()?;
        loop {
            self.stdout.flush()?;
            if self.exit { break }
            self.write_prompt()?;
            //if let Some(key) = stdin().lock().keys().next() {
            //    let key = key?;
            //    match key {
            //        Key::Up | Key::Down => {
            //            match key {
            //                Key::Up => self.inc_history_index(),
            //                Key::Down => self.dec_history_index(),
            //                _ => {},
            //            }
            //        },
            //        _ => {},
            //    }
            //} else {
            if let Some(line) = stdin().lock().lines().next() {
                let line = line?;
                if !line.is_empty() {
                    let cmd = self.read_command(&line)?;
                    //self.append_history_unique(line.clone());
                    self.exec_command(cmd)?
                }
            }
            //}
            //if let Some(line) = self.get_history_index() {
            //    std::io::stdout().write(line.as_bytes()).unwrap();
            //}
            self.stdout.flush()?;
        }
        Ok(())
    }
    fn read_command(&self, line: &str) -> io::Result<Command> {
        Ok(match line  {
            "q" | "quit" | "exit" | ":q" => Command::Exit,
            "h" | "help" | "?" => Command::Help,
            "history" => Command::History,
            line => Command::Text(line.to_string())
        })
    }
    fn exec_command(&mut self, cmd: Command) -> io::Result<()> {
        Ok(match cmd {
            Command::Help => self.write_help()?,
            Command::History => self.write_history()?,
            Command::Exit => { self.exit = true; },
            Command::Text(s) => self.write_line(&s)?,
        })
    }
    pub fn append_history_unique<S: Into<String>>(&mut self, s: S) {
        let s = s.into();
        if !self.history.contains(&s) {
            self.append_history(s);
        }
    }
    pub fn append_history<S: Into<String>>(&mut self, s: S) {
        self.history.push(s.into());
    }
    pub fn reset_history_index(&mut self) {
        self.history_index = None;
    }
    pub fn inc_history_index(&mut self) {
        self.history_index = self.history_index.map_or(
            Some(0),
            |i| Some(i + 1),
        );
    }
    pub fn dec_history_index(&mut self) {
        self.history_index = self.history_index.map_or(
            Some(0),
            |i| Some(i - 1),
        );
    }
    pub fn get_history_index(&self) -> Option<String> {
        self.history_index
            .and_then(|i| self.history.get(i).map(Clone::clone))
    }
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }
    pub fn set_prompt<S: Into<String>>(&mut self, p: S) {
        self.prompt = p.into();
    }
    fn write_line(&mut self, line: &str) -> io::Result<()> {
        write!(self.stdout, "{}", line)?;
        self.stdout.flush()
    }
    fn write_prompt(&mut self) -> io::Result<()> {
        write!(self.stdout, "{}", self.prompt)?;
        self.stdout.flush()
    }
    fn write_help(&mut self) -> io::Result<()> {
        write!(self.stdout, "Natural language interpreter\n
q[uit] | exit | :q\tQuit interpreter.
h[elp] | ?\t\tShow help.\n\n")?;
        self.stdout.flush()
    }
    fn write_history(&mut self) -> io::Result<()> {
        write!(self.stdout, "{:?}", self.history)?;
        self.stdout.flush()
    }
}
