use super::*;

pub struct RehashCommand;

impl Command for RehashCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        prompt.context.borrow_mut().commands.rehash();
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CommandAliases for RehashCommand {
    fn aliases() -> Vec<String> {
        vec!["rehash".to_string()]
    }
}
