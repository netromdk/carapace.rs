use super::*;

/// Unset command removes variables from environment.
pub struct UnsetCommand {
    vars: Vec<String>,
}

impl UnsetCommand {
    pub fn new(args: Vec<String>) -> Result<UnsetCommand, Box<dyn Error>> {
        if args.len() == 0 {
            return Err(Box::new(CommandError::new("Not enough arguments")));
        }
        Ok(UnsetCommand { vars: args })
    }
}

impl Command for UnsetCommand {
    fn execute(&self, prompt: &mut Prompt) -> Result<bool, i32> {
        for var in &self.vars {
            prompt.env.remove(var);
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
