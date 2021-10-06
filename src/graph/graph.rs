#[derive(Debug)]
pub struct Vertex {
    id: i32,
}

#[derive(Debug)]
pub struct Edge {
    id: i32,
    source_id: i32,
    target_id: i32,
}

#[derive(Debug)]
pub struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

#[cfg(test)]
mod tests {
    use crate::graph::graph::*;

    #[test]
    fn create_graph() {
        let v = Vertex { id: 0 };
        let w = Vertex { id: 1 };
        let e = Edge {
            id: 0,
            source_id: v.id,
            target_id: w.id,
        };
        let _g = Graph {
            vertices: vec![v, w],
            edges: vec![e],
        };
    }
}
