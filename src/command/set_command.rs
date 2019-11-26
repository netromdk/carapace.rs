use super::*;

use crate::util::{append_value_for_key, replace_value_for_key};

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
                .after_help(
                    r#"ENVIRONMENT:

  Options currently set can be displayed via environment variable $-.
  Note that it only applies to options with a shorthand form, like 'x' for xtrace.

EXAMPLES:

  Set xtrace option:
    set -x
    set -o xtrace
    set --option xtrace

  Set Emacs edit mode:
    set -o emacs
    set --option emacs

  Unset errexit mode:
    set +e
    set +o errexit
    set +option errexit"#,
                )
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
                            r#"Sets option given option name:
  xtrace     equivalent to -x
  errexit    equivalent to -e
  verbose    equivalent to -v (verbose level 1)

  emacs      edit mode
  vi         edit mode
  ignoreeof  Don't exit shell when reading EOF"#,
                        ),
                )
                .arg(Arg::with_name("unset").value_name("+NAME").help(
                    "Unsets option NAME, like '+x' to unset xtrace option. Can also be \
                     used via '+option <name>'",
                ))
                .arg(
                    // Used in conjunction with "unset" argument in the <name> case of `+o <name>`
                    // and `+option <name>`.
                    Arg::with_name("unset-name").hidden(true),
                ),
        }
    }

    /// Set or unset options by adding or removing from `$-` in environment.
    fn set(&mut self, opt: &str, enable: bool, prompt: &mut Prompt) -> Result<bool, i32> {
        match opt {
            "x" | "e" | "v" => {
                let mut ctx = prompt.context.borrow_mut();

                // Add or remove the option from $-.
                let env = &mut ctx.env;
                if enable {
                    append_value_for_key(opt, "-", env);
                } else {
                    replace_value_for_key(opt, "", "-", env);
                }

                if opt == "x" {
                    ctx.xtrace = enable;
                } else if opt == "e" {
                    ctx.errexit = enable;
                } else if opt == "v" {
                    ctx.verbose = if enable { 1 } else { 0 };
                }
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
            let mut ctx = prompt.context.borrow_mut();
            append_value_for_key("v", "-", &mut ctx.env);

            let level = m.occurrences_of("verbose");
            ctx.verbose = level;
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
                "ignoreeof" => {
                    prompt.context.borrow_mut().ignoreeof = true;
                    return Ok(true);
                }
                _ => {
                    println!("Unknown option name: {}", opt);
                    return Ok(false);
                }
            };
            return self.set(opt, true, prompt);
        }
        // +<name> or +o/+option <name>
        else if let Some(opt) = m.value_of("unset") {
            if opt == "+o" || opt == "+option" {
                if let Some(opt_name) = m.value_of("unset-name") {
                    let opt = match opt_name {
                        "xtrace" => "x",
                        "errexit" => "e",
                        "verbose" => "v",
                        "emacs" | "vi" => {
                            println!(
                                "Cannot unset {} edit mode! Choice must be set explicitly.",
                                opt_name
                            );
                            return Ok(false);
                        }

                        "ignoreeof" => {
                            prompt.context.borrow_mut().ignoreeof = false;
                            return Ok(true);
                        }
                        _ => {
                            println!("Unknown option name: {}", opt_name);
                            return Ok(false);
                        }
                    };
                    return self.set(opt, false, prompt);
                } else {
                    println!("Option name required after {}!", opt);
                    return Ok(false);
                }
            } else {
                // +<option>
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
    fn unset_xtrace() {
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
            let mut cmd = SetCommand::new(vec!["+option".to_string(), "xtrace".to_string()]);
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
    fn unset_errexit() {
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
            let mut cmd = SetCommand::new(vec!["+option".to_string(), "errexit".to_string()]);
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
    fn no_unset_emacs() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["+o".to_string(), "emacs".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert_eq!(false, res.unwrap());
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
    fn no_unset_vi() {
        let mut prompt = Prompt::create(context::default());
        let mut cmd = SetCommand::new(vec!["+o".to_string(), "vi".to_string()]);
        let res = cmd.execute(&mut prompt);
        assert!(res.is_ok());
        assert_eq!(false, res.unwrap());
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
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "v");
        }

        {
            let mut cmd = SetCommand::new(vec!["-v".to_string(), "-v".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert_eq!(ctx.verbose, 2);
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "v");
        }

        {
            let mut cmd =
                SetCommand::new(vec!["-v".to_string(), "-v".to_string(), "-v".to_string()]);
            assert!(cmd.execute(&mut prompt).is_ok());

            let ctx = prompt.context.borrow();
            assert_eq!(ctx.verbose, 3);
            assert!(ctx.env.contains_key("-"));
            assert_eq!(ctx.env["-"], "v");
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
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "v");
    }

    #[test]
    fn unset_v() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().verbose = 1;

        let mut cmd = SetCommand::new(vec!["+v".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert_eq!(ctx.verbose, 0);
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "");
    }

    #[test]
    fn unset_verbose() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().verbose = 1;

        let mut cmd = SetCommand::new(vec!["+option".to_string(), "verbose".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert_eq!(ctx.verbose, 0);
        assert!(ctx.env.contains_key("-"));
        assert_eq!(ctx.env["-"], "");
    }

    #[test]
    fn set_ignoreeof() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().ignoreeof = false;

        let mut cmd = SetCommand::new(vec!["-o".to_string(), "ignoreeof".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(ctx.ignoreeof);
    }

    #[test]
    fn unset_ignoreeof() {
        let mut prompt = Prompt::create(context::default());
        prompt.context.borrow_mut().ignoreeof = true;

        let mut cmd = SetCommand::new(vec!["+o".to_string(), "ignoreeof".to_string()]);
        assert!(cmd.execute(&mut prompt).is_ok());

        let ctx = prompt.context.borrow();
        assert!(!ctx.ignoreeof);
    }
}
