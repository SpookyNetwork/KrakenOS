//! Coordination — Multi-agent pipeline enforcement
//!
//! Ensures correct ordering of agent outputs:
//! planner → executor → critic → memory → risk

use crate::{AgentAction, ActionType};

/// Validates that a set of agent actions follows the coordination protocol.
/// Returns only the actions that are compatible with the current pipeline stage.
pub fn filter_coordination_order(actions: &[AgentAction]) -> Vec<&AgentAction> {
    // Priority: Block > Plan > Execute > Critique > Memorize > Noop
    let mut result: Vec<&AgentAction> = actions.iter().collect();
    result.sort_by_key(|a| match a.action_type {
        ActionType::Block => 0,
        ActionType::Plan => 1,
        ActionType::Execute => 2,
        ActionType::Critique => 3,
        ActionType::Memorize => 4,
        ActionType::Noop => 5,
    });

    // If any agent blocks, only block actions survive.
    if result.first().map(|a| a.action_type == ActionType::Block).unwrap_or(false) {
        result.retain(|a| a.action_type == ActionType::Block);
    }

    result
}
