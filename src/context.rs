use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use config::Config;

pub type Context = Rc<RefCell<ContextData>>;

pub fn new(config_path: Option<&str>) -> Context {
    Rc::new(RefCell::new(ContextData::new(config_path)))
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
    pub fn new(config_path: Option<&str>) -> ContextData {
        ContextData {
            config: Config::new(config_path),
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
