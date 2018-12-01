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

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>), ReadlineError> {
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
