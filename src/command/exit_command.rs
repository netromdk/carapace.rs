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
                    .help("Exit code to return to parent program.")
                    .index(1)
                    .default_value("0")
                    .validator(|v: String| -> Result<(), String> {
                        if v.parse::<i32>().is_ok() {
                            return Ok(());
                        }
                        Err(String::from("Exit code must be an integer!"))
                    }),
            );

        let mut code = 0;
        let matches = app.get_matches_from_safe_borrow(&args);
        if let Ok(value) = matches {
            code = value.value_of("code").unwrap().parse::<i32>().unwrap();
        }

        ExitCommand { code, args, app }
    }
}

impl Command for ExitCommand {
    fn execute(&mut self, _prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
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
