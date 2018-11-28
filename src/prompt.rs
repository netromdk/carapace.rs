use command::Command;

use std::error::Error;
use std::io::{self, Write};

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt;

impl Prompt {
    pub fn new() -> Prompt {
        Prompt
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn parse_command(&self) -> Result<Command, Box<dyn Error>> {
        print!("carapace> ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let values: Vec<&str> = input.trim().split_whitespace().collect();
                Command::new(values)
            }
            Err(error) => {
                println!("Error: {}", error);
                Err(Box::new(error))
            }
        }
    }
}
