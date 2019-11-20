use crate::command::{self, Command};
use crate::context::Context;
use crate::editor::{self, EditorHelper};
use crate::util;

use libc;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;
use std::io::Write;
use std::path::Path;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use rustyline::error::ReadlineError;
use rustyline::Editor;

/// Fallback textual prompt if term formatting fails.
const SAFE_PROMPT: &str = "carapace % ";

/// Shell root user id
const UID_ROOT: u32 = 0;

pub type PromptResult = Result<Box<dyn Command>, Box<dyn Error>>;

/// Controls showing the prompt and yielding lines from stdin.
pub struct Prompt {
    /// General context of the shell.
    pub context: Context,

    /// Readline interface.
    pub editor: Editor<EditorHelper>,

    /// Environment values to be restored before next command due to inline env vars.
    restore_env: HashMap<String, String>,

    /// Environment keys to be deleted before next command due to inline env vars.
    delete_env: HashSet<String>,
}

impl Prompt {
    pub fn new(context: Context) -> Prompt {
        let editor = editor::create(&context.clone());
        let mut p = Prompt {
            context,
            editor,
            restore_env: HashMap::new(),
            delete_env: HashSet::new(),
        };
        p.load_history();
        p.load_env();
        p
    }

    /// Shows prompt and reads command and arguments from stdin.
    pub fn show_parse_command(&mut self) -> PromptResult {
        let prompt_txt = self.prompt();

        let input = self.editor.readline(prompt_txt.as_ref());
        match input {
            Ok(line) => self.parse_command(&line),
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
    pub fn parse_command(&mut self, input: &str) -> PromptResult {
        self.restore_env();
        self.editor.add_history_entry(input);

        let mut input = input.trim().to_string();
        if input.is_empty() {
            return Err(Box::new(NoCommandError));
        }

        if self.context.borrow().verbose > 0 {
            println!("{}", input);
        }

        // Replace all `$VAR` and `${VAR}` occurrences with values from environment.
        input = util::replace_vars(&input, &self.context.borrow().env);

        let mut values: Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();

        // Detect any temporary, inline env vars, like "A=42 ./prog" etc. Also replace any use of
        // the inline env vars in the current input. And remember which env vars to remove and old
        // values to replace them with for next command.
        let mut abort_inline = false;
        values = values
            .into_iter()
            .filter_map(|v| {
                let mut ctx = self.context.borrow_mut();
                if abort_inline {
                    return Some(util::replace_vars(&v, &ctx.env));
                }
                if let Some(pos) = v.find('=') {
                    let (k, val) = (v[..pos].to_string(), v[pos + 1..].to_string());
                    if ctx.env.contains_key(&k) {
                        self.restore_env.insert(k.clone(), ctx.env[&k].clone());
                    } else {
                        self.delete_env.insert(k.clone());
                    }
                    ctx.env.insert(k, val);
                    None
                } else {
                    // Stop looking for inline env vars at first command so env to be permanently
                    // exported aren't replaced. For instance, "B=2" must still be exported in "A=1
                    // export B=2".
                    abort_inline = true;
                    Some(util::replace_vars(&v, &ctx.env))
                }
            })
            .collect();

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
                if !x.starts_with('~') {
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
            })
            .collect();

        // Replace all file globs, like "C*" -> ["Cargo.lock", "Cargo.toml"].
        let mut expanded_values = Vec::new();
        for v in &values {
            if v.contains('*') {
                expanded_values.append(&mut util::expand_glob(v));
            } else {
                expanded_values.push(v.to_string());
            }
        }

        let mut program = expanded_values[0].clone();
        let mut args: Vec<String> = expanded_values.drain(1..).collect();

        // If input is an existing folder, and auto_cd is enabled, then set "cd" as the
        // program.
        if self.context.borrow().config.auto_cd
            && expanded_values.len() == 1
            && Path::new(&values[0]).is_dir()
        {
            args = vec![program];
            program = "cd".to_string();
        }

        Ok(command::parse(program, args))
    }

    /// Check if any env vars must be replaced/deleted due to inline env vars from last command.
    fn restore_env(&mut self) {
        let mut ctx = self.context.borrow_mut();

        for k in &self.delete_env {
            ctx.env.remove(k.as_str());
        }
        self.delete_env.clear();

        for (k, v) in &self.restore_env {
            ctx.env.insert(k.clone(), v.clone());
        }
        self.restore_env.clear();
    }

    /// Yields the textual prompt with term colors.
    fn prompt(&self) -> String {
        // In case of failure, use safe prompt. It is a closure so it is only allocated if it is
        // needed.
        let safe_prompt = || SAFE_PROMPT.to_string();

        let bufwtr = BufferWriter::stderr(ColorChoice::Always);
        let mut buffer = bufwtr.buffer();
        let mut color = ColorSpec::new();
        let mut bright_color = ColorSpec::new();
        bright_color.set_intense(true);

        // Create textual prompt.
        if buffer.set_color(color.set_fg(Some(Color::Green))).is_err() {
            return safe_prompt();
        }
        if write!(&mut buffer, "carapace").is_err() {
            println!("Failed to write to term!");
        }

        if let Ok(cwd) = env::current_dir() {
            if buffer
                .set_color(bright_color.set_fg(Some(Color::Blue)))
                .is_err()
            {
                return safe_prompt();
            }
            if write!(&mut buffer, " {}", cwd.display()).is_err() {
                println!("Failed to write to term!");
            }
        }

        if buffer.set_color(color.set_fg(Some(Color::Green))).is_err() {
            return safe_prompt();
        }

        let uid_ch = if UID_ROOT == unsafe { libc::geteuid() } {
            '#'
        } else {
            '%'
        };
        if write!(&mut buffer, " {} ", uid_ch).is_err() {
            println!("Failed to write to term!");
        }

        // Reset prompt color to white so colors don't flow into the cursor and
        // user commands.
        if buffer.set_color(color.set_fg(Some(Color::White))).is_err() {
            return safe_prompt();
        }

        String::from_utf8_lossy(&buffer.into_inner()).into_owned()
    }

    fn load_history(&mut self) {
        let path = dirs::home_dir().unwrap().join(".carapace").join("history");
        if self.editor.load_history(&path).is_err() {
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
        writeln!(f)
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

    use crate::command::cd_command::CdCommand;
    use crate::command::general_command::GeneralCommand;
    use crate::config::Config;
    use crate::context;

    macro_rules! create_test_prompt {
        ($p:ident) => {
            let context = context::default();
            let editor = editor::create(&context.clone());
            let mut $p = Prompt {
                context,
                editor,
                restore_env: HashMap::new(),
                delete_env: HashSet::new(),
            };
        };
    }

    macro_rules! create_test_prompt_with_config {
        ($p:ident, $cfg:expr) => {
            let context = context::default();
            context.borrow_mut().config = $cfg;
            let editor = editor::create(&context.clone());
            let mut $p = Prompt {
                context,
                editor,
                restore_env: HashMap::new(),
                delete_env: HashSet::new(),
            };
        };
    }

    #[test]
    fn parse_command_empty() {
        create_test_prompt!(prompt);
        let cmd = prompt.parse_command(&String::new());
        assert!(cmd.is_err());
        assert!(cmd.err().unwrap().is::<NoCommandError>());
    }

    #[test]
    /// They should yield the same in this case.
    fn parse_command_calls_command_parse() {
        create_test_prompt!(prompt);

        let cmd = prompt.parse_command("ls -l");
        assert!(cmd.is_ok());
        assert!(cmd
            .unwrap()
            .as_any()
            .downcast_ref::<GeneralCommand>()
            .is_some());
    }

    #[test]
    fn parse_command_auto_cd() {
        // auto-cd is enabled per default in Config.
        create_test_prompt!(prompt);
        let cmd = prompt.parse_command(".");
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
        let cmd = prompt.parse_command("echo $HELLO");
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

        let cmd = prompt.parse_command("l -F");
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "ls".to_string());
        assert_eq!(general_cmd.args, vec!["-l".to_string(), "-F".to_string()]);
    }

    #[test]
    fn parse_command_inline_env_vars() {
        create_test_prompt!(prompt);

        let cmd = prompt.parse_command("A=1 echo test");
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "echo".to_string());
        assert_eq!(general_cmd.args, vec!["test".to_string()]);

        assert!(prompt.delete_env.contains("A"));
    }

    #[test]
    fn parse_command_inline_env_vars_replaced_for_invocation() {
        create_test_prompt!(prompt);

        let cmd = prompt.parse_command("A=1 echo $A");
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "echo".to_string());
        assert_eq!(general_cmd.args, vec!["1".to_string()]);

        assert!(prompt.delete_env.contains("A"));
    }

    #[test]
    fn parse_command_inline_env_vars_replaces_session_env() {
        create_test_prompt!(prompt);
        prompt
            .context
            .borrow_mut()
            .env
            .insert("A".to_string(), "42".to_string());

        let cmd = prompt.parse_command("A=1 echo $A");
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "echo".to_string());

        // $A is replaced with "42" before the inline replacement since it already exists in the
        // environment.
        assert_eq!(general_cmd.args, vec!["42".to_string()]);

        assert!(!prompt.delete_env.contains("A"));
        assert!(prompt.restore_env.contains_key("A"));
        assert_eq!(prompt.restore_env.get("A"), Some(&"42".to_string()));
        assert_eq!(prompt.context.borrow().env.get("A"), Some(&"1".to_string()));
    }

    #[test]
    fn parse_command_inline_env_vars_restored_before_next_command() {
        create_test_prompt!(prompt);
        prompt
            .context
            .borrow_mut()
            .env
            .insert("A".to_string(), "42".to_string());

        let cmd = prompt.parse_command("A=1 echo $A");
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        let general_cmd = cmd.as_any().downcast_ref::<GeneralCommand>().unwrap();
        assert_eq!(general_cmd.program, "echo".to_string());

        // $A is replaced with "42" before the inline replacement since it already exists in the
        // environment.
        assert_eq!(general_cmd.args, vec!["42".to_string()]);

        assert!(!prompt.delete_env.contains("A"));
        assert!(prompt.restore_env.contains_key("A"));
        assert_eq!(prompt.restore_env.get("A"), Some(&"42".to_string()));
        assert_eq!(prompt.context.borrow().env.get("A"), Some(&"1".to_string()));

        // Attempt next command to make sure env is cleaned up.
        let _cmd = prompt.parse_command("");

        assert!(!prompt.restore_env.contains_key("A"));
        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("A"));
        assert_eq!(ctx.env.get("A"), Some(&"42".to_string()));
    }
}
