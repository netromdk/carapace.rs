use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process;

fn set_cwd(dir: &Path) {
    if let Err(err) = env::set_current_dir(dir) {
        println!("Could not change to {}: {}", dir.display(), err);
    }
}

#[derive(Debug)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

impl Command {
    /// Parses a vector of strings into a `Command` with command program and arguments.
    pub fn new(values: Vec<&str>) -> Result<Command, Box<dyn Error>> {
        if values.len() == 0 {
            return Err(Box::new(NoCommandError));
        }

        Ok(Command {
            program: values[0].to_string(),
            args: values[1..].iter().map(|x| x.to_string()).collect(),
        })
    }

    /// Execute command and return `Ok(true)` if command was run successfully, `Ok(false)` if not,
    /// and `Err(exit_code)` on "exit" or "quit".
    pub fn execute(&self) -> Result<bool, i32> {
        match self.program.as_ref() {
            "exit" | "quit" => Err(0),

            "cd" => {
                let path = if self.args.len() > 0 {
                    Path::new(&self.args[0])
                } else {
                    Path::new("~")
                };

                let home_dir = dirs::home_dir().unwrap_or_default();
                if path.starts_with("~") {
                    set_cwd(&home_dir.join(path.strip_prefix("~").unwrap()));
                } else {
                    set_cwd(path);
                }
                Ok(true)
            }

            _ => {
                // Run command with arguments and wait for it to finish.
                let output = process::Command::new(&self.program)
                    .args(&self.args)
                    .output();
                match output {
                    Ok(output) => {
                        print!("{}", String::from_utf8_lossy(&output.stdout));
                        Ok(true)
                    }
                    Err(err) => {
                        println!("{}", err);
                        Ok(false)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct NoCommandError;

impl Error for NoCommandError {}

impl fmt::Display for NoCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No command inputted.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_command() {
        let cmd = Command::new(vec![]);
        assert!(cmd.is_err());
    }

    #[test]
    fn parse_command() {
        let cmd = Command::new(vec!["one"]).unwrap();
        assert_eq!(cmd.program, "one");
    }

    #[test]
    fn parse_command_one_arg() {
        let cmd = Command::new(vec!["one", "two"]).unwrap();
        assert_eq!(cmd.program, "one");
        assert_eq!(cmd.args, vec!["two"]);
    }

    #[test]
    fn parse_command_two_args() {
        let cmd = Command::new(vec!["one", "two", "three"]).unwrap();
        assert_eq!(cmd.program, "one");
        assert_eq!(cmd.args, vec!["two", "three"]);
    }
}
