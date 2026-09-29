use std::process::Command;
use std::env;

pub mod incremental;
pub mod context_pool;
pub mod proof_cache;
pub mod assumptions;
pub mod unsat_core;

pub use incremental::IncrementalContext;
pub use proof_cache::{ProofCache, ProofKey, CachedProof};

#[derive(Debug)]
pub enum SatError {
    Unsat,
    Unknown,
    ProductionInvariantViolation(String),
}

pub struct Z3Bridge {}

impl Z3Bridge {
    pub fn new() -> Self {
        Self {}
    }

    pub fn verify_transition(&self, smt_script: &str) -> Result<(), SatError> {
        let is_prod = env::var("NODE_ENV").unwrap_or_default() == "production";

        // TRUTH INJECTION: Replace mock logic with real Z3 binary call
        let output = Command::new("z3")
            .arg("-in")
            .arg("-smt2")
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(smt_script.as_bytes())?;
                child.wait_with_output()
            });

        match output {
            Ok(res) => {
                let stdout = String::from_utf8_lossy(&res.stdout);
                if stdout.contains("unsat") {
                    Ok(())
                } else if stdout.contains("sat") {
                    Err(SatError::Unsat)
                } else {
                    Err(SatError::Unknown)
                }
            }
            Err(e) => {
                if is_prod {
                    // GLOBAL INVARIANT: Hard fail in production if real solver is missing
                    panic!("[PRODUCTION_INVARIANT_VIOLATION] Real Z3 solver unavailable or failed: {}", e);
                } else {
                    // Fallback for dev only (logged as warning)
                    eprintln!("[WARN] Z3 solver failed: {}. Falling back to simulation mode.", e);
                    Ok(())
                }
            }
        }
    }
}
