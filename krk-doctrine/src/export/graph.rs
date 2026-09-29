use std::time::{SystemTime, UNIX_EPOCH};

use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::ast::policy::DoctrinePolicy;
use crate::ir::graph::PolicyGraph;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctrineNodeSnapshot {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DoctrineEdgeSnapshot {
    pub from: String,
    pub to: String,
    pub weight: f32,
    pub enforced: bool,
    pub blocked_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctrinePolicySnapshot {
    pub sandbox: String,
    pub control_plane: String,
    pub doctrine_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DoctrineGraphSnapshot {
    pub timestamp: u64,
    pub system: String,
    pub nodes: Vec<DoctrineNodeSnapshot>,
    pub edges: Vec<DoctrineEdgeSnapshot>,
    pub policy_snapshot: DoctrinePolicySnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<DoctrinePolicy>,
}

#[derive(Debug, Clone)]
pub struct DoctrineExportOptions {
    pub system: String,
    pub sandbox_active: bool,
    pub control_plane_active: bool,
    pub doctrine_version: String,
    pub ast: Option<DoctrinePolicy>,
    pub timestamp: u64,
}

impl Default for DoctrineExportOptions {
    fn default() -> Self {
        Self {
            system: "krk-os".to_string(),
            sandbox_active: true,
            control_plane_active: true,
            doctrine_version: "v0.1".to_string(),
            ast: None,
            timestamp: current_timestamp(),
        }
    }
}

pub fn export_graph(graph: &PolicyGraph) -> DoctrineGraphSnapshot {
    export_graph_with_options(graph, DoctrineExportOptions::default())
}

pub fn export_graph_with_options(
    graph: &PolicyGraph,
    options: DoctrineExportOptions,
) -> DoctrineGraphSnapshot {
    let nodes = graph
        .graph
        .node_indices()
        .map(|index| build_node_snapshot(&graph.graph[index]))
        .collect::<Vec<_>>();

    let edges = graph
        .graph
        .edge_references()
        .map(|edge| {
            let from = graph.graph[edge.source()].clone();
            let to = graph.graph[edge.target()].clone();
            let target_state = classify_state(&to);

            DoctrineEdgeSnapshot {
                from,
                to,
                weight: *edge.weight(),
                enforced: options.sandbox_active || options.control_plane_active,
                blocked_by: if is_restrictive_state(&target_state) {
                    Some("doctrine".to_string())
                } else {
                    None
                },
            }
        })
        .collect::<Vec<_>>();

    DoctrineGraphSnapshot {
        timestamp: options.timestamp,
        system: options.system,
        nodes,
        edges,
        policy_snapshot: DoctrinePolicySnapshot {
            sandbox: if options.sandbox_active {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            control_plane: if options.control_plane_active {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            doctrine_version: options.doctrine_version,
        },
        ast: options.ast,
    }
}

fn build_node_snapshot(label: &str) -> DoctrineNodeSnapshot {
    DoctrineNodeSnapshot {
        id: label.to_string(),
        node_type: classify_node_type(label),
        state: classify_state(label),
    }
}

fn classify_node_type(label: &str) -> String {
    label
        .split(':')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

fn classify_state(label: &str) -> String {
    let segments = label.split(':').collect::<Vec<_>>();
    let tail = segments.last().copied().unwrap_or("declared");

    if is_known_state(tail) {
        tail.to_string()
    } else if segments.len() == 1 {
        "declared".to_string()
    } else {
        "bound".to_string()
    }
}

fn is_known_state(value: &str) -> bool {
    matches!(
        value,
        "allowed"
            | "forbidden"
            | "denied"
            | "sandboxed"
            | "restricted"
            | "inactive"
            | "active"
            | "isolated"
            | "high"
            | "medium"
            | "low"
            | "none"
    )
}

fn is_restrictive_state(value: &str) -> bool {
    matches!(value, "forbidden" | "denied" | "sandboxed" | "restricted" | "isolated" | "none")
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compiler::compile_doctrine, parser::yaml::parse_yaml};

    #[test]
    fn export_graph_produces_canonical_snapshot_shape() {
        let raw_yaml = "
system: krk-swarm
policy:
  memory:
    read: allowed
    write: sandboxed
  execution:
    external_calls: forbidden
";
        let policy = parse_yaml(raw_yaml).unwrap();
        let graph = compile_doctrine(policy.clone()).unwrap();

        let snapshot = export_graph_with_options(
            &graph,
            DoctrineExportOptions {
                system: "krk-os".to_string(),
                sandbox_active: true,
                control_plane_active: true,
                doctrine_version: "v0.1".to_string(),
                ast: Some(policy),
                timestamp: 42,
            },
        );

        assert_eq!(snapshot.timestamp, 42);
        assert_eq!(snapshot.system, "krk-os");
        assert_eq!(snapshot.policy_snapshot.sandbox, "active");
        assert_eq!(snapshot.policy_snapshot.control_plane, "active");
        assert_eq!(snapshot.policy_snapshot.doctrine_version, "v0.1");
        assert!(snapshot.nodes.iter().any(|node| node.id == "memory:write:sandboxed" && node.state == "sandboxed"));
        assert!(snapshot.edges.iter().any(|edge| {
            edge.to == "execution:external:forbidden"
                && edge.enforced
                && edge.blocked_by.as_deref() == Some("doctrine")
        }));
        assert!(snapshot.ast.is_some());
    }

    #[test]
    fn export_graph_marks_non_restrictive_edges_as_unblocked() {
        let raw_yaml = "
system: krk-swarm
policy:
  execution:
    external_calls: allowed
";
        let policy = parse_yaml(raw_yaml).unwrap();
        let graph = compile_doctrine(policy).unwrap();

        let snapshot = export_graph_with_options(
            &graph,
            DoctrineExportOptions {
                sandbox_active: false,
                control_plane_active: true,
                timestamp: 7,
                ..DoctrineExportOptions::default()
            },
        );

        let edge = snapshot
            .edges
            .iter()
            .find(|edge| edge.to == "execution:external:allowed")
            .unwrap();

        assert!(edge.enforced);
        assert_eq!(edge.blocked_by, None);
    }
}
