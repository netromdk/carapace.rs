use crate::util;

use rustyline::{CompletionType, EditMode};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub max_history_size: usize,
    pub edit_mode: EditMode,
    pub completion_type: CompletionType,
    pub auto_cd: bool,
    pub aliases: HashMap<String, String>, // alias -> actual command.
    pub env: HashMap<String, String>,     // env var -> value.
}

impl Config {
    pub fn new(path: Option<&str>) -> Config {
        let mut c = Config::default();
        c.load(path);
        c
    }

    pub fn load(&mut self, path: Option<&str>) {
        let path = if let Some(path_) = path {
            PathBuf::from(path_)
        } else {
            dirs::home_dir()
                .unwrap()
                .join(".carapace")
                .join("config.json")
        };

        // If config does not exist then save defaults to disk.
        if !path.exists() {
            self.save(&path);
            return;
        }

        match fs::read(&path) {
            Ok(contents) => {
                let data = String::from_utf8_lossy(&contents);
                self.decode(&data);
            }
            Err(err) => println!("Could not load config from: {}\n{}", path.display(), err),
        }
    }

    pub fn save(&self, path: &PathBuf) {
        let output = self.encode();
        if let Err(err) = fs::write(&path, output) {
            println!("Could not write config to: {}\n{}", path.display(), err);
        }
    }

    /// Encodes config values into a JSON string.
    fn encode(&self) -> String {
        let output = json::object![
            "max_history_size" => self.max_history_size,
            "edit_mode" => match self.edit_mode {
                EditMode::Emacs => "emacs",
                EditMode::Vi => "vi"
            },
            "completion_type" => match self.completion_type {
                CompletionType::List => "list",
                CompletionType::Circular => "circular",
            },
            "auto_cd" => self.auto_cd,
            "aliases" => util::hash_map_to_json(&self.aliases),
            "env" => util::hash_map_to_json(&self.env),
        ];

        json::stringify_pretty(output, 2)
    }

    /// Decodes JSON `data` into config values.
    fn decode(&mut self, data: &str) -> bool {
        match json::parse(&data) {
            Ok(input) => {
                for (key, value) in input.entries() {
                    match key.to_lowercase().as_ref() {
                        "max_history_size" => {
                            self.max_history_size =
                                value.as_usize().unwrap_or(self.max_history_size)
                        }
                        "edit_mode" => {
                            self.edit_mode = match value.as_str().unwrap_or("emacs") {
                                        "vi" => EditMode::Vi,
                                        _ /*"emacs"*/ => EditMode::Emacs,
                                    };
                        }
                        "completion_type" => {
                            self.completion_type = match value.as_str().unwrap_or("list") {
                                        "circular" => CompletionType::Circular,
                                        _ /*"list"*/ => CompletionType::List,
                                    };
                        }
                        "auto_cd" => {
                            self.auto_cd = value.as_bool().unwrap_or(true);
                        }
                        "aliases" => {
                            self.aliases = util::json_obj_to_hash_map(value);
                        }
                        "env" => {
                            self.env = util::json_obj_to_hash_map(value);
                        }
                        _ => println!("Unknown config entry: {}={}", key, value),
                    }
                }
                return true;
            }
            Err(err) => println!("Could not parse config: {}", err),
        }
        false
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_history_size: 1000,
            edit_mode: EditMode::Emacs,
            completion_type: CompletionType::List,
            auto_cd: true,
            aliases: HashMap::new(),
            env: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_default() {
        let config = Config::default();
        let output = config.encode();
        assert_eq!(
            output,
            r#"{
  "max_history_size": 1000,
  "edit_mode": "emacs",
  "completion_type": "list",
  "auto_cd": true,
  "aliases": {},
  "env": {}
}"#
        );
    }

    #[test]
    fn decode() {
        let mut config = Config {
            max_history_size: 1,
            edit_mode: EditMode::Vi,
            completion_type: CompletionType::Circular,
            auto_cd: false,
            aliases: HashMap::new(),
            env: HashMap::new(),
        };
        assert!(config.decode(
            r#"{
  "max_history_size": 123,
  "edit_mode": "emacs",
  "completion_type": "list",
  "auto_cd": true,
  "aliases": {
    "l": "ls",
    "ll": "ls -l"
  },
  "env": {
    "PATH": "$PATH:/something/bin"
  }
}"#
        ));
        assert_eq!(config.max_history_size, 123);
        assert_eq!(config.edit_mode, EditMode::Emacs);
        assert_eq!(config.completion_type, CompletionType::List);
        assert_eq!(config.auto_cd, true);
        assert_eq!(config.aliases.len(), 2);
        assert!(config.aliases.contains_key("l"));
        assert_eq!(config.aliases.get("l"), Some(&String::from("ls")));
        assert!(config.aliases.contains_key("ll"));
        assert_eq!(config.aliases.get("ll"), Some(&String::from("ls -l")));
        assert_eq!(config.env.len(), 1);
        assert!(config.env.contains_key("PATH"));
        assert_eq!(
            config.env.get("PATH"),
            Some(&String::from("$PATH:/something/bin"))
        );
    }

    #[test]
    fn encode_decode() {
        let config = Config::default();
        let output = config.encode();
        let mut config2 = Config {
            max_history_size: 1,
            edit_mode: EditMode::Vi,
            completion_type: CompletionType::Circular,
            auto_cd: false,
            aliases: HashMap::new(),
            env: HashMap::new(),
        };
        assert!(config2.decode(output.as_ref()));
        assert_eq!(config, config2);
    }

    #[test]
    fn decode_invalid_data() {
        let mut config = Config::default();
        assert!(!config.decode(""));
        assert!(!config.decode("{"));
        assert!(!config.decode(r#"{"edit_mode":"#));
        assert!(!config.decode(
            r#"{
  "aliases": {
    "ls":
  }
}"#
        ));
    }
}
