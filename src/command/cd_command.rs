use super::*;

use std::path::Path;

use clap::{App, AppSettings, Arg};

/// Cd command changes directory to defined path.
pub struct CdCommand {
    pub path: String,
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl CdCommand {
    /// If no arguments are passed the path will be "~", the home directory, otherwise it will be
    /// the first argument. *Note:* it is expected that all "~" have already been replaced. Only the
    /// placeholder "~" used with no arguments is kept to replace directly in `execute()`.
    pub fn new(args: Vec<String>) -> CdCommand {
        let mut app = App::new("cd")
            .about("Change directory.")
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("directory").index(1).default_value("~"));

        let mut path = "~".to_string();
        let matches = app.get_matches_from_safe_borrow(&args);
        if let Ok(value) = matches {
            path = value.value_of("directory").unwrap().to_string();
        }

        CdCommand { args, path, app }
    }
}

impl Command for CdCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        if self.path == "~" {
            let home_dir = dirs::home_dir().unwrap_or_default();
            prompt.set_cwd(&home_dir);
        } else {
            let path = Path::new(&self.path);
            prompt.set_cwd(&path);
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for CdCommand {
    fn aliases() -> Vec<String> {
        vec!["cd".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_is_tilde() {
        let cmd = CdCommand::new(vec![]);
        assert_eq!(cmd.path, "~");
    }

    #[test]
    fn valid_arg() {
        let cmd = CdCommand::new(vec![String::from("/tmp")]);
        assert_eq!(cmd.path, "/tmp");
    }
}
