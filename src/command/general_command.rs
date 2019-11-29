use super::*;

use std::process::Stdio;

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
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        let mut ctx = prompt.context.borrow_mut();
        let output = process::Command::new(&self.program)
            .args(&self.args)
            .env_clear()
            .envs(ctx.env.as_ref())
            // Inherit stdout/stderr so it is displayed with the shell, including term colors.
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();
        match output {
            Ok(output) => {
                // Update $? with exit code.
                let code = output.status.code().unwrap_or(0);
                ctx.env.insert("?".to_string(), code.to_string());

                // Exit immediately if errexit option enabled.
                let success = output.status.success();
                if ctx.errexit && !success {
                    Err(code)
                } else {
                    Ok(success)
                }
            }
            Err(err) => {
                println!("{}", err);
                if ctx.errexit {
                    Err(1)
                } else {
                    Ok(false)
                }
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
