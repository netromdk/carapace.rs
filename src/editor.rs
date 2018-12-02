use super::*;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{Config, Editor, Helper};

/// Creates `Editor` instance with proper config and completion.
pub fn create(config: &config::Config) -> Editor<EditorHelper> {
    let mut editor = Editor::with_config(
        Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)
            .max_history_size(config.max_history_size)
            .edit_mode(config.edit_mode)
            .completion_type(config.completion_type)
            .build(),
    );

    let h = EditorHelper::new(config);
    editor.set_helper(Some(h));

    editor
}

pub struct EditorHelper<'c> {
    pub config: &'c config::Config,
    pub file_comp: Box<FilenameCompleter>,
}

impl<'c> EditorHelper<'c> {
    pub fn new(config: &'c config::Config) -> EditorHelper {
        EditorHelper {
            config,
            file_comp: Box::new(FilenameCompleter::new()),
        }
    }

    fn builtin_command_completer(
        &self,
        line: &str,
        pos: usize,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut builtins = vec!["cd", "quit", "exit", "h", "hist", "history", "unset"];

        // Add aliases, if any.
        for (k, _) in &self.config.aliases {
            builtins.push(k);
        }

        let mut candidates = Vec::new();

        // Show all candidates with no input and pos=0.
        if pos == 0 {
            for builtin in &builtins {
                candidates.push(Pair {
                    display: builtin.to_string(),
                    replacement: builtin.to_string(),
                });
            }
        }
        // Check for partial matches and their remainders.
        else {
            let slice = &line[..pos];
            for builtin in &builtins {
                if *builtin == slice || builtin.starts_with(slice) {
                    let cmd = builtin.to_string();
                    candidates.push(Pair {
                        display: cmd.clone(),

                        // The missing part of the candidate.
                        replacement: cmd[slice.len()..].to_string(),
                    });
                }
            }
        }

        return Ok((pos, candidates));
    }
}

impl<'c> Completer for EditorHelper<'c> {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Do built-in command completion if position is within first word.
        if let Some(wpos) = line.find(char::is_whitespace) {
            if pos < wpos {
                return self.builtin_command_completer(line, pos);
            }
        } else {
            return self.builtin_command_completer(line, pos);
        }

        // Otherwise, default to file completion.
        self.file_comp.complete(line, pos)
    }
}

impl<'c> Hinter for EditorHelper<'c> {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }
}

impl<'c> Highlighter for EditorHelper<'c> {}
impl<'c> Helper for EditorHelper<'c> {}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! create_test_editor {
        ($e:ident) => {
            let cfg = config::Config::default();
            let $e = create(&cfg);
        };
    }

    #[test]
    fn builtin_complete_no_input_all_candidates() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("", 0)
            .unwrap();
        assert_eq!(pos, 0);
        assert_eq!(pairs.len(), 6);
    }

    #[test]
    fn builtin_complete_quit_cmd() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("q", 1)
            .unwrap();
        assert_eq!(pos, 1);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "quit");
        assert_eq!(&pairs[0].replacement, "uit");
    }

    #[test]
    fn builtin_complete_exit_cmd() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("ex", 2)
            .unwrap();
        assert_eq!(pos, 2);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "exit");
        assert_eq!(&pairs[0].replacement, "it");
    }

    #[test]
    fn builtin_complete_history_cmd_h() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("h", 1)
            .unwrap();
        assert_eq!(pos, 1);
        assert_eq!(pairs.len(), 3);
        assert_eq!(&pairs[0].display, "h");
        assert_eq!(&pairs[0].replacement, "");
        assert_eq!(&pairs[1].display, "hist");
        assert_eq!(&pairs[1].replacement, "ist");
        assert_eq!(&pairs[2].display, "history");
        assert_eq!(&pairs[2].replacement, "istory");
    }

    #[test]
    fn builtin_complete_history_cmd_hist() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("hi", 2)
            .unwrap();
        assert_eq!(pos, 2);
        assert_eq!(pairs.len(), 2);
        assert_eq!(&pairs[0].display, "hist");
        assert_eq!(&pairs[0].replacement, "st");
        assert_eq!(&pairs[1].display, "history");
        assert_eq!(&pairs[1].replacement, "story");
    }

    #[test]
    fn builtin_complete_history_cmd() {
        create_test_editor!(editor);
        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("histo", 5)
            .unwrap();
        assert_eq!(pos, 5);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "history");
        assert_eq!(&pairs[0].replacement, "ry");
    }

    #[test]
    fn builtin_complete_nothing_after_first_whitespace() {
        create_test_editor!(editor);

        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("ls ", 3)
            .unwrap();
        assert_eq!(pos, 3);
        assert_eq!(pairs.len(), 0);

        let (pos, pairs) = editor
            .helper()
            .unwrap()
            .builtin_command_completer("ls -lg /", 8)
            .unwrap();
        assert_eq!(pos, 8);
        assert_eq!(pairs.len(), 0);
    }
}
