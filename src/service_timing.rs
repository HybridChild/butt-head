use crate::TimeDuration;

/// Indicates when `update()` should next be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTiming<D: TimeDuration> {
    /// Call back as soon as possible.
    Immediate,

    /// Nothing will happen until at least this delay elapses.
    Delay(D),

    /// Button is idle. Only call again when the input changes.
    Idle,
}

impl<D: TimeDuration> ServiceTiming<D> {
    /// Returns the sooner of two service timings.
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (ServiceTiming::Immediate, _) | (_, ServiceTiming::Immediate) => {
                ServiceTiming::Immediate
            }
            (ServiceTiming::Delay(a), ServiceTiming::Delay(b)) => {
                if a.as_millis() <= b.as_millis() {
                    ServiceTiming::Delay(a)
                } else {
                    ServiceTiming::Delay(b)
                }
            }
            (ServiceTiming::Delay(d), ServiceTiming::Idle)
            | (ServiceTiming::Idle, ServiceTiming::Delay(d)) => ServiceTiming::Delay(d),
            (ServiceTiming::Idle, ServiceTiming::Idle) => ServiceTiming::Idle,
        }
    }
}
