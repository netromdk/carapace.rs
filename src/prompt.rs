use command::{self, Command};

use std::env;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use term;

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt;

impl Prompt {
    pub fn new() -> Prompt {
        Prompt
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

    /// Shows prompt and reads command and arguments from stdin.
    pub fn parse_command(&self) -> Result<Box<dyn Command>, Box<dyn Error>> {
        self.show()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let values: Vec<&str> = input.trim().split_whitespace().collect();

                if values.len() == 0 {
                    return Err(Box::new(NoCommandError));
                }

                command::parse_command(
                    values[0].to_string(),
                    values[1..].iter().map(|x| x.to_string()).collect(),
                )
            }
            Err(error) => {
                println!("Error: {}", error);
                Err(Box::new(error))
            }
        }
    }
}

#[derive(Debug)]
struct NoCommandError;

impl Error for NoCommandError {}

impl fmt::Display for NoCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No command inputted.")
    }
}
