use super::*;

/// General command that executes program with arguments and waits for it to finish.
pub struct GeneralCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl GeneralCommand {
    pub fn new(program: String, args: Vec<String>) -> GeneralCommand {
        GeneralCommand { program, args }
    }
}

impl Command for GeneralCommand {
    fn execute(&self, _prompt: &Prompt) -> Result<bool, i32> {
        let output = process::Command::new(&self.program)
            .args(&self.args)
            .output();
        match output {
            Ok(output) => {
                let mut txt = String::from_utf8_lossy(&output.stdout);
                if !txt.ends_with("\n") {
                    txt += "\n";
                }
                print!("{}", txt);
                Ok(true)
            }
            Err(err) => {
                println!("{}", err);
                Ok(false)
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let prog = String::from("prog");
        let args = vec![String::from("arg")];
        let cmd = GeneralCommand::new(prog.clone(), args.clone());
        assert_eq!(cmd.program, prog);
        assert_eq!(cmd.args, args);
    }
}
