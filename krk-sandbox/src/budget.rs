use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct ExecutionBudget {
    pub token_budget: TokenBudget,
    pub wall_clock_limit: Duration,
    pub start_time: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct TokenBudget {
    pub max_tokens: usize,
    pub consumed_tokens: usize,
}

impl ExecutionBudget {
    pub fn new(max_tokens: usize, ttl_ms: u64) -> Self {
        Self {
            token_budget: TokenBudget {
                max_tokens,
                consumed_tokens: 0,
            },
            wall_clock_limit: Duration::from_millis(ttl_ms),
            start_time: Instant::now(),
        }
    }

    pub fn consume_tokens(&mut self, amount: usize) -> Result<(), &'static str> {
        if self.token_budget.consumed_tokens + amount > self.token_budget.max_tokens {
            return Err("Token budget exceeded limit.");
        }
        self.token_budget.consumed_tokens += amount;
        Ok(())
    }

    pub fn check_time_budget(&self) -> Result<(), &'static str> {
        if self.start_time.elapsed() > self.wall_clock_limit {
            return Err("Wall-clock TTL exceeded execution limit.");
        }
        Ok(())
    }

    pub fn check_limits(&self) -> Result<(), &'static str> {
        if self.token_budget.consumed_tokens > self.token_budget.max_tokens {
            return Err("Token budget exceeded.");
        }
        self.check_time_budget()
    }
}
