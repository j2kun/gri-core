use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub id: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub id: i64,
    pub source: i64,
    pub target: i64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Graph {
    pub vertices: HashMap<i64, Vertex>,
    pub edges: HashMap<i64, Edge>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GraphOperation {
    AddVertex(Vertex),
    RemoveVertex(Vertex),
    AddEdge(Edge),
    RemoveEdge(Edge),
}

#[derive(Debug, Clone)]
pub struct Diff {
    pub operations: Vec<GraphOperation>,
}


use GraphOperation::*;

impl GraphOperation {
    pub fn invert(self) -> GraphOperation {
        match self {
            AddVertex(v) => RemoveVertex(v),
            RemoveVertex(v) => AddVertex(v),
            AddEdge(e) => RemoveEdge(e),
            RemoveEdge(e) => AddEdge(e),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Graph::new()
    }
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            vertices: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn apply_all(&mut self, operations: Vec<GraphOperation>) -> Diff {
        Diff {
            operations: operations
                .iter()
                .flat_map(|operation| self.apply(*operation).operations)
                .collect(),
        }
    }

    // TODO: change return type to Result<Diff, Error>
    // and define new Error class that can be used to report errors to user
    pub fn apply(&mut self, operation: GraphOperation) -> Diff {
        match operation {
            AddVertex(v) => self.add_vertex(v),
            RemoveVertex(v) => self.remove_vertex(v),
            AddEdge(e) => self.add_edge(e),
            RemoveEdge(e) => self.remove_edge(e),
        }
    }

    pub fn resolve_vertex(&self, vertex: &str) -> Option<i64> {
        vertex
            .trim()
            .parse::<i64>()
            .ok()
            .filter(|x| self.vertices.contains_key(&x))
    }

    pub fn add_vertex(&mut self, v: Vertex) -> Diff {
        let mut ops = Vec::new();
        let result = self.vertices.insert(v.id, v);

        if result.is_none() {
            ops.push(AddVertex(v));
        } else {
            // TODO: add a "modify vertex"?
        }

        Diff { operations: ops }
    }

    pub fn remove_vertex(&mut self, v: Vertex) -> Diff {
        let mut ops = Vec::new();
        let result = self.vertices.remove(&v.id);
        if result.is_some() {
            ops.push(RemoveVertex(v));

            // Each edge referring to this vertex is now
            // invalid and must be removed.
            // TODO: make more efficient with an index
            // from vertex to incident edges.
            let mut edges_to_remove: HashSet<Edge> = HashSet::new();
            for edge in self.edges.values() {
                if edge.source == v.id || edge.target == v.id {
                    edges_to_remove.insert(*edge);
                }
            }

            for edge in edges_to_remove.iter() {
                self.edges.remove(&edge.id);
                ops.push(RemoveEdge(*edge));
            }
        }

        Diff { operations: ops }
    }

    pub fn add_edge(&mut self, e: Edge) -> Diff {
        if !self.vertices.contains_key(&e.source) {
            panic!("Unknown vertex {:?}", e.source);
        }
        if !self.vertices.contains_key(&e.target) {
            panic!("Unknown vertex {:?}", e.target);
        }

        let mut ops = Vec::new();
        let result = self.edges.insert(e.id, e);
        if result.is_none() {
            ops.push(AddEdge(e));
        } else {
            // TODO: add an edge edit operation?
        }

        Diff { operations: ops }
    }

    pub fn remove_edge(&mut self, e: Edge) -> Diff {
        let mut ops = Vec::new();
        let result = self.edges.remove(&e.id);
        if result.is_some() {
            ops.push(RemoveEdge(e));
        }

        Diff { operations: ops }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let g = Graph::new();
        assert_eq!(g.vertices.len(), 0);
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn new_construct_small_graph() {
        let mut g = Graph::new();
        let v1 = Vertex { id: 1 };
        let v2 = Vertex { id: 2 };
        let v3 = Vertex { id: 3 };

        let e1 = Edge {
            id: 1,
            source: v1.id,
            target: v2.id,
        };
        let e2 = Edge {
            id: 2,
            source: v2.id,
            target: v3.id,
        };

        g.add_vertex(v1);
        g.add_vertex(v2);
        g.add_vertex(v3);
        g.add_edge(e1);
        g.add_edge(e2);

        assert_eq!(HashMap::from([(1, v1), (2, v2), (3, v3)]), g.vertices);
        assert_eq!(HashMap::from([(1, e1), (2, e2)]), g.edges);
    }

    #[test]
    fn undo_operations() {
        let mut g = Graph::new();
        let mut history = Vec::new();
        let v1 = Vertex { id: 1 };
        let v2 = Vertex { id: 2 };
        let v3 = Vertex { id: 3 };

        history.extend(g.add_vertex(v1).operations);
        history.extend(g.add_vertex(v2).operations);
        history.extend(g.add_vertex(v3).operations);
        let e1 = Edge {
            id: 1,
            source: v1.id,
            target: v2.id,
        };
        let e2 = Edge {
            id: 2,
            source: v2.id,
            target: v3.id,
        };

        history.extend(g.add_edge(e1).operations);
        history.extend(g.add_edge(e2).operations);

        for op in history.into_iter() {
            g.apply(op.invert());
        }

        assert_eq!(g.vertices, HashMap::new());
        assert_eq!(g.edges, HashMap::new());
    }

    #[test]
    fn remove_vertex_removes_all_incident_edges() {
        let mut g = Graph::new();
        let v1 = Vertex { id: 1 };
        let v2 = Vertex { id: 2 };
        let v3 = Vertex { id: 3 };
        let e1 = Edge {
            id: 1,
            source: v1.id,
            target: v2.id,
        };
        let e2 = Edge {
            id: 2,
            source: v1.id,
            target: v3.id,
        };

        g.add_vertex(v1);
        g.add_vertex(v2);
        g.add_vertex(v3);
        g.add_edge(e1);
        g.add_edge(e2);

        assert_eq!(HashMap::from([(1, v1), (2, v2), (3, v3)]), g.vertices);
        assert_eq!(HashMap::from([(1, e1), (2, e2)]), g.edges);

        g.remove_vertex(v1);

        assert_eq!(HashMap::from([(2, v2), (3, v3)]), g.vertices);
        assert_eq!(HashMap::new(), g.edges);
    }
}
