//! Delta helper built on top of sampled `Instant` values.

use crate::time::{Duration, Instant};

#[derive(Default)]
pub struct DeltaClock {
    pub last_step: Option<Instant>,
}

impl DeltaClock {
    // #[inline(always)]
    pub fn new(now: Instant) -> Self {
        Self {
            last_step: Some(now),
        }
    }

    // #[inline(always)]
    pub fn tick(&mut self, now: Instant) -> Duration {
        match self.last_step {
            Some(last) => {
                let dt = now - last;
                self.last_step = Some(now);
                dt
            }
            None => {
                self.last_step = Some(now);
                Duration::ZERO
            }
        }
    }
}
