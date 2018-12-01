use command::{self, Command};

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Write};

use term;

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt {
    /// History of inputs.
    pub history: Vec<String>,

    /// Maximum of newest entries to keep in history.
    history_max: usize,
}

impl Prompt {
    pub fn new() -> Prompt {
        let mut p = Prompt {
            history: vec![],
            history_max: 1000,
        };
        p.load_history();
        p
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn parse_command(&mut self) -> Result<Box<dyn Command>, Box<dyn Error>> {
        self.show()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Catch ^D/EOF.
                if input.len() == 0 {
                    return Err(Box::new(EofError));
                }

                let input = input.trim();
                if input.len() == 0 {
                    return Err(Box::new(NoCommandError));
                }

                self.push_to_history(input.to_string());

                // Split on whitespace, convert to String parts, and replace all ~ with home dir
                // (for parts starting with it only).
                let home_dir = dirs::home_dir().unwrap_or_default();
                let mut values: Vec<String> = input
                    .split_whitespace()
                    .map(|x| {
                        let mut s = x.to_string();
                        if !s.starts_with("~") {
                            s
                        } else {
                            let cnt = if s.starts_with("~/") { 2 } else { 1 };
                            let rest: String = s.drain(cnt..).collect();
                            if let Ok(res) = home_dir.join(&rest).into_os_string().into_string() {
                                res
                            } else {
                                s
                            }
                        }
                    }).collect();

                let program = values[0].clone();
                let args = values.drain(1..).collect();
                command::parse_command(program, args)
            }
            Err(error) => {
                println!("Error: {}", error);
                Err(Box::new(error))
            }
        }
    }

    /// Shows prompt.
    fn show(&self) -> Result<(), Box<dyn Error>> {
        let mut t = term::stdout().unwrap();

        t.fg(term::color::GREEN).unwrap();
        write!(t, "carapace");

        if let Ok(cwd) = env::current_dir() {
            t.fg(term::color::BRIGHT_BLUE).unwrap();
            write!(t, " {}", cwd.display());
        }

        t.fg(term::color::GREEN).unwrap();
        write!(t, "> ");

        t.reset().unwrap();
        io::stdout().flush()?;
        Ok(())
    }

    fn push_to_history(&mut self, input: String) {
        self.history.push(input);
        if self.history.len() > self.history_max {
            self.history.remove(0);
        }
    }

    fn load_history(&mut self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        match fs::read(&path) {
            Ok(contents) => {
                self.history = String::from_utf8_lossy(&contents)
                    .split('\n')
                    .map(|x| x.to_string())
                    .collect();
            }
            Err(err) => println!("Could not load history from: {}\n{}", path.display(), err),
        }
    }

    fn save_history(&self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if let Err(err) = fs::write(&path, self.history.join("\n")) {
            println!("Could not write history to: {}\n{}", path.display(), err);
        }
    }
}

impl Drop for Prompt {
    fn drop(&mut self) {
        self.save_history();
    }
}

#[derive(Debug)]
struct EofError;

impl Error for EofError {}

impl fmt::Display for EofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write newline on ^D/EOF so next prompt doesn't appear on same line.
        writeln!(f, "")
    }
}

#[derive(Debug)]
struct NoCommandError;

impl Error for NoCommandError {}

impl fmt::Display for NoCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
