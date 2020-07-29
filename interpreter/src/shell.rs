use std::io::{
    self,
    Write,
    stdin,
    Stdout,
    stdout,
    BufRead,
};

pub struct Shell {
    prompt: String,
    stdout: Stdout,
    exit: bool,
}

enum Command {
    Exit,
    Help,
    Text(String),
}

impl Shell {
    pub fn new() -> Self {
        Self {
            prompt: "> ".into(),
            stdout: stdout(),
            exit: false,
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
        Ok(match line  {
            "q" | "quit" | "exit" | ":q" => Command::Exit,
            "h" | "help" | "?" => Command::Help,
            line => Command::Text(line.to_string())
        })
    }
    fn exec_command(&mut self, cmd: Command) -> io::Result<()> {
        Ok(match cmd {
            Command::Help => self.write_help()?,
            Command::Exit => { self.exit = true; },
            Command::Text(s) => self.write_line(&s)?,
        })
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
}
