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

extern crate clap;
extern crate dirs;
extern crate json;
extern crate regex;
extern crate rustyline;
extern crate term;

#[macro_use]
extern crate lazy_static;

pub mod command;
pub mod config;
pub mod context;
pub mod editor;
pub mod prompt;
pub mod util;

use prompt::{EofError, Prompt};

use clap::ArgMatches;

use std::fs;

/// Starts the read-eval-print-loop of the Carapace shell, with supplied, parsed CLI arguments, if
/// any. Returns the exit code.
pub fn repl(arg_matches: &ArgMatches) -> i32 {
    // Create init folder if not present.
    let path = dirs::home_dir().unwrap().join(".carapace");
    if let Err(err) = fs::create_dir_all(&path) {
        println!("Could not create init folder: {}\n{}", path.display(), err);
        return 1;
    }

    let context = context::new();
    let mut prompt = Prompt::new(context);

    // If -c <command> is specified then run command and exit.
    if let Some(command) = arg_matches.value_of("command") {
        match prompt.parse_command(command.to_string()) {
            Ok(cmd) => match cmd.execute(&mut prompt) {
                Ok(_) => (),
                Err(code) => return code,
            },
            Err(err) => {
                println!("{}", err);
            }
        }
        return 0;
    }

    loop {
        match prompt.show_parse_command() {
            Ok(cmd) => match cmd.execute(&mut prompt) {
                Ok(_) => (),
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
