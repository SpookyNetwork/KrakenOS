#[derive(Debug, Clone)]
pub enum ExecutionScope {
    Global,
    Sandboxed,
    Isolated,
    Denied,
}
