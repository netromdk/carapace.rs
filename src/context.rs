use std::collections::HashMap;
use std::env;

use config::Config;

pub struct Context {
    pub config: Config,

    /// Environment passed to newly spawned processes.
    pub env: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            config: Config::new(),
            env: env::vars().collect(),
        }
    }
}

impl Default for Context {
    fn default() -> Context {
        Context {
            config: Config::default(),
            env: HashMap::new(),
        }
    }
}
