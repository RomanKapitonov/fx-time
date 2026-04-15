//! Semantic monotonic time model for the workspace.

#[cfg(feature = "defmt")]
use defmt::Format;

/// Primary elapsed-time type for the workspace.
///
/// Backed by a plain `u64` microsecond count. Arithmetic is saturating where
/// noted; the raw operators (`+`, `-`) behave like `u64` (panic on overflow /
/// underflow in debug builds, wrap in release) — callers that cannot guarantee
/// ordering should use [`Duration::saturating_add`] or
/// [`Instant::saturating_sub`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub(crate) u64);

impl Default for Duration {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Duration {
    pub const ZERO: Self = Self(0);

    pub const fn from_micros(us: u64) -> Self {
        Self(us)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }

    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub const fn from_millis(ms: u64) -> Self {
        Self::from_micros(ms.saturating_mul(1_000))
    }
}

impl core::ops::Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        Duration::from_micros(self.as_micros().saturating_mul(rhs as u64))
    }
}

impl core::ops::Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

/// Monotonic timestamp in microseconds since boot.
///
/// Raw wrap period: ~584,000 years. Use [`Instant::saturating_sub`] when
/// ordering cannot be guaranteed.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(pub(crate) u64);

impl Default for Instant {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Instant {
    pub const ZERO: Self = Self(0);

    pub const fn from_micros(us: u64) -> Self {
        Self(us)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Returns elapsed time from `earlier` to `self`, or `Duration::ZERO` if `self < earlier`.
    ///
    /// Use this instead of `self - earlier` whenever the caller cannot guarantee ordering —
    /// for example, when storing timestamps across frames.
    pub fn saturating_sub(self, earlier: Instant) -> Duration {
        if self.0 >= earlier.0 {
            Duration(self.0 - earlier.0)
        } else {
            Duration::ZERO
        }
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

    /// Panics in debug builds if `rhs > self`. Use [`Instant::saturating_sub`]
    /// when ordering is not guaranteed.
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
    fn instant_operators_delegate_correctly() {
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

    #[test]
    fn from_millis_converts_correctly() {
        assert_eq!(Duration::from_millis(1), Duration::from_micros(1_000));
        assert_eq!(Duration::from_millis(500), Duration::from_micros(500_000));
    }

    #[test]
    fn from_millis_saturates_on_overflow() {
        let large = Duration::from_millis(u64::MAX / 1_000 + 1);
        let _ = large.as_micros();
    }

    #[test]
    fn duration_mul_u32_scales_correctly() {
        let base = Duration::from_micros(1_000);
        assert_eq!(base * 3, Duration::from_micros(3_000));
        assert_eq!(base * 0, Duration::ZERO);
    }

    #[test]
    fn u32_mul_duration_is_commutative() {
        let base = Duration::from_micros(250);
        assert_eq!(4 * base, base * 4);
    }

    #[test]
    fn duration_mul_saturates_on_overflow() {
        let large = Duration::from_micros(u64::MAX / 2 + 1);
        let _ = large * 3;
    }

    #[test]
    fn saturating_sub_returns_correct_duration_when_lhs_gt_rhs() {
        let a = Instant::from_micros(2_000);
        let b = Instant::from_micros(750);
        assert_eq!(a.saturating_sub(b), Duration::from_micros(1_250));
    }

    #[test]
    fn saturating_sub_returns_zero_when_lhs_lt_rhs() {
        let earlier = Instant::from_micros(5_000);
        let later = Instant::from_micros(1_000);
        assert_eq!(later.saturating_sub(earlier), Duration::ZERO);
    }

    #[test]
    fn saturating_sub_returns_zero_when_equal() {
        let t = Instant::from_micros(42);
        assert_eq!(t.saturating_sub(t), Duration::ZERO);
    }

    #[test]
    fn instant_ord_compares_by_time() {
        let a = Instant::from_micros(100);
        let b = Instant::from_micros(200);
        assert!(a < b);
        assert!(b > a);
        assert!(a <= a);
        assert_eq!(a, Instant::from_micros(100));
    }
}
