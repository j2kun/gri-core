use crate::graph::graph::Operation;
use crate::editor::keys::*;

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    // Like vim, Command mode is the default mode with no pending operations.
    Command,
    // Insert mode commands modify the graph object-at-a-time, like insert mode in vim allows you
    // to modify character-at-a-time.
    Insert,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TransitionResult {
    ModeChange(EditorMode),
    Error(String, EditorMode),

    // For now this is operation because the only thing I can do is add/remove vertices and edges.
    // Later this will move somewhere else and include any sort of modification to the document.
    Apply(Operation, EditorMode),
}

// Probably should go elsewhere once input is more complex.
#[derive(Debug, Eq, PartialEq)]
pub enum Input {
    Key(char),
}

use EditorMode::*;
use Input::*;
use TransitionResult::*;

impl EditorMode {
    // Transition from one mode to another, possibly the same,
    // and optionally emitting a document-modifying operation
    // or an error.
    pub fn transition(self, input: Input) -> TransitionResult {
        match self {
            Command => match input {
                Key(I_LOWER) => ModeChange(Insert),
                _ => self.unknown_command(input),
            },
            Insert => match input {
                Key(ESC) => ModeChange(Command),
                _ => self.unknown_command(input),
            },
        }
    }

    fn unknown_command(self, input: Input) -> TransitionResult {
        Error(
            format!(
                "Input {:?} doesn't do anything in the current mode: {:#?}",
                input, self
            ),
            self,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_to_insert_mode() {
        let mode = Command;
        let actual = mode.transition(Key(I_LOWER));
        let expected = ModeChange(Insert);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transition_to_command_mode() {
        let mode = Insert;
        let actual = mode.transition(Key(ESC));
        let expected = ModeChange(Command);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transition_command_err() {
        let mode = Command;
        let actual = mode.transition(Key('f'));
        let expected = Error(
            "Input Key('f') doesn't do anything in the current mode: Command".to_string(),
            Command,
        );
        assert_eq!(expected, actual);
    }
}
