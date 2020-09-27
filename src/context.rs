use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::env::Env;
use crate::path_commands::PathCommands;

pub type Context = Rc<RefCell<ContextData>>;

pub fn new(verbose: u64, config_path: Option<&str>) -> Context {
    Rc::new(RefCell::new(ContextData::new(verbose, config_path)))
}

pub fn default() -> Context {
    Rc::new(RefCell::new(ContextData::default()))
}

pub struct ContextData {
    pub verbose: u64,
    pub config: Config,

    /// Environment passed to newly spawned processes.
    pub env: Env,

    /// Commands detected in $PATH.
    pub commands: PathCommands,

    /// Extra trace option (set via `set -x`) outputs command trace to stdout.
    pub xtrace: bool,

    /// Whether or not to exit shell immediately if a command exit with non-zero status
    /// (set via `set -e`).
    pub errexit: bool,

    /// Whether or not to not exit shell when reading EOF.
    pub ignoreeof: bool,

    /// Stack of directories manipulated via `pushd` and `popd`.
    pub dir_stack: Vec<String>,
}

impl ContextData {
    pub fn new(verbose: u64, config_path: Option<&str>) -> ContextData {
        ContextData {
            verbose,
            config: Config::new(config_path),
            env: Env::new(),
            commands: PathCommands::new(),
            xtrace: false,
            errexit: false,
            ignoreeof: false,
            dir_stack: Vec::new(),
        }
    }

    /// Prints directory stack to stdout.
    ///
    /// `short` means all on one line, otherwise a list.
    pub fn print_dir_stack(&self, short: bool) {
        if self.dir_stack.is_empty() {
            return;
        }

        // Print in reverse order so that the head dir is first.
        let len = self.dir_stack.len();
        for (num, dir) in self.dir_stack.iter().rev().enumerate() {
            if short {
                if num == 0 {
                    if len == 1 {
                        println!("{}", dir);
                    } else {
                        print!("[{}] ", dir);
                    }
                } else if num < len - 1 {
                    print!("{} ", dir);
                } else {
                    println!("{}", dir);
                }

            // Long formats.
            } else if num == 0 {
                println!("{}>\t{}", num, dir);
            } else {
                println!("{} \t{}", num, dir);
            }
        }
    }

    pub fn print_short_dir_stack(&self) {
        let short = true;
        self.print_dir_stack(short);
    }
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            verbose: 0,
            config: Config::default(),
            env: Env::default(),
            commands: PathCommands::default(),
            xtrace: false,
            errexit: false,
            ignoreeof: false,
            dir_stack: Vec::new(),
        }
    }
}
