use command::{self, Command};
use config::Config;
use editor::{create_editor, EditorHelper};

use std::env;
use std::error::Error;
use std::fmt;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use term::{self, Terminal};

use rustyline::error::ReadlineError;
use rustyline::Editor;

/// Fallback textual prompt if term formatting fails.
const SAFE_PROMPT: &str = "carapace % ";

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt<'c> {
    config: &'c Config,

    /// Readline interface.
    pub editor: Editor<EditorHelper>,
}

impl<'c> Prompt<'c> {
    pub fn new(config: &'c Config) -> Prompt {
        let mut p = Prompt {
            config,
            editor: create_editor(config),
        };
        p.load_history();
        p
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn parse_command(&mut self) -> Result<Box<dyn Command>, Box<dyn Error>> {
        let prompt_txt = self.prompt();

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

    /// Yields the textual prompt with term colors.
    fn prompt(&self) -> String {
        // In case of failure, use safe prompt. It is a closure so it is only allocated if it is
        // needed.
        let safe_prompt = || SAFE_PROMPT.to_string();

        // In-memory cursor using a vector for data.
        let cursor = Cursor::new(Vec::new());

        let t = term::TerminfoTerminal::new(cursor);
        if t.is_none() {
            return safe_prompt();
        }
        let mut t = t.unwrap();

        // Create textual prompt.
        if t.fg(term::color::GREEN).is_err() {
            return safe_prompt();
        }
        write!(t, "carapace");

        if let Ok(cwd) = env::current_dir() {
            if t.fg(term::color::BRIGHT_BLUE).is_err() {
                return safe_prompt();
            }
            write!(t, " {}", cwd.display());
        }

        if t.fg(term::color::GREEN).is_err() {
            return safe_prompt();
        }
        write!(t, " % ");

        // NOTE: Resetting yields extra space at end with is annoying, so hardcoding to white color
        // at end.
        // t.reset().unwrap();
        if t.fg(term::color::WHITE).is_err() {
            return safe_prompt();
        }

        // Get inner cursor (it was moved above) and read what was written to its data vector.
        let mut inner_cursor = t.into_inner();
        if inner_cursor.seek(SeekFrom::Start(0)).is_err() {
            return safe_prompt();
        }

        let mut out = Vec::new();
        if inner_cursor.read_to_end(&mut out).is_err() {
            return safe_prompt();
        }

        String::from_utf8_lossy(&out).into_owned()
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

impl<'c> Drop for Prompt<'c> {
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
