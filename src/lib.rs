//! Carapace is a general-purpose shell implementation done purely in Rust.

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
        let cmd = cmd.unwrap();

        match cmd.name.as_ref() {
            "exit" | "quit" => break,

            cmd @ _ => {
                println!("Unknown command: {}", cmd);
                continue;
            }
        }
    }
}
