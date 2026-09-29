use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    Network(NetworkScope),
    FileSystem(FsScope),
    ProcessExecution(ProcessScope),
    StateMutation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityScope {
    pub capabilities: Vec<Capability>,
    pub max_ttl_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkScope {
    DenyAll,
    AllowLocalhostOnly,
    AllowEgress(Vec<String>), // List of allowed domains/IPs
    AllowAll, // Danger
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsScope {
    ReadOnly,
    IsolatedTempDir(String), // Path prefix
    ReadWriteWorkspace(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessScope {
    None,
    WasmOnly,
    SubprocessAllowlist(Vec<String>), // E.g. ["git", "cargo"]
}

impl CapabilityScope {
    pub fn new_strict_isolation() -> Self {
        Self {
            capabilities: vec![
                Capability::Network(NetworkScope::DenyAll),
                Capability::FileSystem(FsScope::IsolatedTempDir("/tmp/krk-agent-sandbox".to_string())),
                Capability::ProcessExecution(ProcessScope::None),
            ],
            max_ttl_ms: 10_000, // 10s default TTL
        }
    }

    pub fn has_network_egress(&self, domain: &str) -> bool {
        for cap in &self.capabilities {
            if let Capability::Network(net) = cap {
                match net {
                    NetworkScope::AllowAll => return true,
                    NetworkScope::AllowEgress(domains) => return domains.contains(&domain.to_string()),
                    _ => return false,
                }
            }
        }
        false
    }
}
