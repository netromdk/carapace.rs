use super::*;

/// Quit command provides an exit code of zeroo on execution.
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self) -> Result<bool, i32> {
        Err(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_returns_zero() {
        let cmd = QuitCommand {};
        let res = cmd.execute();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), 0);
    }
}
