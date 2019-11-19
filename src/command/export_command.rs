use super::*;

use clap::{App, AppSettings, Arg};

/// Export command adds (variable, value) pairs to environment.
pub struct ExportCommand {
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl ExportCommand {
    pub fn new(args: Vec<String>) -> ExportCommand {
        ExportCommand {
            args,
            app: App::new("export")
                .about("List or export new environment variables with values.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(
                    Arg::with_name("vars").multiple(true).help(
                        "Variable with optional value input as: 'variable' or 'variable=value'",
                    ),
                ),
        }
    }
}

impl Command for ExportCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        if self.args.is_empty() {
            let ctx = prompt.context.borrow();
            let mut keys: Vec<&String> = ctx.env.keys().peekable().collect();
            keys.sort();
            for k in &keys {
                println!("{}={}", k, ctx.env[*k]);
            }
        } else {
            for var in &self.args {
                let (k, v) = match var.find('=') {
                    Some(pos) => (var[..pos].to_string(), var[pos + 1..].to_string()),
                    None => (var.clone(), String::from("")),
                };
                prompt.context.borrow_mut().env.insert(k, v);
            }
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
