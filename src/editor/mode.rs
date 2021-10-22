use crate::editor::keys::*;
use crate::editor::state::Input;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum EditorMode {
    // Like vim, Command mode is the default mode with no pending operations.
    Command,
    // Insert mode commands modify the graph object-at-a-time, like insert mode in vim allows you
    // to modify character-at-a-time.
    Insert,
    // After the user declares they want to create an edge, the state machine requires extra
    // information regarding which vertices to connect.
    InsertEdgePending(String),
}

/**
 * A ModalOperation represents an abstract operation, comprehensible to an end user, that is the
 * result of a particular sequence of commands. The state machine that handles user input "emits"
 * ModalOperations that are then interpreted by the Editor to determine how to update the editor
 * state.
 */
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ModalOperation {
    CreateNewVertex,
    CreateNewEdge(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TransitionResult {
    // An action emitted, with a new (but possibly unchanged) mode to enter.
    Apply(ModalOperation, EditorMode),
    // A simple mode change that results in no action for the editor to take.
    ModeChange(EditorMode),
    // An error (e.g., an unsupported operation) to report to the user, and a mode to enter as a
    // consequence.
    Error(String, EditorMode),
}

use EditorMode::*;
use ModalOperation::*;
use TransitionResult::*;

impl EditorMode {
    // Transition from one mode to another, possibly the same,
    // and optionally emitting a document-modifying operation
    // or an error.
    pub fn transition(self, input: Input) -> TransitionResult {
        match self {
            Command => match input {
                Input::Key(I_LOWER) => ModeChange(Insert),
                _ => self.unknown_command(input),
            },
            Insert => match input {
                Input::Key(ESC) => ModeChange(Command),
                Input::Key(V_LOWER) => Apply(CreateNewVertex, Insert),
                Input::Key(E_LOWER) => ModeChange(InsertEdgePending("".to_string())),
                _ => self.unknown_command(input),
            },
            InsertEdgePending(s) => match input {
                Input::Key(ESC) => ModeChange(Insert),
                Input::Key(ENTER) => Apply(CreateNewEdge(s), Insert),
                Input::Key(next_key) => ModeChange(InsertEdgePending(s + &next_key.to_string())),
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
        let actual = mode.transition(Input::Key(I_LOWER));
        let expected = ModeChange(Insert);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transition_to_command_mode() {
        let mode = Insert;
        let actual = mode.transition(Input::Key(ESC));
        let expected = ModeChange(Command);
        assert_eq!(expected, actual);
    }

    #[test]
    fn emit_operation_new_vertex() {
        let mode = Insert;
        let actual = mode.transition(Input::Key(V_LOWER));
        let expected = Apply(CreateNewVertex, Insert);
        assert_eq!(expected, actual);
    }

    #[test]
    fn transition_command_err() {
        let mode = Command;
        let actual = mode.transition(Input::Key('f'));
        let expected = Error(
            "Input Key('f') doesn't do anything in the current mode: Command".to_string(),
            Command,
        );
        assert_eq!(expected, actual);
    }
}
