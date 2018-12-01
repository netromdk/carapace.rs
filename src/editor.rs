use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{CompletionType, Config, EditMode, Editor, Helper};

/// Creates `Editor` instance with proper config and completion.
pub fn create_editor() -> Editor<EditorHelper> {
    let config = Config::builder()
        .history_ignore_space(true)
        .edit_mode(EditMode::Emacs)
        .completion_type(CompletionType::List)
        .build();

    let mut editor = Editor::with_config(config);

    let h = EditorHelper::new();
    editor.set_helper(Some(h));

    editor
}

pub struct EditorHelper {
    pub file_comp: Box<FilenameCompleter>,
}

impl EditorHelper {
    pub fn new() -> EditorHelper {
        EditorHelper {
            file_comp: Box::new(FilenameCompleter::new()),
        }
    }
}

fn builtin_command_completer(line: &str, pos: usize) -> Result<(usize, Vec<Pair>), ReadlineError> {
    let builtins = vec!["cd", "quit", "exit", "h", "hist", "history"];

    // Show all candidates with no input and pos=0.
    if pos == 0 {
        let mut candidates = Vec::new();
        for builtin in &builtins {
            candidates.push(Pair {
                display: builtin.to_string(),
                replacement: builtin.to_string(),
            });
        }
        return Ok((pos, candidates));
    }
    // Check for partial matches and their remainders.
    else {
        let slice = &line[..pos];
        for builtin in &builtins {
            if *builtin == slice || builtin.starts_with(slice) {
                let cmd = builtin.to_string();
                return Ok((
                    pos,
                    vec![Pair {
                        display: cmd.clone(),

                        // The missing part of the candidate.
                        replacement: cmd[slice.len()..].to_string(),
                    }],
                ));
            }
        }
    }

    // No matches.
    return Ok((pos, vec![]));
}

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Do built-in command completion if position is within first word.
        if let Some(wpos) = line.find(char::is_whitespace) {
            if pos < wpos {
                return builtin_command_completer(line, pos);
            }
        } else {
            return builtin_command_completer(line, pos);
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
