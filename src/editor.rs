use super::*;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{Config, Editor, Helper};

use util;

use std::cell::RefCell;
use std::rc::Rc;

/// Creates `Editor` instance with proper config and completion.
pub fn create(context: Rc<RefCell<context::Context>>) -> Editor<EditorHelper> {
    let config = &context.borrow().config;
    let mut editor = Editor::with_config(
        Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)
            .max_history_size(config.max_history_size)
            .edit_mode(config.edit_mode)
            .completion_type(config.completion_type)
            .build(),
    );

    let h = EditorHelper::new(context.clone());
    editor.set_helper(Some(h));

    editor
}

pub struct EditorHelper {
    pub context: Rc<RefCell<context::Context>>,
    pub file_comp: Box<FilenameCompleter>,
}

impl EditorHelper {
    pub fn new(context: Rc<RefCell<context::Context>>) -> EditorHelper {
        EditorHelper {
            context,
            file_comp: Box::new(FilenameCompleter::new()),
        }
    }

    fn builtin_command_completer(&self, line: &str, pos: usize) -> Vec<Pair> {
        let mut builtins: Vec<String> = vec![
            "cd", "exit", "export", "h", "hist", "history", "quit", "set", "unset",
        ].into_iter()
        .map(|x| x.to_string())
        .collect();

        // Add aliases, if any.
        let config = &self.context.borrow().config;
        for (k, _) in &config.aliases {
            builtins.push(k.clone());
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

        return candidates;
    }

    fn env_var_completer(&self, line: &str, pos: usize) -> Vec<Pair> {
        let mut candidates = Vec::new();

        let word = util::partial_env_var_at_pos(pos, line);
        if word.len() == 0 {
            return candidates;
        }

        for (k, _) in &self.context.borrow().env {
            let lookfor = format!("${}", k);
            let lookfor2 = format!("${{{}", k);

            // Look for normal env var: $VAR
            if lookfor.starts_with(&word) {
                candidates.push(Pair {
                    display: lookfor.clone(),
                    replacement: lookfor[word.len()..].to_string(),
                });
            }
            // Look for bracketed env var with no ending bracket: ${VAR
            else if lookfor2.starts_with(&word) && !word.ends_with("}") {
                candidates.push(Pair {
                    display: lookfor2.clone() + "}",
                    replacement: lookfor2[word.len()..].to_string() + "}",
                });
            }
        }

        candidates
    }
}

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Do built-in command completion if position is within first word.
        if util::in_first_word(pos, line) {
            let candidates = self.builtin_command_completer(line, pos);

            // Only return candidates if more than none, otherwise default to file completion so it
            // can be done on the first word also.
            if candidates.len() > 0 {
                return Ok((pos, candidates));
            }
        }

        // Do environment variable completion.
        let candidates = self.env_var_completer(line, pos);
        if candidates.len() > 0 {
            return Ok((pos, candidates));
        }

        // Otherwise, default to file completion.
        self.file_comp.complete(line, pos)
    }
}

impl Hinter for EditorHelper {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }
}

impl Highlighter for EditorHelper {}
impl Helper for EditorHelper {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    macro_rules! create_test_editor {
        ($e:ident) => {
            let ctx = Rc::new(RefCell::new(context::Context::default()));
            let $e = create(ctx);
        };
    }

    macro_rules! create_test_editor_with_env {
        ($e:ident; $map:expr) => {
            let ctx = Rc::new(RefCell::new(context::Context::default()));
            ctx.borrow_mut().env = $map;
            let $e = create(ctx);
        };
    }

    #[test]
    fn builtin_complete_no_input_all_candidates() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("", 0);
        assert_eq!(pairs.len(), 9);
    }

    #[test]
    fn builtin_complete_quit_cmd() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("q", 1);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "quit");
        assert_eq!(&pairs[0].replacement, "uit");
    }

    #[test]
    fn builtin_complete_exit_cmd() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("exi", 3);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "exit");
        assert_eq!(&pairs[0].replacement, "t");
    }

    #[test]
    fn builtin_complete_history_cmd_h() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("h", 1);
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
        let pairs = editor.helper().unwrap().builtin_command_completer("hi", 2);
        assert_eq!(pairs.len(), 2);
        assert_eq!(&pairs[0].display, "hist");
        assert_eq!(&pairs[0].replacement, "st");
        assert_eq!(&pairs[1].display, "history");
        assert_eq!(&pairs[1].replacement, "story");
    }

    #[test]
    fn builtin_complete_history_cmd() {
        create_test_editor!(editor);
        let pairs = editor
            .helper()
            .unwrap()
            .builtin_command_completer("histo", 5);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "history");
        assert_eq!(&pairs[0].replacement, "ry");
    }

    #[test]
    fn builtin_complete_unset_cmd() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("un", 2);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "unset");
        assert_eq!(&pairs[0].replacement, "set");
    }

    #[test]
    fn builtin_complete_export_cmd() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("exp", 3);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "export");
        assert_eq!(&pairs[0].replacement, "ort");
    }

    #[test]
    fn builtin_complete_export_cmd_set() {
        create_test_editor!(editor);
        let pairs = editor.helper().unwrap().builtin_command_completer("s", 1);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "set");
        assert_eq!(&pairs[0].replacement, "et");
    }

    #[test]
    fn builtin_complete_nothing_after_first_whitespace() {
        create_test_editor!(editor);

        let pairs = editor.helper().unwrap().builtin_command_completer("ls ", 3);
        assert_eq!(pairs.len(), 0);

        let pairs = editor
            .helper()
            .unwrap()
            .builtin_command_completer("ls -lg /", 8);
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn env_var_completer_normal_var() {
        let mut env = HashMap::new();
        env.insert("HELLO".to_string(), "WORLD".to_string());
        create_test_editor_with_env!(editor; env);

        let pairs = editor.helper().unwrap().env_var_completer("echo $HE", 8);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "$HELLO");
        assert_eq!(&pairs[0].replacement, "LLO");
    }

    #[test]
    fn env_var_completer_multiple_values() {
        let mut env = HashMap::new();
        env.insert("HELLO".to_string(), "0".to_string());
        env.insert("HEY".to_string(), "1".to_string());
        env.insert("HAND".to_string(), "2".to_string());
        create_test_editor_with_env!(editor; env);

        let pairs = editor.helper().unwrap().env_var_completer("echo $H", 7);
        assert_eq!(pairs.len(), 3);

        let contains = |needle: &Pair| {
            for p in &pairs {
                if p.display == needle.display && p.replacement == needle.replacement {
                    return true;
                }
            }
            return false;
        };

        // Cannot rely on the order of the vector due to the HashMap used internally.
        assert!(contains(&Pair {
            display: "$HELLO".to_string(),
            replacement: "ELLO".to_string(),
        }));
        assert!(contains(&Pair {
            display: "$HEY".to_string(),
            replacement: "EY".to_string(),
        }));
        assert!(contains(&Pair {
            display: "$HAND".to_string(),
            replacement: "AND".to_string(),
        }));
    }

    #[test]
    fn env_var_completer_bracket_var() {
        let mut env = HashMap::new();
        env.insert("HELLO".to_string(), "WORLD".to_string());
        create_test_editor_with_env!(editor; env);

        let pairs = editor.helper().unwrap().env_var_completer("echo ${HE", 9);
        assert_eq!(pairs.len(), 1);
        assert_eq!(&pairs[0].display, "${HELLO}");
        assert_eq!(&pairs[0].replacement, "LLO}");
    }
}
