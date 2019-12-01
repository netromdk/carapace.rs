use super::*;

use clap::{App, AppSettings, Arg};

/// Hash command checks if command is known.
pub struct HashCommand {
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl HashCommand {
    pub fn new(args: Vec<String>) -> HashCommand {
        let app =
            App::new("hash")
                .about("Check command existence or rehash.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(Arg::with_name("rehash").short("r").long("rehash").help(
                    "Detects commands from $PATH from scratch. Is equivalent to running \
                     the 'rehash' command.",
                ))
                .arg(Arg::with_name("command").index(1).help(
                    "Checks if command is known. Exit code is 0 for success and 1 otherwise.",
                ));

        HashCommand { args, app }
    }
}

impl Command for HashCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }
        let m = matches.unwrap();

        let mut ctx = prompt.context.borrow_mut();
        let commands = &mut ctx.commands;

        // -r
        if m.is_present("rehash") {
            commands.rehash();
        }
        // command
        else if let Some(cmd) = m.value_of("command") {
            let success = commands.contains(cmd);

            // Reflect the success in $?.
            ctx.env
                .insert("?".to_string(), if success { 0 } else { 1 }.to_string());

            return Ok(success);
        }

        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for HashCommand {
    fn aliases() -> Vec<String> {
        vec!["hash".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context;

    #[test]
    fn new() {
        let args = vec![String::from("arg")];
        let cmd = HashCommand::new(args.clone());
        assert_eq!(cmd.args, args);
    }

    #[test]
    fn rehash() {
        let ctx = context::default();
        assert!(ctx.borrow().commands.is_empty());

        let mut prompt = Prompt::create(ctx);
        let mut cmd = HashCommand::new(vec!["-r".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.unwrap());
        assert!(!prompt.context.borrow().commands.is_empty());
    }

    #[test]
    fn command_unknown() {
        let ctx = context::default();
        assert!(ctx.borrow().commands.is_empty());

        let mut prompt = Prompt::create(ctx);
        let mut cmd = HashCommand::new(vec!["command".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(!res.unwrap());

        let env = &prompt.context.borrow().env;
        assert!(env.contains_key("?"));
        assert_eq!("1", env["?"]);
    }

    #[test]
    fn command_known() {
        let ctx = context::default();
        ctx.borrow_mut().commands.insert("command".to_string());

        let mut prompt = Prompt::create(ctx);
        let mut cmd = HashCommand::new(vec!["command".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.unwrap());

        let env = &prompt.context.borrow().env;
        assert!(env.contains_key("?"));
        assert_eq!("0", env["?"]);
    }
}
