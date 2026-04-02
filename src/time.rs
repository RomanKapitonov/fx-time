//! Semantic monotonic time model for the workspace.

type RawDuration = fugit::MicrosDurationU64;
type RawInstant = fugit::TimerInstantU64<1_000_000>;

#[cfg(feature = "defmt")]
use defmt::Format;

/// Primary elapsed-time type for the workspace.
///
/// Backed by `fugit` microsecond ticks to keep arithmetic cheap and deterministic on MCUs.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub(crate) RawDuration);

impl Default for Duration {
    
    fn default() -> Self {
        Self::ZERO
    }
}

impl Duration {
    pub const ZERO: Self = Self(RawDuration::from_ticks(0));

    
    pub const fn from_micros(us: u64) -> Self {
        Self(RawDuration::from_ticks(us))
    }

    
    pub const fn as_micros(self) -> u64 {
        self.0.ticks()
    }

    
    pub const fn saturating_add(self, rhs: Self) -> Self {
        match self.0.checked_add(rhs.0) {
            Some(value) => Self(value),
            None => Self(RawDuration::from_ticks(u64::MAX)),
        }
    }
}

/// Monotonic timestamp in microseconds since boot.
///
/// Backed by `fugit` microsecond ticks while keeping a semantic workspace-facing API.
/// Raw wrap period remains ~584k years.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instant(pub(crate) RawInstant);

impl Default for Instant {
    
    fn default() -> Self {
        Self::ZERO
    }
}

impl Instant {
    pub const ZERO: Self = Self(RawInstant::from_ticks(0));

    
    pub const fn from_micros(us: u64) -> Self {
        Self(RawInstant::from_ticks(us))
    }

    
    pub const fn as_micros(self) -> u64 {
        self.0.ticks()
    }
}

#[cfg(feature = "defmt")]
impl Format for Instant {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "t+{=u64}us", self.as_micros());
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;

    
    fn sub(self, rhs: Instant) -> Duration {
        Duration(self.0 - rhs.0)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;

    
    fn add(self, rhs: Duration) -> Instant {
        Instant(self.0 + rhs.0)
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;

    
    fn sub(self, rhs: Duration) -> Instant {
        Instant(self.0 - rhs.0)
    }
}

impl core::ops::Add<Duration> for Duration {
    type Output = Duration;

    
    fn add(self, rhs: Duration) -> Duration {
        Duration(self.0 + rhs.0)
    }
}

impl core::ops::Sub<Duration> for Duration {
    type Output = Duration;

    
    fn sub(self, rhs: Duration) -> Duration {
        Duration(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_uses_microsecond_storage() {
        assert_eq!(Duration::from_micros(42).as_micros(), 42);
    }

    #[test]
    fn duration_saturating_add_behaves_as_expected() {
        let a = Duration::from_micros(2_000);
        let b = Duration::from_micros(750);

        assert_eq!(
            Duration::from_micros(u64::MAX - 1).saturating_add(Duration::from_micros(10)),
            Duration::from_micros(u64::MAX)
        );
        assert_eq!(a.saturating_add(b), Duration::from_micros(2_750));
    }

    #[test]
    fn instant_operators_delegate_to_fugit() {
        let earlier = Instant::from_micros(1_000);
        let later = Instant::from_micros(2_750);

        assert_eq!(later - earlier, Duration::from_micros(1_750));
    }

    #[test]
    fn instant_and_duration_operators_use_standard_shapes() {
        let base = Instant::from_micros(1_000);
        let dt = Duration::from_micros(250);

        assert_eq!(base + dt, Instant::from_micros(1_250));
        assert_eq!((base + dt) - dt, base);
        assert_eq!((base + dt) - base, dt);
    }
}
