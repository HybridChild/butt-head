/// Abstraction for a duration of time.
pub trait TimeDuration: Copy + PartialEq + 'static {
    const ZERO: Self;
    fn as_millis(&self) -> u64;
    fn from_millis(millis: u64) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
}

/// Abstraction for a point in time.
pub trait TimeInstant: Copy {
    type Duration: TimeDuration;
    fn duration_since(&self, earlier: Self) -> Self::Duration;
    fn checked_add(self, duration: Self::Duration) -> Option<Self>;
    fn checked_sub(self, duration: Self::Duration) -> Option<Self>;
}
