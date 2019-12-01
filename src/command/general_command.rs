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

        // Spawn child process and inherit stdout/stderr so it is displayed within carapace,
        // including term colors.
        let proc = process::Command::new(&self.program)
            .args(&self.args)
            .env_clear()
            .envs(ctx.env.as_ref())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();

        match proc {
            Ok(mut child) => {
                // Wait for child process to exit.
                if let Ok(status) = child.wait() {
                    // Update $? with exit code.
                    let code = status.code().unwrap_or(0);
                    ctx.env.insert("?".to_string(), code.to_string());

                    // Exit immediately if errexit option enabled.
                    let success = status.success();
                    if ctx.errexit && !success {
                        return Err(code);
                    } else {
                        return Ok(success);
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
                if ctx.errexit {
                    return Err(1);
                }
            }
        }

        // Program could not be started.
        Ok(false)
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
