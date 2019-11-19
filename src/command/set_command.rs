use super::*;

use clap::{App, AppSettings, Arg};

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
                    Arg::with_name("option")
                        .short("o")
                        .long("option")
                        .takes_value(true)
                        .value_name("name")
                        .help(
                            "Sets option given option name:\n\
                             xtrace   equivalent to -x",
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
            "x" => {
                let env = &mut prompt.context.borrow_mut().env;
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

                // TODO: when xtrace is enabled then use it to display commands and arguments..
            }
            _ => {
                println!("Unknown option: {}", opt);
                return Ok(false);
            }
        }
        return Ok(true);
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
        // -o <name>
        else if let Some(opt) = m.value_of("option") {
            let opt = match opt {
                "xtrace" => "x",
                _ => {
                    println!("Unknown option name: {}", opt);
                    return Ok(false);
                }
            };
            return self.set(opt, true, prompt);
        }
        // +<name>
        else if let Some(opt) = m.value_of("unset") {
            if !opt.starts_with("+") {
                println!("Argument to unset must start with '+', Like '+x'.");
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
