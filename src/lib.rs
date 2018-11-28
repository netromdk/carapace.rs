//! Carapace is a general-purpose shell implementation done purely in Rust.

extern crate dirs;

mod command;
mod prompt;

use prompt::Prompt;

use std::env;
use std::path::Path;
use std::process;

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

            "cd" => {
                let home_dir = dirs::home_dir().unwrap_or_default();
                let path = if cmd.args.len() > 0 {
                    Path::new(&cmd.args[0])
                } else {
                    home_dir.as_path()
                };
                if let Err(err) = env::set_current_dir(path) {
                    println!("Could not change to {}: {}", path.display(), err);
                }
            }

            _ => {
                // Run command with arguments and wait for it to finish.
                let output = process::Command::new(cmd.name).args(cmd.args).output();
                match output {
                    Ok(output) => print!("{}", String::from_utf8_lossy(&output.stdout)),
                    Err(err) => println!("{}", err),
                };
            }
        }
    }
}
