//! Authority Model — Hierarchical trust levels
//!
//! Defines the authority hierarchy for control decisions.
//! Each escalation level requires progressively stronger authorization.

use crate::AuthorityLevel;

/// Checks whether a given authority level satisfies a required level.
pub fn authority_satisfies(has: AuthorityLevel, needs: AuthorityLevel) -> bool {
    // Constitutional > Gated > Supervised > Autonomous
    let rank = |a: AuthorityLevel| -> u8 {
        match a {
            AuthorityLevel::Autonomous => 0,
            AuthorityLevel::Supervised => 1,
            AuthorityLevel::Gated => 2,
            AuthorityLevel::Constitutional => 3,
        }
    };
    rank(has) >= rank(needs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_hierarchy() {
        assert!(authority_satisfies(AuthorityLevel::Constitutional, AuthorityLevel::Gated));
        assert!(authority_satisfies(AuthorityLevel::Gated, AuthorityLevel::Supervised));
        assert!(!authority_satisfies(AuthorityLevel::Autonomous, AuthorityLevel::Gated));
    }
}
