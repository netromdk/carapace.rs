use super::*;

/// Quit command provides an exit code of zeroo on execution.
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self, _prompt: &mut Prompt) -> Result<bool, i32> {
        Err(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
