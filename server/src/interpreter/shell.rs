use std::sync::{Arc, Mutex};
use termion::{
    *,
    AsyncReader,
    input::{
        TermRead,
        Keys,
    },
    event::{
        Key,
    },
    raw::{
        RawTerminal,
        IntoRawMode,
    },
};
use std::io::{
    self,
    Write,
    stdin,
    Stdin,
    StdinLock,
    Stdout,
    stdout,
    Error,
    ErrorKind,
    BufRead,
};

//#[derive(Clone)]
pub struct Shell {
    prompt: String,
    //stdout: RawTerminal<Stdout>,
    stdout: Stdout,
    stdin: Stdin,
    history: Vec<String>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            prompt: "".into(),
            stdin: stdin(),
            //stdout: stdout().into_raw_mode().unwrap(),
            stdout: stdout(),
            history: Vec::new(),
        }
    }
    pub fn append_history_unique<S: Into<String>>(&mut self, s: S) {
        let s = s.into();
        if !self.history.contains(&s) {
            self.append_history(s);
        }
    }
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }
    pub fn append_history<S: Into<String>>(&mut self, s: S) {
        self.history.push(s.into());
    }
    pub fn set_prompt<S: Into<String>>(&mut self, p: S) {
        self.prompt = p.into();
    }
    fn write_prompt(&mut self) {
        write!(self.stdout, "{}", self.prompt).unwrap();
        self.stdout.flush().unwrap();
    }
    fn remove_newline(s: &mut String) {
    }
    pub fn read_line(&mut self) -> io::Result<String> {
        self.write_prompt();
        Ok(self.lines().next().unwrap().unwrap())
    }
    pub fn lines(&mut self) -> std::io::Lines<StdinLock> {
        self.stdin.lock().lines()
    }
    pub fn keys(&mut self) -> Keys<StdinLock> {
        self.stdin.lock().keys()
    }
    pub fn read_key(&mut self) -> io::Result<Key> {
        self.write_prompt();
        Ok(self.keys().next().unwrap().unwrap())
    }
}
