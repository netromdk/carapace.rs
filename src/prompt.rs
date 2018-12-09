use command::{self, CommandResult};
use context::Context;
use editor::{self, EditorHelper};
use util;

use std::env;
use std::error::Error;
use std::fmt;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

use term::{self, Terminal};

use rustyline::error::ReadlineError;
use rustyline::Editor;

/// Fallback textual prompt if term formatting fails.
const SAFE_PROMPT: &'static str = "carapace % ";

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt {
    /// General context of the shell.
    pub context: Context,

    /// Readline interface.
    pub editor: Editor<EditorHelper>,
}

impl Prompt {
    pub fn new(context: Context) -> Prompt {
        let editor = editor::create(context.clone());
        let mut p = Prompt { context, editor };
        p.load_history();
        p.load_env();
        p
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn show_parse_command(&mut self) -> CommandResult {
        let prompt_txt = self.prompt();

        let input = self.editor.readline(prompt_txt.as_ref());
        match input {
            Ok(line) => self.parse_command(line),
            Err(ReadlineError::Interrupted) => {
                // TODO: Unhandled for now!
                println!("^C");
                Err(Box::new(NoCommandError))
            }
            Err(ReadlineError::Eof) => Err(Box::new(EofError)),
            Err(err) => {
                println!("Error: {:?}", err);
                Err(Box::new(err))
            }
        }
    }

    /// Parses command from input.
    pub fn parse_command(&mut self, input: String) -> CommandResult {
        self.editor.add_history_entry(input.as_ref());

        let mut input = input.trim().to_string();
        if input.len() == 0 {
            return Err(Box::new(NoCommandError));
        }

        // Replace all `$VAR` and `${VAR}` occurrences with values from environment.
        input = util::replace_vars(&input, &self.context.borrow().env);

        let mut values: Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();

        // Check if program is an alias, and substitute in values.
        if self
            .context
            .borrow()
            .config
            .aliases
            .contains_key(&values[0])
        {
            let alias_values: Vec<String> = self.context.borrow().config.aliases[&values[0]]
                .split_whitespace()
                .map(|x| x.to_string())
                .collect();
            let mut new_values = alias_values;
            new_values.append(&mut values.drain(1..).collect());
            values = new_values;
        }

        // Replace all ~ with home dir (for parts starting with it only).
        let home_dir = dirs::home_dir().unwrap_or_default();
        values = values
            .into_iter()
            .map(|mut x| {
                if !x.starts_with("~") {
                    x
                } else {
                    let cnt = if x.starts_with("~/") { 2 } else { 1 };
                    let rest: String = x.drain(cnt..).collect();
                    if let Ok(res) = home_dir.join(&rest).into_os_string().into_string() {
                        res
                    } else {
                        x
                    }
                }
            }).collect();

        let mut program = values[0].clone();
        let mut args: Vec<String> = values.drain(1..).collect();

        // If input is an existing folder, and auto_cd is enabled, then set "cd" as the
        // program.
        if self.context.borrow().config.auto_cd
            && values.len() == 1
            && Path::new(&values[0]).is_dir()
        {
            args = vec![program];
            program = "cd".to_string();
        }

        command::parse(program, args)
    }

    /// Yields the textual prompt with term colors.
    fn prompt(&self) -> String {
        // In case of failure, use safe prompt. It is a closure so it is only allocated if it is
        // needed.
        let safe_prompt = || SAFE_PROMPT.to_string();

        // In-memory cursor using a vector for data.
        let cursor = Cursor::new(Vec::new());

        let t = term::TerminfoTerminal::new(cursor);
        if t.is_none() {
            return safe_prompt();
        }
        let mut t = t.unwrap();

        // Create textual prompt.
        if t.fg(term::color::GREEN).is_err() {
            return safe_prompt();
        }
        write!(t, "carapace");

        if let Ok(cwd) = env::current_dir() {
            if t.fg(term::color::BRIGHT_BLUE).is_err() {
                return safe_prompt();
            }
            write!(t, " {}", cwd.display());
        }

        if t.fg(term::color::GREEN).is_err() {
            return safe_prompt();
        }
        write!(t, " % ");

        // NOTE: Resetting yields extra space at end with is annoying, so hardcoding to white color
        // at end.
        // t.reset().unwrap();
        if t.fg(term::color::WHITE).is_err() {
            return safe_prompt();
        }

        // Get inner cursor (it was moved above) and read what was written to its data vector.
        let mut inner_cursor = t.into_inner();
        if inner_cursor.seek(SeekFrom::Start(0)).is_err() {
            return safe_prompt();
        }

        let mut out = Vec::new();
        if inner_cursor.read_to_end(&mut out).is_err() {
            return safe_prompt();
        }

        String::from_utf8_lossy(&out).into_owned()
    }

    fn load_history(&mut self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if let Err(_) = self.editor.load_history(&path) {
            println!("No history loaded.");
        }
    }

    pub fn save_history(&self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if let Err(err) = self.editor.save_history(&path) {
            println!("Could not save history to: {}\n{}", path.display(), err);
        }
    }

    /// Load environment entries from config into session environment.
    fn load_env(&mut self) {
        for (k, v) in &self.context.borrow().config.env {
            let mut ctx = self.context.borrow_mut();
            let v = util::replace_vars(v, &ctx.env);
            ctx.env.insert(k.clone(), v);
        }
    }
}

impl Drop for Prompt {
    fn drop(&mut self) {
        self.save_history();
    }
}

#[derive(Debug)]
pub struct EofError;

impl Error for EofError {}

impl fmt::Display for EofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write newline on ^D/EOF so next prompt doesn't appear on same line.
        writeln!(f, "")
    }
}

#[derive(Debug)]
struct NoCommandError;

impl Error for NoCommandError {}

impl fmt::Display for NoCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use command::cd_command::CdCommand;
    use command::general_command::GeneralCommand;
    use config::Config;
    use context;

    macro_rules! create_test_prompt {
        ($p:ident) => {
            let context = context::default();
            let editor = editor::create(context.clone());
            let mut $p = Prompt { context, editor };
        };
    }

    macro_rules! create_test_prompt_with_config {
        ($p:ident, $cfg:expr) => {
            let context = context::default();
            context.borrow_mut().config = $cfg;
            let editor = editor::create(context.clone());
            let mut $p = Prompt { context, editor };
        };
    }

    #[test]
    fn parse_command_empty() {
        create_test_prompt!(prompt);
        let cmd = prompt.parse_command(String::new());
        assert!(cmd.is_err());
        assert!(cmd.err().unwrap().is::<NoCommandError>());
    }

    #[test]
    /// They should yield the same in this case.
    fn parse_command_calls_command_parse() {
        create_test_prompt!(prompt);

        let cmd = prompt.parse_command("ls -l".to_string());
        assert!(cmd.is_ok());
        assert!(
            cmd.unwrap()
                .as_any()
                .downcast_ref::<GeneralCommand>()
                .is_some()
        );
    }

    #[test]
    fn parse_command_auto_cd() {
        // auto-cd is enabled per default in Config.
        create_test_prompt!(prompt);
        let cmd = prompt.parse_command(".".to_string());
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let cd_cmd = cmd.as_any().downcast_ref::<CdCommand>().unwrap();
        assert_eq!(cd_cmd.path, ".");
    }

    #[test]
    fn parse_command_env_vars_replaced() {
        create_test_prompt!(prompt);
        prompt
            .context
            .borrow_mut()
            .env
            .insert("HELLO".to_string(), "WORLD".to_string());
        let cmd = prompt.parse_command("echo $HELLO".to_string());
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "echo".to_string());
        assert_eq!(general_cmd.args, vec!["WORLD".to_string()]);
    }

    #[test]
    fn parse_command_alias_substituted() {
        let mut config = Config::default();
        config.aliases.insert("l".to_string(), "ls -l".to_string());
        create_test_prompt_with_config!(prompt, config);

        let cmd = prompt.parse_command("l -F".to_string());
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "ls".to_string());
        assert_eq!(general_cmd.args, vec!["-l".to_string(), "-F".to_string()]);
    }
}
