/// Abstraction for a duration of time.
pub trait TimeDuration: Copy + PartialEq {
    const ZERO: Self;
    fn as_millis(&self) -> u64;
    fn from_millis(millis: u64) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
}

/// Abstraction for a point in time.
pub trait TimeInstant: Copy {
    type Duration: TimeDuration;
    fn duration_since(&self, earlier: Self) -> Self::Duration;
}

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;
    use std::time;

    impl super::TimeDuration for time::Duration {
        const ZERO: Self = time::Duration::ZERO;

        fn as_millis(&self) -> u64 {
            time::Duration::as_millis(self) as u64
        }

        fn from_millis(millis: u64) -> Self {
            time::Duration::from_millis(millis)
        }

        fn saturating_sub(self, other: Self) -> Self {
            time::Duration::saturating_sub(&self, other)
        }
    }

    impl super::TimeInstant for time::Instant {
        type Duration = time::Duration;

        fn duration_since(&self, earlier: Self) -> Self::Duration {
            time::Instant::duration_since(self, earlier)
        }
    }
}

#[cfg(feature = "embassy")]
mod embassy_impl {
    use embassy_time;

    impl super::TimeDuration for embassy_time::Duration {
        const ZERO: Self = embassy_time::Duration::MIN;

        fn as_millis(&self) -> u64 {
            embassy_time::Duration::as_millis(self)
        }

        fn from_millis(millis: u64) -> Self {
            embassy_time::Duration::from_millis(millis)
        }

        fn saturating_sub(self, other: Self) -> Self {
            // embassy_time::Duration doesn't have saturating_sub,
            // so we implement it manually
            let a = self.as_ticks();
            let b = other.as_ticks();
            embassy_time::Duration::from_ticks(a.saturating_sub(b))
        }
    }

    impl super::TimeInstant for embassy_time::Instant {
        type Duration = embassy_time::Duration;

        fn duration_since(&self, earlier: Self) -> Self::Duration {
            embassy_time::Instant::duration_since(self, earlier)
        }
    }
}
