use command::{self, Command};

use std::env;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use term;

use rustyline::error::ReadlineError;
use rustyline::Editor;

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt {
    /// Readline interface.
    pub editor: Editor<()>,
}

impl Prompt {
    pub fn new() -> Prompt {
        let mut p = Prompt {
            editor: Editor::<()>::new(),
        };
        p.load_history();
        p
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn parse_command(&mut self) -> Result<Box<dyn Command>, Box<dyn Error>> {
        let mut prompt_txt = String::from("[carapace] ");
        if let Ok(cwd) = env::current_dir() {
            prompt_txt = format!("{} {}", prompt_txt, cwd.display());
        }
        prompt_txt += " % ";

        let input = self.editor.readline(prompt_txt.as_ref());
        match input {
            Ok(line) => {
                self.editor.add_history_entry(line.as_ref());

                let input = line.trim();
                if input.len() == 0 {
                    return Err(Box::new(NoCommandError));
                }

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
            Err(ReadlineError::Interrupted) => {
                // TODO: Unhandled for now!
                println!("^C");
                Err(Box::new(NoCommandError))
            }
            Err(ReadlineError::Eof) => Err(Box::new(EofError)),
            Err(err) => {
                println!("Error: {:?}", err);
                Err(Box::new(err))
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

    fn load_history(&mut self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if let Err(_) = self.editor.load_history(&path) {
            println!("No history loaded.");
        }
    }

    fn save_history(&self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if let Err(err) = self.editor.save_history(&path) {
            println!("Could not save history to: {}\n{}", path.display(), err);
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
