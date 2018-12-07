use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use config::Config;

pub type Context = Rc<RefCell<ContextData>>;

pub fn new() -> Context {
    Rc::new(RefCell::new(ContextData::new()))
}

pub fn default() -> Context {
    Rc::new(RefCell::new(ContextData::default()))
}

pub struct ContextData {
    pub config: Config,

    /// Environment passed to newly spawned processes.
    pub env: HashMap<String, String>,
}

impl ContextData {
    pub fn new() -> ContextData {
        ContextData {
            config: Config::new(),
            env: env::vars().collect(),
        }
    }
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            config: Config::default(),
            env: HashMap::new(),
        }
    }
}
