//! Carapace is a general-purpose shell implementation done purely in Rust.
//!
//! # Configuration
//!
//! The configuration file resides at "~/.carapace/config.json". It will be created when first
//! running carapace.
//!
//! An example config:
//! ```json
//! {
//!   "max_history_size": 1000,
//!   "edit_mode": "emacs",
//!   "completion_type": "list",
//!   "aliases": {
//!     "l": "ls",
//!     "ll": "ls -l"
//!   }
//! }
//! ```
//!
//! ## Options
//!
//! - `max_history_size` takes a positive number as the maximum of entries to keep in history (at
//! "~/.carapace/history").
//! - `edit_mode` gives either `"emacs"` or `"vi"` bindings.
//! - `completion_type` can either give a `"list"` of all possibilities, like Bash, or provide a
//! `"circular"` completion of each candidate, like VI.
//! - `aliases` is a "map" of (alias, command replacement) pairs, like `"ll": "ls -l"`.

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
