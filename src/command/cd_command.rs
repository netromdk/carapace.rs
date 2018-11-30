use super::*;

use std::env;
use std::path::Path;

/// Cd command changes directory to defined path.
pub struct CdCommand {
    pub path: String,
}

impl CdCommand {
    /// If no arguments are passed the path will be "~", the home directory, otherwise it will be
    /// the first argument.
    pub fn new(args: Vec<String>) -> CdCommand {
        let path = if args.len() > 0 {
            args[0].clone()
        } else {
            String::from("~")
        };
        CdCommand { path }
    }

    fn set_cwd(&self, dir: &Path) {
        if let Err(err) = env::set_current_dir(dir) {
            println!("Could not change to {}: {}", dir.display(), err);
        }
    }
}

impl Command for CdCommand {
    fn execute(&self) -> Result<bool, i32> {
        let home_dir = dirs::home_dir().unwrap_or_default();
        let path = Path::new(&self.path);
        if path.starts_with("~") {
            self.set_cwd(&home_dir.join(path.strip_prefix("~").unwrap()));
        } else {
            self.set_cwd(path);
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

    #[test]
    fn no_args_is_tilde() {
        let cmd = CdCommand::new(vec![]);
        assert_eq!(cmd.path, "~");
    }

    #[test]
    fn valid_arg() {
        let cmd = CdCommand::new(vec![String::from("/tmp")]);
        assert_eq!(cmd.path, "/tmp");
    }
}
