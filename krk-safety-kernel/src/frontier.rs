//! Reachability Frontier
//!
//! Maintains the evolving boundary of the reachable quotient space itself.

use std::collections::HashSet;
use krk_quotient::ClassId;

#[derive(Debug, Clone)]
pub struct ReachSet {
    pub classes: HashSet<ClassId>,
}

impl ReachSet {
    pub fn new(classes: HashSet<ClassId>) -> Self {
        Self { classes }
    }
}

pub struct ReachFrontier {
    pub current: HashSet<ClassId>,
    pub next: HashSet<ClassId>,
}

impl ReachFrontier {
    pub fn new() -> Self {
        Self {
            current: HashSet::new(),
            next: HashSet::new(),
        }
    }

    /// Fixed-Point Step: μX. { [s0] } ∪ TQ(X)
    pub fn step(&mut self, classes: &[ClassId], quotient: &krk_quotient::QuotientLattice) -> ReachSet {
        for &c in classes {
            self.current.insert(c);
            for next in quotient.lift_transition(c) {
                self.next.insert(next);
            }
        }

        let result = ReachSet::new(self.next.clone());
        std::mem::swap(&mut self.current, &mut self.next);
        self.next.clear();

        result
    }

    pub fn is_exploding(&self) -> bool {
        self.current.len() > 10000 // Simple heuristic for symbolic explosion
    }
}
