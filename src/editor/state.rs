use indextree::Arena;
use indextree::NodeId;

use crate::editor::mode::EditorMode;
use crate::editor::mode::ModalOperation;
use crate::editor::mode::TransitionResult;
use crate::graph::Diff;
use crate::graph::Edge;
use crate::graph::Graph;
use crate::graph::GraphOperation;
use crate::graph::Vertex;

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

    next_vertex_id: i64,
    next_edge_id: i64,
}

#[derive(Debug)]
pub struct OpInterpretation {
    document_changes: Diff,
    new_history_node: bool,
    set_last_edit: Option<NodeId>,
}

impl Default for OpInterpretation {
    fn default() -> Self {
        OpInterpretation {
            document_changes: Diff {
                operations: Vec::new(),
            },
            new_history_node: false,
            set_last_edit: None,
        }
    }
}

impl OpInterpretation {
    pub fn standard_op(ops: Vec<GraphOperation>) -> Self {
        OpInterpretation {
            document_changes: Diff { operations: ops },
            new_history_node: true,
            set_last_edit: None,
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState::new()
    }
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            mode: EditorMode::Command,
            document: Graph::new(),
            history_tree: Arena::new(),
            last_edit: None,
            next_vertex_id: 0,
            next_edge_id: 0,
        }
    }

    pub fn evaluate(&mut self, input: Input) {
        let transition_result = self.mode.clone().transition(input);
        match transition_result {
            TransitionResult::ModeChange(next_mode) => {
                self.mode = next_mode;
            }
            TransitionResult::Apply(op, next_mode) => {
                self.mode = next_mode;
                let interpreted_op = self.interpret_modal_operation(op);
                let diff = self
                    .document
                    .apply_all(interpreted_op.document_changes.operations);

                if interpreted_op.new_history_node {
                    let new_node_id = self.history_tree.new_node(diff);
                    if let Some(node_id) = self.last_edit {
                        node_id.append(new_node_id, &mut self.history_tree);
                    }
                    self.last_edit = Some(new_node_id);
                }

                if let Some(node_id) = interpreted_op.set_last_edit {
                    self.last_edit = Some(node_id);
                }
            }
            TransitionResult::Error(msg, next_mode) => {
                println!("{}", msg);
                self.mode = next_mode;
            }
        }
    }

    fn interpret_modal_operation(&mut self, op: ModalOperation) -> OpInterpretation {
        match op {
            ModalOperation::CreateNewVertex => {
                let v = Vertex {
                    id: self.next_vertex_id,
                };
                self.next_vertex_id += 1;
                OpInterpretation::standard_op(vec![GraphOperation::AddVertex(v)])
            }
            ModalOperation::CreateNewEdge(chosen_vertices) => {
                let maybe_edge =
                    chosen_vertices
                        .rsplit_once(',')
                        .map(|(source_id, target_id)| {
                            let source = self.document.resolve_vertex(source_id).expect(
                                format!("Could not find source vertex {}", source_id).as_str(),
                            );
                            let target = self.document.resolve_vertex(target_id).expect(
                                format!("Could not find target vertex {}", target_id).as_str(),
                            );
                            Edge {
                                id: self.next_edge_id,
                                source: source,
                                target: target,
                            }
                        });

                match maybe_edge {
                    Some(e) => {
                        self.next_edge_id += 1;
                        OpInterpretation::standard_op(vec![GraphOperation::AddEdge(e)])
                    }
                    // Replace this with an error message reported to the user.
                    None => panic!(
                        "Unable to parse '{}' as a list of two vertex ids.",
                        chosen_vertices
                    ),
                }
            }
            ModalOperation::Undo => match self.last_edit {
                None => OpInterpretation::default(),
                Some(last_edit_id) => {
                    let last_edit = (*(self.history_tree.get(last_edit_id).unwrap())).clone();
                    let diff = Diff {
                        operations: last_edit
                            .get()
                            .operations
                            .iter()
                            .map(|op| op.invert())
                            .collect(),
                    };
                    OpInterpretation {
                        document_changes: diff,
                        new_history_node: false,
                        set_last_edit: last_edit.parent(),
                    }
                }
            },
            ModalOperation::Redo => match self.last_edit {
                None => OpInterpretation::default(),
                Some(last_edit_id) => match (*self.history_tree.get(last_edit_id).unwrap())
                    .last_child()
                {
                    None => OpInterpretation::default(),
                    Some(next_state_id) => OpInterpretation {
                        document_changes: (*(*self.history_tree.get(next_state_id).unwrap()).get())
                            .clone(),
                        new_history_node: false,
                        set_last_edit: Some(next_state_id),
                    },
                },
            },
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
    use crate::graph::Edge;
    use crate::graph::Graph;
    use crate::graph::Vertex;

    fn single_edge_graph() -> Graph {
        let mut single_edge = Graph::new();
        let v0 = Vertex { id: 0 };
        let v1 = Vertex { id: 1 };
        let e0 = Edge {
            id: 0,
            source: v0.id,
            target: v1.id,
        };
        single_edge.add_vertex(v0);
        single_edge.add_vertex(v1);
        single_edge.add_edge(e0);
        return single_edge.clone();
    }

    #[test]
    fn insert_two_vertices_and_edge() {
        let mut state = EditorState::new();
        state.evaluate(Input::Key(I_LOWER));
        state.evaluate(Input::Key(V_LOWER));
        state.evaluate(Input::Key(V_LOWER));
        state.evaluate(Input::Key(E_LOWER));
        state.evaluate(Input::Key(DIGIT_0));
        state.evaluate(Input::Key(COMMA));
        state.evaluate(Input::Key(DIGIT_1));
        state.evaluate(Input::Key(ENTER));

        assert_eq!(EditorMode::Insert, state.mode);

        let expected = single_edge_graph();
        assert_eq!(expected, state.document);
    }

    #[test]
    fn undo_redo() {
        let mut state = EditorState::new();
        state.evaluate(Input::Key(I_LOWER));
        state.evaluate(Input::Key(V_LOWER));
        state.evaluate(Input::Key(V_LOWER));
        state.evaluate(Input::Key(E_LOWER));
        state.evaluate(Input::Key(DIGIT_0));
        state.evaluate(Input::Key(COMMA));
        state.evaluate(Input::Key(DIGIT_1));
        state.evaluate(Input::Key(ENTER));
        state.evaluate(Input::Key(ESC));

        let single_edge = single_edge_graph();
        assert_eq!(EditorMode::Command, state.mode);
        assert_eq!(single_edge, state.document);

        let mut undid = single_edge_graph();
        undid.remove_edge(*undid.edges.values().next().unwrap());
        state.evaluate(Input::Key(U_LOWER));

        assert_eq!(EditorMode::Command, state.mode);
        assert_eq!(undid, state.document);

        state.evaluate(Input::Key(U_UPPER));

        assert_eq!(EditorMode::Command, state.mode);
        assert_eq!(single_edge, state.document);
    }
}
