//! Shared monotonic time primitives for the workspace.
//!
//! Target redesign vocabulary:
//!
//! - `Instant`: monotonic timestamp in microseconds
//! - `Duration`: elapsed time in microseconds
//! - `DeltaClock`: helper that computes successive frame/step deltas
//!
//! Immediate direction:
//!
//! - keep microseconds as the internal storage unit
//! - keep the crate `no_std`
//! - move fixed-point projection concerns out of `fx-time`
//! - make wrapping behavior explicit in the future API
#![cfg_attr(not(test), no_std)]

pub mod clock;
pub mod time;

pub use clock::DeltaClock;
pub use time::{Duration, Instant};
