//! Trajectory Entropy Bounding
//!
//! Prevents adversarial soak fields from becoming infinite-dimensional.

pub struct TrajectoryWindow {
    pub max_depth: u32,
    pub max_branching: u32,
    pub max_concurrency: u32,
    pub max_pressure_entropy: f64,
}

impl TrajectoryWindow {
    pub fn new(depth: u32, branching: u32) -> Self {
        Self {
            max_depth: depth,
            max_branching: branching,
            max_concurrency: 10,
            max_pressure_entropy: 0.95,
        }
    }

    pub fn is_within_bounds(&self, current_depth: u32) -> bool {
        current_depth <= self.max_depth
    }
}
