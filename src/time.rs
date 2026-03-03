use fixed::{traits::ToFixed, types::U48F16};

type WideFx = fixed::types::I16F16;
type TimeFx = fixed::types::U16F16;
type AbsoluteTimeFx = U48F16;

#[cfg(feature = "defmt")]
use defmt::Format;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Microseconds(pub u64);

/// Absolute timestamp in seconds since boot.
///
/// Uses `U48F16` so it can represent the full `u64` microsecond range without saturating.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsoluteSeconds(pub AbsoluteTimeFx);

impl AbsoluteSeconds {
    pub const ZERO: Self = Self(AbsoluteTimeFx::ZERO);

    #[inline(always)]
    pub fn from_num<T: ToFixed>(v: T) -> Self {
        Self(AbsoluteTimeFx::from_num(v))
    }

    #[inline(always)]
    pub const fn whole_seconds(self) -> u64 {
        self.0.to_bits() >> 16
    }
}

#[cfg(feature = "defmt")]
impl Format for AbsoluteSeconds {
    fn format(&self, f: defmt::Formatter) {
        let bits = self.0.to_bits();
        let whole = bits >> 16;
        let frac_millis = (((bits & 0xFFFF) as u32).saturating_mul(1000) >> 16) as u16;

        defmt::write!(f, "{=u64}.", whole);
        if frac_millis < 10 {
            defmt::write!(f, "00{=u16}s", frac_millis);
        } else if frac_millis < 100 {
            defmt::write!(f, "0{=u16}s", frac_millis);
        } else {
            defmt::write!(f, "{=u16}s", frac_millis);
        }
    }
}

/// Monotonic timestamp in microseconds since boot.
///
/// Stored as a wrapping `u64` to keep it cheap (no `Duration` plumbing).
/// Wrap period: ~584k years.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Instant(pub u64);

impl Instant {
    #[inline(always)]
    pub const fn from_micros(us: u64) -> Self {
        Self(us)
    }

    #[inline(always)]
    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Wrapping delta in microseconds.
    #[inline(always)]
    pub fn elapsed_since(self, earlier: Instant) -> Microseconds {
        Microseconds(self.0.wrapping_sub(earlier.0))
    }

    /// Convert an absolute timestamp to seconds without saturation.
    ///
    /// Uses `AbsoluteSeconds` (`U48F16`) so full `u64` microsecond range is preserved.
    #[inline(always)]
    pub fn as_seconds(self) -> AbsoluteSeconds {
        micros_to_absolute_seconds(Microseconds(self.0))
    }

    /// Convert an absolute timestamp to `DurationSeconds` (`U16F16`), saturating at max.
    ///
    /// This is kept for APIs that explicitly operate in duration domain.
    #[inline(always)]
    pub fn as_duration_seconds(self) -> DurationSeconds {
        micros_to_duration(Microseconds(self.0))
    }

    /// Convert a delta between two timestamps to seconds (fixed point) with no division.
    ///
    /// Uses wrapping subtraction in microseconds and the same reciprocal conversion.
    /// This is unambiguous as long as true elapsed time is less than one wrap period (~584k years).
    #[inline(always)]
    pub fn dt_since(self, earlier: Instant) -> FrameDelta {
        micros_to_frame_delta(self.elapsed_since(earlier))
    }
}

#[cfg(feature = "defmt")]
impl Format for Instant {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "t+{=u64}us", self.0);
    }
}

/// 1 / 1_000_000 in U32F32, rounded.
///
/// Value: 2^32 / 1_000_000 ≈ 4294.967296 -> 4295.
const INV_US_TO_S_BITS: u64 = 4295;

/// Convert microseconds to absolute seconds in `AbsoluteSeconds` (`U48F16`).
#[inline(always)]
pub fn micros_to_absolute_seconds(us: Microseconds) -> AbsoluteSeconds {
    let bits_q48_16 = ((us.0 as u128) << 16) / 1_000_000u128;
    // Guaranteed to fit for all u64 microsecond inputs.
    let bits = bits_q48_16 as u64;
    AbsoluteSeconds(AbsoluteTimeFx::from_bits(bits))
}

/// Convert microseconds to seconds in `Seconds` (U16F16) without division.
#[inline(always)]
pub fn micros_to_seconds(us: Microseconds) -> Seconds {
    let us = us.0;
    let prod = (us as u128).saturating_mul(INV_US_TO_S_BITS as u128);
    let secs_q16_16 = prod >> 16;
    let clamped = secs_q16_16.min(u32::MAX as u128) as u32;
    Seconds(TimeFx::from_bits(clamped))
}

#[inline(always)]
pub fn micros_to_frame_delta(us: Microseconds) -> FrameDelta {
    FrameDelta::from(micros_to_seconds(us))
}

#[inline(always)]
pub fn micros_to_duration(us: Microseconds) -> DurationSeconds {
    DurationSeconds::from(micros_to_seconds(us))
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seconds(pub TimeFx);

impl Seconds {
    pub const ZERO: Self = Self(TimeFx::ZERO);

    #[inline(always)]
    pub fn from_num<T: ToFixed>(v: T) -> Self {
        Self(TimeFx::from_num(v))
    }

    #[inline(always)]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    #[inline(always)]
    pub fn is_zero(self) -> bool {
        self.0 == TimeFx::ZERO
    }

    #[inline(always)]
    pub fn is_positive(self) -> bool {
        self.0 > TimeFx::ZERO
    }

    #[inline(always)]
    pub fn as_wide(self) -> WideFx {
        WideFx::from_num(self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameDelta(pub TimeFx);

impl FrameDelta {
    pub const ZERO: Self = Self(TimeFx::ZERO);

    #[inline(always)]
    pub fn from_num<T: ToFixed>(v: T) -> Self {
        Self(TimeFx::from_num(v))
    }

    #[inline(always)]
    pub const fn from_seconds(seconds: Seconds) -> Self {
        Self(seconds.0)
    }

    #[inline(always)]
    pub const fn as_seconds(self) -> Seconds {
        Seconds(self.0)
    }

    #[inline(always)]
    pub fn is_zero(self) -> bool {
        self.0 == TimeFx::ZERO
    }

    #[inline(always)]
    pub fn is_positive(self) -> bool {
        self.0 > TimeFx::ZERO
    }
}

#[cfg(feature = "defmt")]
impl Format for FrameDelta {
    fn format(&self, f: defmt::Formatter) {
        let bits = self.0.to_bits() as u64;
        let millis = bits.saturating_mul(1000) >> 16;
        let whole = millis / 1000;
        let frac = (millis % 1000) as u16;

        defmt::write!(f, "{=u64}.", whole);
        if frac < 10 {
            defmt::write!(f, "00{=u16}s", frac);
        } else if frac < 100 {
            defmt::write!(f, "0{=u16}s", frac);
        } else {
            defmt::write!(f, "{=u16}s", frac);
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DurationSeconds(pub TimeFx);

impl DurationSeconds {
    pub const ZERO: Self = Self(TimeFx::ZERO);

    #[inline(always)]
    pub fn from_num<T: ToFixed>(v: T) -> Self {
        Self(TimeFx::from_num(v))
    }

    #[inline(always)]
    pub const fn from_seconds(seconds: Seconds) -> Self {
        Self(seconds.0)
    }

    #[inline(always)]
    pub const fn as_seconds(self) -> Seconds {
        Seconds(self.0)
    }

    #[inline(always)]
    pub const fn saturating_sub_frame(self, dt: FrameDelta) -> Self {
        Self(self.0.saturating_sub(dt.0))
    }

    #[inline(always)]
    pub fn is_positive(self) -> bool {
        self.0 > TimeFx::ZERO
    }
}

impl From<Seconds> for FrameDelta {
    #[inline(always)]
    fn from(value: Seconds) -> Self {
        Self(value.0)
    }
}

impl From<FrameDelta> for Seconds {
    #[inline(always)]
    fn from(value: FrameDelta) -> Self {
        Self(value.0)
    }
}

impl From<Seconds> for DurationSeconds {
    #[inline(always)]
    fn from(value: Seconds) -> Self {
        Self(value.0)
    }
}

impl From<DurationSeconds> for Seconds {
    #[inline(always)]
    fn from(value: DurationSeconds) -> Self {
        Self(value.0)
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = FrameDelta;

    #[inline(always)]
    fn sub(self, rhs: Instant) -> FrameDelta {
        self.dt_since(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_as_seconds_is_not_limited_by_u16f16_ceiling() {
        // 70,000 seconds (> 65,535.999s U16F16 ceiling)
        let t = Instant::from_micros(70_000_000_000);
        let s = t.as_seconds();
        assert_eq!(s.whole_seconds(), 70_000);
    }

    #[test]
    fn instant_as_duration_seconds_saturates_at_u16f16_max() {
        let t = Instant::from_micros(70_000_000_000);
        let s = t.as_duration_seconds();
        assert_eq!(s.0.to_bits(), u32::MAX);
    }
}
