use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process;

/// Base trait of all commands.
pub trait Command {
    /// Execute command and return `Ok(true)` if command was run successfully, `Ok(false)` if not,
    /// and `Err(exit_code)` on "exit" or "quit".
    fn execute(&self) -> Result<bool, i32>;
}

#[derive(Debug)]
struct CommandError {
    error: &'static str,
}

impl CommandError {
    fn new(error: &'static str) -> CommandError {
        CommandError { error }
    }
}

impl Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

/// Exit command provides an exit code on execution, if no argument is provided the code zero is
/// used.
pub struct ExitCommand {
    code: i32,
}

impl ExitCommand {
    fn new(args: Vec<String>) -> Result<ExitCommand, Box<dyn Error>> {
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
    fn execute(&self) -> Result<bool, i32> {
        Err(self.code)
    }
}

/// Quit command provides an exit code of zeroo on execution.
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self) -> Result<bool, i32> {
        Err(0)
    }
}

/// Cd command changes directory to defined path.
pub struct CdCommand {
    path: String,
}

impl CdCommand {
    /// If no arguments are passed the path will be "~", the home directory, otherwise it will be
    /// the first argument.
    fn new(args: Vec<String>) -> CdCommand {
        let path = if args.len() > 0 {
            args[0].clone()
        } else {
            String::from("~")
        };
        CdCommand { path }
    }

    fn set_cwd(&self, dir: &Path) {
        if let Err(err) = env::set_current_dir(dir) {
            println!("Could not change to {}: {}", dir.display(), err);
        }
    }
}

impl Command for CdCommand {
    fn execute(&self) -> Result<bool, i32> {
        let home_dir = dirs::home_dir().unwrap_or_default();
        let path = Path::new(&self.path);
        if path.starts_with("~") {
            self.set_cwd(&home_dir.join(path.strip_prefix("~").unwrap()));
        } else {
            self.set_cwd(path);
        }

        Ok(true)
    }
}

/// General command that executes program with arguments and waits for it to finish.
pub struct GeneralCommand {
    program: String,
    args: Vec<String>,
}

impl GeneralCommand {
    fn new(program: String, args: Vec<String>) -> GeneralCommand {
        GeneralCommand { program, args }
    }
}

impl Command for GeneralCommand {
    fn execute(&self) -> Result<bool, i32> {
        let output = process::Command::new(&self.program)
            .args(&self.args)
            .output();
        match output {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                Ok(true)
            }
            Err(err) => {
                println!("{}", err);
                Ok(false)
            }
        }
    }
}

/// Create command instance from `program` and `args`.
pub fn parse_command(
    program: String,
    args: Vec<String>,
) -> Result<Box<dyn Command>, Box<dyn Error>> {
    match program.as_ref() {
        "quit" => Ok(Box::new(QuitCommand {})),
        "exit" => Ok(Box::new(ExitCommand::new(args)?)),
        "cd" => Ok(Box::new(CdCommand::new(args))),
        _ => Ok(Box::new(GeneralCommand::new(program, args))),
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
        assert_eq!(cmd.program, "one");
    }

    #[test]
    fn parse_command_one_arg() {
        let cmd = Command::new(vec!["one", "two"]).unwrap();
        assert_eq!(cmd.program, "one");
        assert_eq!(cmd.args, vec!["two"]);
    }

    #[test]
    fn parse_command_two_args() {
        let cmd = Command::new(vec!["one", "two", "three"]).unwrap();
        assert_eq!(cmd.program, "one");
        assert_eq!(cmd.args, vec!["two", "three"]);
    }
}
