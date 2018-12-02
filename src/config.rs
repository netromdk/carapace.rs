use json::{self, JsonValue};

use rustyline::{CompletionType, EditMode};

use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub max_history_size: usize,
    pub edit_mode: EditMode,
    pub completion_type: CompletionType,
    pub aliases: HashMap<String, String>, // alias -> actual command.
}

impl Config {
    pub fn new() -> Config {
        // Set defaults.
        let mut c = Config {
            max_history_size: 1000,
            edit_mode: EditMode::Emacs,
            completion_type: CompletionType::List,
            aliases: HashMap::new(),
        };
        c.load();
        c
    }

    pub fn load(&mut self) {
        let path = dirs::home_dir()
            .unwrap()
            .join(".carapace")
            .join("config.json");

        // If config does not exist then save defaults to disk.
        if !path.exists() {
            self.save();
            return;
        }

        match fs::read(&path) {
            Ok(contents) => {
                let data = String::from_utf8_lossy(&contents);
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
                                "aliases" => {
                                    for (key, val) in value.entries() {
                                        if let Some(s) = val.as_str() {
                                            self.aliases.insert(key.to_string(), s.to_string());
                                        }
                                    }
                                }
                                _ => println!("Unknown config entry: {}={}", key, value),
                            }
                        }
                    }
                    Err(err) => {
                        println!("Could not parse config from: {}\n{}", path.display(), err)
                    }
                }
            }
            Err(err) => println!("Could not load config from: {}\n{}", path.display(), err),
        }
    }

    pub fn save(&self) {
        let mut aliases = JsonValue::new_object();
        for (key, value) in &self.aliases {
            aliases[key] = JsonValue::from(value.clone());
        }

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
            "aliases" => aliases,
        ];

        let output = json::stringify_pretty(output, 2);
        let path = dirs::home_dir()
            .unwrap()
            .join(".carapace")
            .join("config.json");
        if let Err(err) = fs::write(&path, output) {
            println!("Could not write config to: {}\n{}", path.display(), err);
        }
    }
}
