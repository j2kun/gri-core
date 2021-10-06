#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    id: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Edge {
    id: i32,
    source_id: i32,
    target_id: i32,
}

#[derive(Debug)]
pub struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    next_vertex_id: i32,
    next_edge_id: i32,
}

impl Graph {
    pub fn new() -> Graph {
        return Graph {
            vertices: Vec::new(),
            edges: Vec::new(),
            next_vertex_id: 0,
            next_edge_id: 0,
        };
    }

    pub fn add_vertex(&mut self) -> Vertex {
        let v = Vertex {
            id: self.next_vertex_id,
        };
        self.next_vertex_id += 1;
        self.vertices.push(v);
        return v;
    }

    pub fn add_edge(&mut self, source: Vertex, target: Vertex) -> Edge {
        if !self.vertices.contains(&source) {
            panic!("Unknown vector {:?}", source);
        }
        if !self.vertices.contains(&target) {
            panic!("Unknown vector {:?}", target);
        }

        let e = Edge {
            id: self.next_edge_id,
            source_id: source.id,
            target_id: target.id,
        };
        self.next_edge_id += 1;
        self.edges.push(e);
        return e;
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
    fn new_add_vertex() {
        let mut g = Graph::new();
        let v = g.add_vertex();
        assert_eq!(g.vertices, vec![v]);
        assert_eq!(v.id, 0);
        
        let w = g.add_vertex();
        assert_eq!(g.vertices, vec![v, w]);
        assert_eq!(w.id, 1);
    }

    #[test]
    fn new_add_edge() {
        let mut g = Graph::new();
        let v = g.add_vertex();
        let w = g.add_vertex();
        assert_eq!(g.vertices, vec![v, w]);

        let e = g.add_edge(v, w);
        assert_eq!(g.edges, vec![e]);
        assert_eq!(e.id, 0);

        let f = g.add_edge(v, w);
        assert_eq!(g.edges, vec![e, f]);
        assert_eq!(f.id, 1);
    }
}
