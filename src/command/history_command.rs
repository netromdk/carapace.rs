use super::*;

use clap::{App, AppSettings, Arg};

/// History command shows the list of inputs.
pub struct HistoryCommand {
    vars: Vec<String>,
    app: App<'static, 'static>,
}

impl HistoryCommand {
    pub fn new(args: Vec<String>) -> HistoryCommand {
        HistoryCommand {
            vars: args,
            app: App::new("history")
                .about("When no options are specified, all history items will be listed.")
                .setting(AppSettings::NoBinaryName)
                .setting(AppSettings::DisableVersion)
                .arg(
                    Arg::with_name("clear")
                        .short("c")
                        .long("clear")
                        .help("Clear current session's history (not what's saved on disk)."),
                )
                .arg(
                    Arg::with_name("write")
                        .short("w")
                        .long("write")
                        .help("Writes history to disk."),
                ),
        }
    }
}

impl Command for HistoryCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let matches = self.app.get_matches_from_safe_borrow(&self.vars);
        if let Err(err) = matches {
            println!("{}", err);
            return Ok(false);
        }

        let matches = matches.unwrap();
        if matches.is_present("clear") {
            prompt.editor.history_mut().clear();
        } else if matches.is_present("write") {
            prompt.save_history();
        } else {
            let mut num = 1;
            for line in prompt.editor.history().iter() {
                println!("{:4}: {}", num, line);
                num += 1;
            }
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
