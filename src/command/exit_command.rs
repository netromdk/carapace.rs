use super::*;

use clap::{App, AppSettings, Arg};

/// Exit command provides an exit code on execution, if no argument is provided the code zero is
/// used.
pub struct ExitCommand {
    pub code: i32,
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl ExitCommand {
    pub fn new(args: Vec<String>) -> ExitCommand {
        let mut app = App::new("exit")
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::DisableVersion)
            .arg(
                Arg::with_name("code")
                    .index(1)
                    .default_value("0")
                    .help("Exit code to return to parent program."),
            );

        let mut code = 0;
        let matches = app.get_matches_from_safe_borrow(&args);
        if matches.is_ok() {
            if let Ok(c) = matches.unwrap().value_of("code").unwrap().parse::<i32>() {
                code = c;
            }
        }

        ExitCommand { code, args, app }
    }
}

impl Command for ExitCommand {
    fn execute(&mut self, _prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if matches.is_err() {
            println!("{}", matches.unwrap_err());
            return Ok(false);
        }

        Err(self.code)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_is_zero() {
        let cmd = ExitCommand::new(vec![]);
        assert_eq!(cmd.code, 0);
    }

    #[test]
    fn invalid_arg() {
        let cmd = ExitCommand::new(vec![String::from("abc")]);
        assert_eq!(cmd.code, 0);
    }

    #[test]
    fn valid_arg() {
        let cmd = ExitCommand::new(vec![String::from("42")]);
        assert_eq!(cmd.code, 42);
    }
}
