#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Basic movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    // Word-based motion (with counts)
    MoveWordForward(usize),
    MoveWordBackward(usize),
    MoveWordEnd(usize),

    // Line motion
    MoveLineStart,
    MoveLineEnd(usize),
    MoveFirstNonBlank,

    // File motion
    MoveFileStart,
    MoveFileEnd,
    MoveScreenTop,
    MoveScreenMiddle,
    MoveScreenBottom,

    // Character insertion/deletion
    InsertChar(char),
    DeleteChar,
    DeleteCharForward(usize),
    ReplaceChar(char),

    // Line operations
    DeleteLine,
    DeleteLineIntoRegister(char),

    // Word operations
    DeleteWord(usize),
    DeleteToEndWord(usize),
    DeleteToStartWord(usize),
    DeleteInnerWord(usize),
    DeleteAWord(usize),

    // Range operations
    DeleteToEnd,
    DeleteToStart,
    DeleteToEndOfFile,
    DeleteToStartOfFile,

    // Yank operations
    YankLine,
    YankWord(usize),
    YankToEnd,
    YankToStart,
    YankInnerWord(usize),
    YankAWord(usize),

    // Change operations
    ChangeLine,
    ChangeWord(usize),
    ChangeToEnd,
    ChangeToStart,
    ChangeInnerWord(usize),
    ChangeAWord(usize),
    SubstituteChar,
    SubstituteLine,

    // Paste operations
    PasteAfter,
    PasteBefore,

    // Join operations
    JoinLines(usize),

    // Indent operations
    IndentLine(usize),
    UnindentLine(usize),

    // Undo/Redo
    Undo,
    Redo,

    // Mode switching
    InsertMode,
    NormalMode,
    VisualChar,
    VisualLine,

    // Command mode
    EnterCommandMode,

    // File operations
    SaveFile,
    FormatBuffer,
    Quit,

    // LSP integration
    Completion,
    CompletionNext,
    CompletionPrev,
    CompletionAccept,
    CodeActionNext,
    CodeActionPrev,
    CodeActionAccept,
    GotoDefinition,
    FindReferences,
    Hover,
    WorkspaceSymbols,
    CodeAction,

    // Fuzzy search
    OpenFuzzySearch,
    FuzzySearchUp,
    FuzzySearchDown,
    FuzzySearchSelect,
    FuzzySearchCancel,
    FuzzySearchToggleRecursive,
    FuzzySearchToggleGitignore,
    FuzzySearchLoadMore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_variants() {
        assert_eq!(Command::MoveLeft, Command::MoveLeft);
        assert_ne!(Command::MoveRight, Command::MoveUp);
        let cmd = Command::InsertChar('a');
        if let Command::InsertChar(c) = cmd {
            assert_eq!(c, 'a');
        } else {
            panic!("Expected InsertChar");
        }
    }

    #[test]
    fn test_command_clone() {
        let cmd = Command::SaveFile;
        let cloned = cmd.clone();
        assert_eq!(cmd, cloned);
    }
}
