#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    InsertChar(char),
    DeleteChar,
    InsertMode,
    NormalMode,
    SaveFile,
    FormatBuffer,
    Completion,
    CompletionNext,
    CompletionPrev,
    CompletionAccept,
    CodeActionNext,
    CodeActionPrev,
    CodeActionAccept,
    EnterCommandMode,
    GotoDefinition,
    FindReferences,
    Hover,
    WorkspaceSymbols,
    CodeAction,
    OpenFuzzySearch,
    FuzzySearchUp,
    FuzzySearchDown,
    FuzzySearchSelect,
    FuzzySearchCancel,
    FuzzySearchToggleRecursive,
    FuzzySearchToggleGitignore,
    FuzzySearchLoadMore,
    Quit,
    // Add more as needed
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
