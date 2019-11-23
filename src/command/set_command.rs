use super::*;

use clap::{App, AppSettings, Arg};

use rustyline::config::Configurer;
use rustyline::EditMode;

/// Set command manipulates shell options.
pub struct SetCommand {
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl SetCommand {
    pub fn new(args: Vec<String>) -> SetCommand {
        SetCommand {
            args,
            app: App::new("set")
                .about("Set or unset shell options.")
                .after_help("Options currently set can be displayed via environment variable $-.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(
                    Arg::with_name("xtrace")
                        .short("x")
                        .help("Prints commands and their arguments when executed."),
                )
                .arg(
                    Arg::with_name("errexit")
                        .short("e")
                        .help("Exit shell if a command yields non-zero exit code."),
                )
                .arg(Arg::with_name("verbose").short("v").multiple(true).help(
                    "Sets verbosity level. Can be used multiple times, like '-v -v -v' or '-vvv' \
                     for a verbosity level of 3. With >=1 the shell prints input lines as they \
                     are read.",
                ))
                .arg(
                    Arg::with_name("option")
                        .short("o")
                        .long("option")
                        .takes_value(true)
                        .value_name("name")
                        .help(
                            "Sets option given option name:\n\
                             xtrace   equivalent to -x\n\
                             errexit  equivalent to -e\n\
                             verbose  equivalent to -v (verbose level 1)\n\
                             \n\
                             emacs    edit mode\n\
                             vi       edit mode",
                        ),
                )
                .arg(
                    Arg::with_name("unset")
                        .value_name("+NAME")
                        .help("Unsets option NAME, like '+x' to unset xtrace option."),
                ),
        }
    }

    /// Set or unset options by adding or removing from `$-` in environment.
    fn set(&mut self, opt: &str, enable: bool, prompt: &mut Prompt) -> Result<bool, i32> {
        match opt {
            "x" | "e" => {
                let mut ctx = prompt.context.borrow_mut();

                let env = &mut ctx.env;
                if !env.contains_key("-") {
                    env.insert(String::from("-"), String::from(""));
                }

                // Add or remove the option from $-.
                if enable {
                    let old_value = env[&String::from("-")].clone();
                    if !old_value.contains(opt) {
                        env.insert(String::from("-"), old_value + opt);
                    }
                } else {
                    let old_value = env[&String::from("-")].clone();
                    if old_value.contains(opt) {
                        env.insert(String::from("-"), old_value.replace(opt, ""));
                    }
                }

                if opt == "x" {
                    ctx.xtrace = enable;
                } else if opt == "e" {
                    ctx.errexit = enable;
                }
            }
            "v" => {
                prompt.context.borrow_mut().verbose = if enable { 1 } else { 0 };
            }
            _ => {
                println!("Unknown option: {}", opt);
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Command for SetCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }
        // TODO: find better way to unwrap matches without writing like this..
        let m = matches.unwrap();

        // -x
        if m.is_present("xtrace") {
            return self.set("x", true, prompt);
        }
        // -e
        else if m.is_present("errexit") {
            return self.set("e", true, prompt);
        }
        // -v..
        else if m.is_present("verbose") {
            let level = m.occurrences_of("verbose");
            prompt.context.borrow_mut().verbose = level;
            return Ok(true);
        }
        // -o <name>
        else if let Some(opt) = m.value_of("option") {
            let opt = match opt {
                "xtrace" => "x",
                "errexit" => "e",
                "verbose" => "v",
                "emacs" => {
                    prompt.editor.set_edit_mode(EditMode::Emacs);
                    return Ok(true);
                }
                "vi" => {
                    prompt.editor.set_edit_mode(EditMode::Vi);
                    return Ok(true);
                }
                _ => {
                    println!("Unknown option name: {}", opt);
                    return Ok(false);
                }
            };
            return self.set(opt, true, prompt);
        }
        // +<name>
        else if let Some(opt) = m.value_of("unset") {
            if !opt.starts_with('+') || opt.len() == 1 {
                println!(
                    "Argument to unset must start with '+' with a non-empty string following, \
                     Like '+x'."
                );
                return Ok(false);
            }
            let opt = opt.get(1..).unwrap();
            return self.set(opt, false, prompt);
        }

        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context;

    #[test]
    fn new() {
        let args = vec![String::from("arg")];
        let cmd = SetCommand::new(args.clone());
        assert_eq!(cmd.args, args);
    }

    #[test]
    fn invalid_arg_single_plus() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["+".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    #[test]
    fn invalid_arg_not_starting_with_plus() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["x".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    #[test]
    fn unknown_option_name() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["-o".to_string(), "foobarbaz".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    #[test]
    fn unknown_option() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["-q".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    #[test]
    fn set_x() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["-x".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "x");
        assert!(ctx.xtrace);
    }

    #[test]
    fn set_xtrace() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "xtrace".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "x");
        assert!(ctx.xtrace);
    }

    #[test]
    fn set_xtrace_long() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["--option".to_string(), "xtrace".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "x");
        assert!(ctx.xtrace);
    }

    #[test]
    fn unset_x() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        {
            let mut cmd = SetCommand::new(vec!["-x".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "x");
            assert!(ctx.xtrace);
        }

        {
            let mut cmd = SetCommand::new(vec!["+x".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "");
            assert!(!ctx.xtrace);
        }
    }

    #[test]
    fn set_e() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["-e".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "e");
        assert!(ctx.errexit);
    }

    #[test]
    fn set_errexit() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "errexit".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "e");
        assert!(ctx.errexit);
    }

    #[test]
    fn set_errexit_long() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        let mut cmd = SetCommand::new(vec!["--option".to_string(), "errexit".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "e");
        assert!(ctx.errexit);
    }

    #[test]
    fn unset_e() {
        let mut prompt = Prompt::create(context::default());
        assert!(!prompt.context.borrow().env.contains_key("-"));

        {
            let mut cmd = SetCommand::new(vec!["-e".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "e");
            assert!(ctx.errexit);
        }

        {
            let mut cmd = SetCommand::new(vec!["+e".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "");
            assert!(!ctx.errexit);
        }
    }

    #[test]
    fn set_emacs() {
        let mut prompt = Prompt::create(context::default());

        // Set it to something else to ensure it gets set.
        prompt.editor.set_edit_mode(EditMode::Vi);

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "emacs".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        assert_eq!(EditMode::Emacs, prompt.editor.config_mut().edit_mode());
    }

    #[test]
    fn set_vi() {
        let mut prompt = Prompt::create(context::default());

        // Set it to something else to ensure it gets set.
        prompt.editor.set_edit_mode(EditMode::Emacs);

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "vi".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        assert_eq!(EditMode::Vi, prompt.editor.config_mut().edit_mode());
    }

    #[test]
    fn set_v() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().verbose = 0;

        {
            let mut cmd = SetCommand::new(vec!["-v".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert_eq!(ctx.verbose, 1);
        }

        {
            let mut cmd = SetCommand::new(vec!["-v".to_string(), "-v".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert_eq!(ctx.verbose, 2);
        }

        {
            let mut cmd =
                SetCommand::new(vec!["-v".to_string(), "-v".to_string(), "-v".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert_eq!(ctx.verbose, 3);
        }
    }

    #[test]
    fn set_verbose() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().verbose = 0;

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "verbose".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert_eq!(ctx.verbose, 1);
    }

    #[test]
    fn unset_v() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().verbose = 1;

        let mut cmd = SetCommand::new(vec!["+v".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert_eq!(ctx.verbose, 0);
    }
}
