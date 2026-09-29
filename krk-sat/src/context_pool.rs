//! Context Pooling for Incremental Z3 Sessions
pub struct ContextPool {}
impl ContextPool {
    pub fn acquire(&self) -> crate::IncrementalContext { crate::IncrementalContext::new() }
    pub fn release(&self, _ctx: crate::IncrementalContext) {}
}
