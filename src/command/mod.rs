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

pub mod rehash_command;
use self::rehash_command::RehashCommand;

/// Base trait of all commands.
pub trait Command {
    /// Execute command and return `Ok(true)` if command was run successfully, `Ok(false)` if not,
    /// and `Err(exit_code)` on "exit" or "quit".
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32>;

    /// Enable downcasting from trait object, like `dyn Command`, to concrete type, like
    /// `ExitCommand`.
    fn as_any(&self) -> &dyn Any;
}

/// Commands define their name and aliases with the CommandAliases trait.
pub trait CommandAliases {
    fn aliases() -> Vec<String>;
}

/// Builtin command names and aliases of the shell.
pub fn builtins() -> Vec<String> {
    vec![
        CdCommand::aliases(),
        ExitCommand::aliases(),
        ExportCommand::aliases(),
        HistoryCommand::aliases(),
        QuitCommand::aliases(),
        RehashCommand::aliases(),
        SetCommand::aliases(),
        UnsetCommand::aliases(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Create command instance from `program` and `args`.
pub fn parse(program: String, args: Vec<String>) -> Box<dyn Command> {
    match program.as_ref() {
        "cd" => Box::new(CdCommand::new(args)),
        "exit" => Box::new(ExitCommand::new(args)),
        "export" => Box::new(ExportCommand::new(args)),
        "history" | "hist" | "h" => Box::new(HistoryCommand::new(args)),
        "quit" => Box::new(QuitCommand {}),
        "rehash" => Box::new(RehashCommand {}),
        "set" => Box::new(SetCommand::new(args)),
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
    fn check_builtins() {
        // The order is important!
        let cmds: Vec<String> = vec![
            "cd", "exit", "export", "h", "hist", "history", "quit", "rehash", "set", "unset",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect();
        assert_eq!(cmds, builtins());
    }

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

    #[test]
    fn parse_set() {
        let cmd = parse(String::from("set"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<SetCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_unset() {
        let cmd = parse(String::from("unset"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<UnsetCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_export() {
        let cmd = parse(String::from("export"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<ExportCommand>();
        assert!(cmd.is_some());
    }

    #[test]
    fn parse_rehash() {
        let cmd = parse(String::from("rehash"), vec![]);
        let cmd = cmd.as_any().downcast_ref::<RehashCommand>();
        assert!(cmd.is_some());
    }
}
