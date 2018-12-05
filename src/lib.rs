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
//!   "auto_cd": true,
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
//! - `auto_cd` enables implicit `cd` command usage by inputting existing folder paths.
//! - `aliases` is a "map" of (alias, command replacement) pairs, like `"ll": "ls -l"`.

extern crate dirs;
extern crate json;
extern crate regex;
extern crate rustyline;
extern crate term;

pub mod command;
pub mod config;
pub mod editor;
pub mod prompt;
pub mod util;

use config::Config;
use prompt::{EofError, Prompt};

use std::fs;

/// Starts the read-eval-print-loop of the Carapace shell.
/// Returns the exit code.
pub fn repl() -> i32 {
    // Create init folder if not present.
    let path = dirs::home_dir().unwrap().join(".carapace");
    if let Err(err) = fs::create_dir_all(&path) {
        println!("Could not create init folder: {}\n{}", path.display(), err);
        return 1;
    }

    let config = Config::new();
    let mut prompt = Prompt::new(&config);

    loop {
        match prompt.parse_command() {
            Ok(cmd) => match cmd.execute(&mut prompt) {
                Ok(_) => continue,
                Err(code) => {
                    return code;
                }
            },
            Err(err) => {
                if err.is::<EofError>() {
                    return 0;
                } else {
                    println!("{}", err);
                }
            }
        }
    }
}
