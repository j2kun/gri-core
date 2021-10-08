use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vertex {
    id: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    id: i64,
    source: Vertex,
    target: Vertex,
}

#[derive(Debug)]
pub struct Graph {
    vertices: HashSet<Vertex>,
    edges: HashSet<Edge>,
}

#[derive(Debug)]
pub enum Operation {
    AddVertex(Vertex),
    RemoveVertex(Vertex),
    AddEdge(Edge),
    RemoveEdge(Edge),
}

#[derive(Debug)]
pub struct Diff {
    operations: Vec<Operation>,
}

use Operation::*;

impl Operation {
    pub fn invert(self) -> Operation {
        match self {
            AddVertex(v) => RemoveVertex(v),
            RemoveVertex(v) => AddVertex(v),
            AddEdge(e) => RemoveEdge(e),
            RemoveEdge(e) => AddEdge(e),
        }
    }
}

impl Graph {
    pub fn new() -> Graph {
        return Graph {
            vertices: HashSet::new(),
            edges: HashSet::new(),
        };
    }

    // TODO: change return type to Result<Diff, Error>
    // and define new Error class that can be used to report errors to user
    pub fn apply(&mut self, operation: Operation) -> Diff {
        match operation {
            AddVertex(v) => self.add_vertex(v),
            RemoveVertex(v) => self.remove_vertex(v),
            AddEdge(e) => self.add_edge(e),
            RemoveEdge(e) => self.remove_edge(e),
        }
    }

    pub fn add_vertex(&mut self, v: Vertex) -> Diff {
        let mut ops = Vec::new();
        let result = self.vertices.insert(v);
        if result {
            ops.push(AddVertex(v));
        }

        Diff { operations: ops }
    }

    pub fn remove_vertex(&mut self, v: Vertex) -> Diff {
        let mut ops = Vec::new();
        let result = self.vertices.remove(&v);
        if result {
            ops.push(RemoveVertex(v));

            // Each edge referring to this vertex is now
            // invalid and must be removed.
            // TODO: make more efficient with an index
            // from vertex to incident edges.
            let mut edges_to_remove: HashSet<Edge> = HashSet::new();
            for edge in self.edges.iter() {
                if edge.source == v || edge.target == v {
                    edges_to_remove.insert(*edge);
                }
            }

            for edge in edges_to_remove.iter() {
                self.edges.remove(edge);
                ops.push(RemoveEdge(*edge));
            }
        }

        Diff { operations: ops }
    }

    pub fn add_edge(&mut self, e: Edge) -> Diff {
        if !self.vertices.contains(&e.source) {
            panic!("Unknown vertex {:?}", e.source);
        }
        if !self.vertices.contains(&e.target) {
            panic!("Unknown vertex {:?}", e.target);
        }

        let mut ops = Vec::new();
        let result = self.edges.insert(e);
        if result {
            ops.push(AddEdge(e));
        }

        Diff { operations: ops }
    }

    pub fn remove_edge(&mut self, e: Edge) -> Diff {
        let mut ops = Vec::new();
        let result = self.edges.remove(&e);
        if result {
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
            source: v1,
            target: v2,
        };
        let e2 = Edge {
            id: 2,
            source: v2,
            target: v3,
        };

        g.add_vertex(v1);
        g.add_vertex(v2);
        g.add_vertex(v3);
        g.add_edge(e1);
        g.add_edge(e2);

        assert_eq!(g.vertices, vec![v1, v2, v3].into_iter().collect());
        assert_eq!(g.edges, vec![e1, e2].into_iter().collect());
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
            source: v1,
            target: v2,
        };
        let e2 = Edge {
            id: 2,
            source: v2,
            target: v3,
        };

        history.extend(g.add_edge(e1).operations);
        history.extend(g.add_edge(e2).operations);

        for op in history.into_iter() {
            g.apply(op.invert());
        }

        assert_eq!(g.vertices, HashSet::new());
        assert_eq!(g.edges, HashSet::new());
    }

    #[test]
    fn remove_vertex_removes_all_incident_edges() {
        let mut g = Graph::new();
        let v1 = Vertex { id: 1 };
        let v2 = Vertex { id: 2 };
        let v3 = Vertex { id: 3 };
        let e1 = Edge {
            id: 1,
            source: v1,
            target: v2,
        };
        let e2 = Edge {
            id: 2,
            source: v1,
            target: v3,
        };

        g.add_vertex(v1);
        g.add_vertex(v2);
        g.add_vertex(v3);
        g.add_edge(e1);
        g.add_edge(e2);

        assert_eq!(g.vertices, vec![v1, v2, v3].into_iter().collect());
        assert_eq!(g.edges, vec![e1, e2].into_iter().collect());

        g.remove_vertex(v1);

        assert_eq!(g.vertices, vec![v2, v3].into_iter().collect());
        assert_eq!(g.edges, HashSet::new());
    }
}
