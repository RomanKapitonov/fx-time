use crate::time::{FrameDelta, Instant, Microseconds, micros_to_frame_delta};

#[derive(Default)]
pub struct Clock {
    pub last_step: Option<Instant>,
}

impl Clock {
    #[inline(always)]
    pub fn new(now: Instant) -> Self {
        Self {
            last_step: Some(now),
        }
    }

    #[inline(always)]
    pub fn tick_us(&mut self, now: Instant) -> Microseconds {
        match self.last_step {
            Some(last) => {
                let dt_us = now.elapsed_since(last);
                self.last_step = Some(now);
                dt_us
            }
            None => {
                self.last_step = Some(now);
                Microseconds(0)
            }
        }
    }

    #[inline(always)]
    pub fn tick(&mut self, now: Instant) -> FrameDelta {
        micros_to_frame_delta(self.tick_us(now))
    }
}
