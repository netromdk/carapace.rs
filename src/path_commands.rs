use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

use is_executable::IsExecutable;

type Value = String;
type Container = BTreeSet<Value>;

#[derive(Default)]
pub struct PathCommands {
    commands: Container,
}

impl PathCommands {
    /// Create new instance of PathCommands and rehash from $PATH.
    pub fn new() -> PathCommands {
        let mut pc = PathCommands::default();
        pc.rehash();
        pc
    }

    /// Finds all executable programs in $PATH and adds the base file names to the internal set.
    pub fn rehash(&mut self) {
        self.clear();

        if let Ok(value) = env::var("PATH") {
            let dirs: Vec<&str> = value.split(':').filter(|x| !x.is_empty()).collect();
            for dir in dirs {
                let path = Path::new(dir);
                if !path.exists() || !path.is_dir() {
                    continue;
                }

                // Find executable files at the top-level of the directory.
                if let Ok(rd) = fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.is_executable() {
                            if let Some(file_name) = path.file_name().unwrap().to_str() {
                                self.insert(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn insert(&mut self, value: Value) {
        self.commands.insert(value);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn contains<S>(&self, value: &S) -> bool
    where
        S: ?Sized + Ord,
        Value: Borrow<S>,
    {
        self.commands.contains(value)
    }
}

impl AsRef<Container> for PathCommands {
    fn as_ref(&self) -> &Container {
        &self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_default() {
        let pc = PathCommands::default();
        assert!(pc.is_empty());
    }

    #[test]
    fn len() {
        let mut pc = PathCommands::default();
        assert_eq!(0, pc.len());

        pc.insert("foo".to_string());
        assert_eq!(1, pc.len());
    }

    #[test]
    fn is_empty() {
        let mut pc = PathCommands::default();
        assert!(pc.is_empty());

        pc.insert("foo".to_string());
        assert!(!pc.is_empty());
    }

    #[test]
    fn insert() {
        let mut pc = PathCommands::default();
        assert_eq!(0, pc.len());

        pc.insert("foo".to_string());
        assert_eq!(1, pc.len());

        pc.insert("bar".to_string());
        pc.insert("baz".to_string());
        assert_eq!(3, pc.len());
    }

    #[test]
    fn clear() {
        let mut pc = PathCommands::default();
        assert!(pc.is_empty());

        pc.insert("foo".to_string());
        pc.insert("bar".to_string());
        pc.insert("baz".to_string());
        assert_eq!(3, pc.len());

        pc.clear();
        assert!(pc.is_empty());
    }

    #[test]
    fn contains() {
        let mut pc = PathCommands::default();
        assert!(!pc.contains("foo"));
        pc.insert("foo".to_string());
        assert!(pc.contains("foo"));
    }
}
