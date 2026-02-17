use crate::{ServiceTiming, TimeDuration, TimeInstant};

pub(crate) struct Debouncer<I: TimeInstant> {
    state: bool,
    lockout_until: Option<I>,
    duration: I::Duration,
}

impl<I: TimeInstant> Debouncer<I> {
    pub fn new(duration: I::Duration) -> Self {
        Self {
            state: false,
            lockout_until: None,
            duration,
        }
    }

    /// Returns (debounced_value, next_service).
    pub fn update(&mut self, raw: bool, now: I) -> (bool, ServiceTiming<I::Duration>) {
        if let Some(until) = self.lockout_until {
            let elapsed = now.duration_since(until);
            if elapsed.as_millis() < self.duration.as_millis() {
                let remaining = self.duration.saturating_sub(elapsed);
                return (self.state, ServiceTiming::Delay(remaining));
            }
            self.lockout_until = None;
        }

        if raw != self.state {
            self.state = raw;
            self.lockout_until = Some(now);
            (self.state, ServiceTiming::Delay(self.duration))
        } else {
            (self.state, ServiceTiming::Idle)
        }
    }
}
