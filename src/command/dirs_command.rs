use super::*;

use clap::{App, AppSettings, Arg};

/// Dirs command shows the directory stack.
pub struct DirsCommand {
    args: Vec<String>,
    app: App<'static, 'static>,
}

impl DirsCommand {
    pub fn new(args: Vec<String>) -> DirsCommand {
        DirsCommand {
            args,
            app: App::new("dirs")
                .about("Display directory stack.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .help("Verbose mode shows directory stack in list form."),
                ),
        }
    }
}

impl Command for DirsCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.args);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        let m = matches.unwrap();

        let ctx = prompt.context.borrow();
        if ctx.dir_stack.is_empty() {
            println!("Directory stack is empty");
        } else {
            let verbose = m.is_present("verbose");
            let short = !verbose;
            ctx.print_dir_stack(short);
        }

        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for DirsCommand {
    fn aliases() -> Vec<String> {
        vec!["dirs".to_string()]
    }
}
