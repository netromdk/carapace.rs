use super::*;

/// Export command adds (variable, value) pairs to environment.
pub struct ExportCommand {
    vars: Vec<String>,
}

impl ExportCommand {
    pub fn new(args: Vec<String>) -> ExportCommand {
        ExportCommand { vars: args }
    }
}

impl Command for ExportCommand {
    fn execute(&self, prompt: &mut Prompt) -> Result<bool, i32> {
        if self.vars.len() == 0 {
            let mut keys: Vec<&String> = prompt.env.keys().peekable().collect();
            keys.sort();
            for k in &keys {
                println!("{}={}", k, prompt.env[*k]);
            }
        } else {
            for var in &self.vars {
                let (k, v) = match var.find('=') {
                    Some(pos) => (var[..pos].to_string(), var[pos + 1..].to_string()),
                    None => (var.clone(), String::from("")),
                };
                prompt.env.insert(k, v);
            }
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
