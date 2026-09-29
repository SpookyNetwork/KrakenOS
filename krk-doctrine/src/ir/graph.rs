use petgraph::graph::DiGraph;

#[derive(Debug, Clone)]
pub struct PolicyGraph {
    pub graph: DiGraph<String, f32>,
}
