use super::*;

use std::path::PathBuf;

use clap::{App, AppSettings, Arg};

/// Cd command changes directory to defined path.
pub struct CdCommand {
    pub path: String,
    program: String,
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl CdCommand {
    /// If no arguments are passed the path will be "~", the home directory, otherwise it will be
    /// the first argument. *Note:* it is expected that all "~" have already been replaced. Only the
    /// placeholder "~" used with no arguments is kept to replace directly in `execute()`.
    pub fn new(program: String, args: Vec<String>) -> CdCommand {
        let mut app = App::new("cd")
            .about("Change directory and push to directory stack.")
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("directory").index(1).default_value("~"));

        let mut path = "~".to_string();
        let matches = app.get_matches_from_safe_borrow(&args);
        if let Ok(value) = matches {
            path = value.value_of("directory").unwrap().to_string();
        }

        CdCommand {
            args,
            program,
            path,
            app,
        }
    }
}

impl Command for CdCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        let path = if self.path == "~" {
            dirs_next::home_dir().unwrap_or_default()
        } else {
            PathBuf::from(&self.path)
        };

        if let Some(oldpwd) = prompt.set_cwd(&path) {
            let mut ctx = prompt.context.borrow_mut();

            // Only add to stack if empty or not the same value as the head value.
            let head = ctx.dir_stack.last();
            if head.is_none() || head.unwrap() != &oldpwd {
                ctx.dir_stack.push(oldpwd);
            }

            if self.program == "pushd" {
                ctx.print_short_dir_stack();
            }
        }

        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for CdCommand {
    fn aliases() -> Vec<String> {
        vec!["cd".to_string(), "pushd".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_is_tilde() {
        let cmd = CdCommand::new("cd".to_string(), vec![]);
        assert_eq!(cmd.path, "~");
    }

    #[test]
    fn valid_arg() {
        let cmd = CdCommand::new("cd".to_string(), vec![String::from("/tmp")]);
        assert_eq!(cmd.path, "/tmp");
    }
}
