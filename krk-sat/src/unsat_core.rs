//! UNSAT Core Extraction for Counterexample Trace Analysis
pub struct UnsatCore {
    pub conflicting_constraints: Vec<crate::incremental::Bool>,
}
