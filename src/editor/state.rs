use indextree::Arena;
use indextree::NodeId;

use crate::editor::mode::EditorMode;
use crate::editor::mode::TransitionResult;
use crate::graph::graph::Diff;
use crate::graph::graph::Graph;

#[derive(Debug)]
pub struct EditorState {
    mode: EditorMode,

    // The document, as materialized with respect to the current
    // position in the history tree.
    document: Graph,

    // The "undo tree" of this editing session
    // Children of a given node are appended in time order.
    history_tree: Arena<Diff>,

    // The id of the node in the history tree corresponding
    // to the last edit of the document.
    last_edit: Option<NodeId>,
}

impl EditorState {
    pub fn new() -> EditorState {
        return EditorState {
            mode: EditorMode::Command,
            document: Graph::new(),
            history_tree: Arena::new(),
            last_edit: None,
        };
    }

    pub fn evaluate(&mut self, input: Input) {
        let transition_result = self.mode.transition(input);
        match transition_result {
            TransitionResult::ModeChange(next_mode) => {
                self.mode = next_mode;
            }
            TransitionResult::Apply(op, next_mode) => {
                self.mode = next_mode;
                let diff = self.document.apply(op);
                let new_node_id = self.history_tree.new_node(diff);
                match self.last_edit {
                    Some(node_id) => {
                        node_id.append(new_node_id, &mut self.history_tree);
                    }
                    _ => {}
                }
                self.last_edit = Some(new_node_id);
            }
            TransitionResult::Error(msg, next_mode) => {
                println!("{}", msg);
                self.mode = next_mode;
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Input {
    Key(char),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::keys::*;

    #[test]
    fn insert_two_vertices_and_edge() {
        let mut state = EditorState::new();
        state.evaluate(Input::Key(I_LOWER));
        assert_eq!(EditorMode::Insert, state.mode);
    }
}
