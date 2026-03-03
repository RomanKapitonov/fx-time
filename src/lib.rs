#![cfg_attr(not(test), no_std)]

pub mod clock;
pub mod time;

pub use clock::Clock;
pub use time::{
    AbsoluteSeconds,
    DurationSeconds,
    FrameDelta,
    Instant,
    Microseconds,
    Seconds,
    micros_to_absolute_seconds,
    micros_to_duration,
    micros_to_frame_delta,
    micros_to_seconds,
};
