use std::any::Any;
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

    /// Enable downcasting from trait object, like `dyn Command`, to concrete type, like
    /// `ExitCommand`.
    fn as_any(&self) -> &dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Quit command provides an exit code of zeroo on execution.
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self) -> Result<bool, i32> {
        Err(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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
    fn exit_cmd_no_args_is_zero() {
        let cmd = ExitCommand::new(vec![]).unwrap();
        assert_eq!(cmd.code, 0);
    }

    #[test]
    fn exit_cmd_invalid_arg() {
        let cmd = ExitCommand::new(vec![String::from("abc")]);
        assert!(cmd.is_err());
    }

    #[test]
    fn exit_cmd_valid_arg() {
        let cmd = ExitCommand::new(vec![String::from("42")]).unwrap();
        assert_eq!(cmd.code, 42);
    }

    #[test]
    fn exit_cmd_execute_returns_code() {
        let cmd = ExitCommand::new(vec![String::from("42")]).unwrap();
        let res = cmd.execute();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), 42);
    }

    #[test]
    fn quit_cmd_execute_returns_zero() {
        let cmd = QuitCommand {};
        let res = cmd.execute();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), 0);
    }

    #[test]
    fn cd_cmd_no_args_is_tilde() {
        let cmd = CdCommand::new(vec![]);
        assert_eq!(cmd.path, "~");
    }

    #[test]
    fn cd_cmd_valid_arg() {
        let cmd = CdCommand::new(vec![String::from("/tmp")]);
        assert_eq!(cmd.path, "/tmp");
    }

    #[test]
    fn general_cmd_new() {
        let prog = String::from("prog");
        let args = vec![String::from("arg")];
        let cmd = GeneralCommand::new(prog.clone(), args.clone());
        assert_eq!(cmd.program, prog);
        assert_eq!(cmd.args, args);
    }

    #[test]
    fn parse_command_quit() {
        let cmd = parse_command(String::from("quit"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<QuitCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_command_exit() {
        let cmd = parse_command(String::from("exit"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<ExitCommand>();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().code, 0);
    }

    #[test]
    fn parse_command_cd() {
        let cmd = parse_command(String::from("cd"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<CdCommand>();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().path, "~");
    }

    #[test]
    fn parse_command_general() {
        let prog = String::from("ls");
        let args = vec![String::from("-lh"), String::from("~/git")];
        let cmd = parse_command(prog.clone(), args.clone()).unwrap();

        let cmd = cmd.as_any().downcast_ref::<GeneralCommand>();
        assert!(cmd.is_some());

        let cmd = cmd.unwrap();
        assert_eq!(cmd.program, prog);
        assert_eq!(cmd.args, args);
    }
}
