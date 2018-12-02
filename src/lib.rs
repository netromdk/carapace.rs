//! Carapace is a general-purpose shell implementation done purely in Rust.

extern crate dirs;
extern crate json;
extern crate rustyline;
extern crate term;

pub mod command;
pub mod config;
pub mod editor;
pub mod prompt;

use config::Config;
use prompt::Prompt;

use std::fs;
use std::process;

/// Starts the read-eval-print-loop of the Carapace shell.
pub fn repl() {
    // Create init folder if not present.
    let path = dirs::home_dir().unwrap().join(".carapace");
    if let Err(err) = fs::create_dir_all(&path) {
        println!("Could not create init folder: {}\n{}", path.display(), err);
        return;
    }

    let exit_code;

    {
        let config = Config::new();
        let mut prompt = Prompt::new(&config);

        loop {
            let cmd = prompt.parse_command();
            if let Err(err) = cmd {
                print!("{}", err);
                continue;
            }

            match cmd.unwrap().execute(&prompt) {
                Ok(_) => continue,
                Err(code) => {
                    exit_code = code;
                    break;
                }
            }
        }
    }

    // Exits process immediately so things must be cleaned up before here!
    process::exit(exit_code);
}
