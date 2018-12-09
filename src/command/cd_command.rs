use super::*;

use std::env;
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
        if matches.is_ok() {
            path = matches.unwrap().value_of("directory").unwrap().to_string();
        }

        CdCommand { args, path, app }
    }

    fn set_cwd(&self, dir: &Path, prompt: &mut Prompt) {
        let fallback = "/";
        let oldpwd = env::current_dir()
            .unwrap_or(Path::new(fallback).to_path_buf())
            .to_str()
            .unwrap_or(fallback)
            .to_string();
        if let Err(err) = env::set_current_dir(dir) {
            println!("Could not change to {}: {}", dir.display(), err);
        } else {
            prompt
                .context
                .borrow_mut()
                .env
                .insert("OLDPWD".to_string(), oldpwd);
        }
    }
}

impl Command for CdCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if matches.is_err() {
            println!("{}", matches.unwrap_err());
            return Ok(false);
        }

        if self.path == "~" {
            let home_dir = dirs::home_dir().unwrap_or_default();
            self.set_cwd(&home_dir, prompt);
        } else {
            let path = Path::new(&self.path);
            self.set_cwd(&path, prompt);
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
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
