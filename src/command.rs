use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    /// Parses a vector of strings into a `Command` with command name and arguments.
    pub fn new(values: Vec<&str>) -> Result<Command, Box<dyn Error>> {
        if values.len() == 0 {
            return Err(Box::new(NoCommandError));
        }

        Ok(Command {
            name: values[0].to_string(),
            args: values[1..].iter().map(|x| x.to_string()).collect(),
        })
    }
}

#[derive(Debug)]
struct NoCommandError;

impl Error for NoCommandError {}

impl fmt::Display for NoCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No command inputted.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_command() {
        let cmd = Command::new(vec![]);
        assert!(cmd.is_err());
    }

    #[test]
    fn parse_command() {
        let cmd = Command::new(vec!["one"]).unwrap();
        assert_eq!(cmd.name, "one");
    }

    #[test]
    fn parse_command_one_arg() {
        let cmd = Command::new(vec!["one", "two"]).unwrap();
        assert_eq!(cmd.name, "one");
        assert_eq!(cmd.args, vec!["two"]);
    }

    #[test]
    fn parse_command_two_args() {
        let cmd = Command::new(vec!["one", "two", "three"]).unwrap();
        assert_eq!(cmd.name, "one");
        assert_eq!(cmd.args, vec!["two", "three"]);
    }
}
