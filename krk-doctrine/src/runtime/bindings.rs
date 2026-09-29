use crate::ir::graph::PolicyGraph;

#[derive(Clone, Debug)]
pub struct RuntimePolicyHandle {
    pub graph: PolicyGraph,
}

impl RuntimePolicyHandle {
    pub fn new(graph: PolicyGraph) -> Self {
        Self { graph }
    }

    /// Evaluates whether a generic action is permitted under the current control graph
    pub fn allows(&self, action: &str) -> bool {
        // Search the graph nodes to check if they block this action
        for idx in self.graph.graph.node_indices() {
            let node_label = &self.graph.graph[idx];
            if node_label.contains(action) && node_label.contains("forbidden") {
                return false;
            }
            if node_label.contains(action) && node_label.contains("denied") {
                return false;
            }
        }
        true
    }
}
