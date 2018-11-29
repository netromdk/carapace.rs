//! Carapace is a general-purpose shell implementation done purely in Rust.

extern crate dirs;
extern crate term;

mod command;
mod prompt;

use prompt::Prompt;

/// Starts the read-eval-print-loop of the Carapace shell.
pub fn repl() {
    let prompt = Prompt::new();

    loop {
        let cmd = prompt.parse_command();
        if let Err(_) = cmd {
            continue;
        }

        match cmd.unwrap().execute() {
            Ok(ret) => println!("ret: {}", ret),
            Err(exit_code) => {
                println!("exit with {}", exit_code);
                break;
            }
        }
    }
}
