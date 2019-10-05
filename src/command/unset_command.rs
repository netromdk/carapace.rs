use super::*;

use clap::{App, AppSettings, Arg};

/// Unset command removes variables from environment.
pub struct UnsetCommand {
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl UnsetCommand {
    pub fn new(args: Vec<String>) -> UnsetCommand {
        UnsetCommand {
            args,
            app: App::new("unset")
                .about("Unset environment variables.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(Arg::with_name("vars").multiple(true).help("Variable name.")),
        }
    }
}

impl Command for UnsetCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        for var in &self.args {
            prompt.context.borrow_mut().env.remove(var);
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
