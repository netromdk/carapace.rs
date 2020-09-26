use super::*;

/// Dirs command shows the directory stack.
pub struct DirsCommand;

impl Command for DirsCommand {
    fn execute(&mut self, prompt: &mut Prompt) -> Result<bool, i32> {
        // Show stack in all cases.
        // TODO: display more nicely later on
        let ctx = prompt.context.borrow();
        println!("{:?}", ctx.dir_stack);

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
