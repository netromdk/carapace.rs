use std::collections::HashMap;
use std::env;

pub struct Context {
    /// Environment passed to newly spawned processes.
    pub env: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            env: env::vars().collect(),
        }
    }
}
