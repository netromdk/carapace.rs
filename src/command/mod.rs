use std::any::Any;
use std::error::Error;
use std::fmt;
use std::process;

use super::Prompt;

pub mod exit_command;
use self::exit_command::ExitCommand;

pub mod quit_command;
use self::quit_command::QuitCommand;

pub mod cd_command;
use self::cd_command::CdCommand;

pub mod general_command;
use self::general_command::GeneralCommand;

pub mod history_command;
use self::history_command::HistoryCommand;

pub mod unset_command;
use self::unset_command::UnsetCommand;

pub mod export_command;
use self::export_command::ExportCommand;

/// Base trait of all commands.
pub trait Command {
    /// Execute command and return `Ok(true)` if command was run successfully, `Ok(false)` if not,
    /// and `Err(exit_code)` on "exit" or "quit".
    fn execute(&self, prompt: &mut Prompt) -> Result<bool, i32>;

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

/// Create command instance from `program` and `args`.
pub fn parse_command(
    program: String,
    args: Vec<String>,
) -> Result<Box<dyn Command>, Box<dyn Error>> {
    match program.as_ref() {
        "cd" => Ok(Box::new(CdCommand::new(args))),
        "exit" => Ok(Box::new(ExitCommand::new(args)?)),
        "export" => Ok(Box::new(ExportCommand::new(args))),
        "history" | "hist" | "h" => Ok(Box::new(HistoryCommand {})),
        "quit" => Ok(Box::new(QuitCommand {})),
        "unset" => Ok(Box::new(UnsetCommand::new(args)?)),
        _ => Ok(Box::new(GeneralCommand::new(program, args))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_command_history() {
        let cmd = parse_command(String::from("history"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_command_history_hist() {
        let cmd = parse_command(String::from("hist"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_command_history_h() {
        let cmd = parse_command(String::from("h"), vec![]).unwrap();
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
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
