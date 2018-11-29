//! Carapace is a general-purpose shell implementation done purely in Rust.

extern crate dirs;
extern crate term;

pub mod command;
pub mod prompt;

use prompt::Prompt;

/// Starts the read-eval-print-loop of the Carapace shell.
pub fn repl() {
    let prompt = Prompt::new();

    loop {
        let cmd = prompt.parse_command();
        if let Err(err) = cmd {
            println!("{}", err);
            continue;
        }

        match cmd.unwrap().execute() {
            Ok(_) => continue,
            Err(_exit_code) => {
                break;
            }
        }
    }
}
