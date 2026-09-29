//! Control Scheduler — Tick-based execution ordering
//!
//! Ensures control decisions are sequenced correctly relative to PSC ticks.

/// Scheduling priority for control actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchedulePriority {
    /// Safety-critical — runs before any other action.
    Critical = 0,
    /// Standard operational action.
    Normal = 1,
    /// Low-priority background task.
    Background = 2,
}

/// A scheduled control action.
#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub priority: SchedulePriority,
    pub action_id: String,
    pub tick_deadline: u64,
}

/// Simple priority queue for control actions.
pub struct ControlScheduler {
    queue: Vec<ScheduledAction>,
}

impl ControlScheduler {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn enqueue(&mut self, action: ScheduledAction) {
        self.queue.push(action);
        self.queue.sort_by_key(|a| a.priority);
    }

    pub fn drain_ready(&mut self, current_tick: u64) -> Vec<ScheduledAction> {
        let (ready, pending): (Vec<_>, Vec<_>) = self.queue
            .drain(..)
            .partition(|a| a.tick_deadline <= current_tick);
        self.queue = pending;
        ready
    }
}
