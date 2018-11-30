use super::*;

/// Exit command provides an exit code on execution, if no argument is provided the code zero is
/// used.
pub struct ExitCommand {
    pub code: i32,
}

impl ExitCommand {
    pub fn new(args: Vec<String>) -> Result<ExitCommand, Box<dyn Error>> {
        if args.len() == 0 {
            return Ok(ExitCommand { code: 0 });
        }
        if let Ok(code) = args[0].parse::<i32>() {
            Ok(ExitCommand { code })
        } else {
            return Err(Box::new(CommandError::new("Argument not an integer")));
        }
    }
}

impl Command for ExitCommand {
    fn execute(&self, _prompt: &Prompt) -> Result<bool, i32> {
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
        let cmd = ExitCommand::new(vec![]).unwrap();
        assert_eq!(cmd.code, 0);
    }

    #[test]
    fn invalid_arg() {
        let cmd = ExitCommand::new(vec![String::from("abc")]);
        assert!(cmd.is_err());
    }

    #[test]
    fn valid_arg() {
        let cmd = ExitCommand::new(vec![String::from("42")]).unwrap();
        assert_eq!(cmd.code, 42);
    }
}
