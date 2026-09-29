use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctrinePolicy {
    pub system: String,
    pub policy: PolicyBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyBlock {
    pub memory: Option<MemoryPolicy>,
    pub agents: Option<std::collections::HashMap<String, AgentPolicy>>,
    pub execution: Option<ExecutionPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryPolicy {
    pub read: String,
    pub write: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPolicy {
    pub budget: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPolicy {
    pub external_calls: String,
}
