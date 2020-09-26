use super::*;

use clap::{App, AppSettings, Arg};
use std::path::Path;

/// Pushd command pushes directory to stack or shows it.
pub struct PushdCommand {
    path: Option<String>,
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl PushdCommand {
    pub fn new(args: Vec<String>) -> PushdCommand {
        let mut app = App::new("pushd")
            .about("When no options are specified, the directory stack will be listed.")
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("directory").index(1));

        let mut path = None;
        let matches = app.get_matches_from_safe_borrow(&args);
        if let Ok(value) = matches {
            if let Some(p) = value.value_of("directory") {
                path = Some(p.to_string());
            }
        }

        PushdCommand { args, path, app }
    }
}

impl Command for PushdCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        if let Some(path) = &self.path {
            if let Some(oldpwd) = prompt.set_cwd(Path::new(&path)) {
                prompt.context.borrow_mut().dir_stack.push(oldpwd);
            }
        }

        // Show stack in all cases.
        let short = true;
        prompt.context.borrow().print_dir_stack(short);

        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for PushdCommand {
    fn aliases() -> Vec<String> {
        vec!["pushd".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_is_none_path() {
        let cmd = PushdCommand::new(vec![]);
        assert_eq!(cmd.path, None);
    }

    #[test]
    fn arg_is_path() {
        let dir = String::from("dir");
        let cmd = PushdCommand::new(vec![dir.clone()]);
        assert_eq!(cmd.path, Some(dir));
    }
}
