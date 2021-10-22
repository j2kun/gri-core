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
                if let Some(graph_op) = self.interpret_modal_operation(op) {
                    let diff = self.document.apply(graph_op);
                    let new_node_id = self.history_tree.new_node(diff);
                    if let Some(node_id) = self.last_edit {
                        node_id.append(new_node_id, &mut self.history_tree);
                    }
                    self.last_edit = Some(new_node_id);
                }
            }
            TransitionResult::Error(msg, next_mode) => {
                println!("{}", msg);
                self.mode = next_mode;
            }
        }
    }

    fn interpret_modal_operation(&mut self, op: ModalOperation) -> Option<GraphOperation> {
        match op {
            ModalOperation::CreateNewVertex => {
                let v = Vertex {
                    id: self.next_vertex_id,
                };
                self.next_vertex_id += 1;
                Some(GraphOperation::AddVertex(v))
            }
            ModalOperation::CreateNewEdge(chosen_vertices) => {
                let maybe_edge = chosen_vertices
                    .rsplit_once(',')
                    .map(|(source_id, target_id)| {
                        let source = self
                            .document
                            .resolve_vertex(source_id)
                            .expect(format!("Could not find source vertex {}", source_id).as_str());
                        let target = self
                            .document
                            .resolve_vertex(target_id)
                            .expect(format!("Could not find target vertex {}", target_id).as_str());
                        Edge {
                            id: self.next_edge_id,
                            source: source,
                            target: target,
                        }
                    });

                match maybe_edge {
                    Some(e) => {
                        self.next_edge_id += 1;
                        Some(GraphOperation::AddEdge(e))
                    }
                    // Replace this with an error message reported to the user.
                    None => panic!(
                        "Unable to parse '{}' as a list of two vertex ids.",
                        chosen_vertices
                    ),
                }
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
    use crate::graph::Edge;
    use crate::graph::Graph;
    use crate::graph::Vertex;

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

        let mut g = Graph::new();
        let v0 = Vertex { id: 0 };
        let v1 = Vertex { id: 1 };
        let e0 = Edge {
            id: 0,
            source: v0.id,
            target: v1.id,
        };
        g.add_vertex(v0);
        g.add_vertex(v1);
        g.add_edge(e0);

        assert_eq!(EditorMode::Insert, state.mode);
        assert_eq!(g, state.document);
    }
}
