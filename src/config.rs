use json;

use rustyline::{CompletionType, EditMode};

use std::fs;

#[derive(Debug)]
pub struct Config {
    pub edit_mode: EditMode,
    pub completion_type: CompletionType,
}

impl Config {
    pub fn new() -> Config {
        // Set defaults.
        let mut c = Config {
            edit_mode: EditMode::Emacs,
            completion_type: CompletionType::List,
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
                                "edit_mode" => {
                                    self.edit_mode = match value.as_str().unwrap() {
                                        "vi" => EditMode::Vi,
                                        _ /*"emacs"*/ => EditMode::Emacs,
                                    };
                                }
                                "completion_type" => {
                                    self.completion_type = match value.as_str().unwrap() {
                                        "circular" => CompletionType::Circular,
                                        _ /*"list"*/ => CompletionType::List,
                                    };
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
        let output = json::object![
            "edit_mode" => match self.edit_mode {
                EditMode::Emacs => "emacs",
                EditMode::Vi => "vi"
            },
            "completion_type" => match self.completion_type {
                CompletionType::List => "list",
                CompletionType::Circular => "circular",
            },
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
