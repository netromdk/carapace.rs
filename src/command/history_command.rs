use super::*;

/// History command shows the list of inputs.
pub struct HistoryCommand;

impl Command for HistoryCommand {
    fn execute(&self, prompt: &Prompt) -> Result<bool, i32> {
        let mut num = 1;
        for line in prompt.editor.history().iter() {
            println!("{:4}: {}", num, line);
            num += 1;
        }
        Ok(true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
