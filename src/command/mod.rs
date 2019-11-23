use std::any::Any;
use std::process;

use super::prompt::{EofError, Prompt, PromptResult};

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

pub mod set_command;
use self::set_command::SetCommand;

/// Base trait of all commands.
pub trait Command {
    /// Execute command and return `Ok(true)` if command was run successfully, `Ok(false)` if not,
    /// and `Err(exit_code)` on "exit" or "quit".
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32>;

    /// Enable downcasting from trait object, like `dyn Command`, to concrete type, like
    /// `ExitCommand`.
    fn as_any(&self) -> &dyn Any;
}

/// Create command instance from `program` and `args`.
pub fn parse(program: String, args: Vec<String>) -> Box<dyn Command> {
    match program.as_ref() {
        "cd" => Box::new(CdCommand::new(args)),
        "exit" => Box::new(ExitCommand::new(args)),
        "export" => Box::new(ExportCommand::new(args)),
        "set" => Box::new(SetCommand::new(args)),
        "history" | "hist" | "h" => Box::new(HistoryCommand::new(args)),
        "quit" => Box::new(QuitCommand {}),
        "unset" => Box::new(UnsetCommand::new(args)),
        _ => Box::new(GeneralCommand::new(program, args)),
    }
}

/// Execute command and yield optional exit code value.
pub fn execute(cmd: PromptResult, prompt: &mut Prompt) -> Option<i32> {
    match cmd {
        Ok(mut cmd) => match cmd.execute(prompt) {
            Ok(_) => None,
            Err(code) => Some(code),
        },
        Err(err) => {
            if err.is::<EofError>() {
                if prompt.context.borrow().ignoreeof {
                    None
                } else {
                    Some(0)
                }
            } else {
                println!("{}", err);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quit() {
        let cmd = parse(String::from("quit"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<QuitCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_exit() {
        let cmd = parse(String::from("exit"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<ExitCommand>();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().code, 0);
    }

    #[test]
    fn parse_cd() {
        let cmd = parse(String::from("cd"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<CdCommand>();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().path, "~");
    }

    #[test]
    fn parse_history() {
        let cmd = parse(String::from("history"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_history_hist() {
        let cmd = parse(String::from("hist"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_history_h() {
        let cmd = parse(String::from("h"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<HistoryCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_general() {
        let prog = String::from("ls");
        let args = vec![String::from("-lh"), String::from("~/git")];
        let cmd = parse(prog.clone(), args.clone());

        let cmd = cmd.as_any().downcast_ref::<GeneralCommand>();
        assert!(cmd.is_some());

        let cmd = cmd.unwrap();
        assert_eq!(cmd.program, prog);
        assert_eq!(cmd.args, args);
    }
}
