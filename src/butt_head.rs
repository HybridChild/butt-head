use crate::TimeDuration;
use crate::config::Config;
use crate::debouncer::Debouncer;
use crate::event::Event;
use crate::service_timing::ServiceTiming;
use crate::state_machine::{Edge, StateMachine};
use crate::time::TimeInstant;

/// The result of a single `update()` call.
#[derive(Debug, Clone, Copy)]
pub struct UpdateResult<D: TimeDuration> {
    pub event: Option<Event<D>>,
    pub next_service: ServiceTiming<D>,
}

/// Button input processor.
pub struct ButtHead<I: TimeInstant> {
    debouncer: Debouncer<I>,
    prev_debounced: bool,
    state_machine: StateMachine<I>,
    active_low: bool,
}

impl<I: TimeInstant> ButtHead<I> {
    pub fn new(config: &Config<I::Duration>) -> Self {
        Self {
            debouncer: Debouncer::new(config.debounce),
            prev_debounced: false,
            state_machine: StateMachine::new(
                config.hold_delay,
                config.hold_interval,
                config.click_timeout,
            ),
            active_low: config.active_low,
        }
    }

    pub fn update(&mut self, raw_input: bool, now: I) -> UpdateResult<I::Duration> {
        let input = if self.active_low {
            !raw_input
        } else {
            raw_input
        };

        let (debounced, debounce_timing) = self.debouncer.update(input, now);

        let edge = if debounced != self.prev_debounced {
            self.prev_debounced = debounced;
            Some(if debounced {
                Edge::Press
            } else {
                Edge::Release
            })
        } else {
            None
        };

        let (event, sm_timing) = self.state_machine.update(edge, now);

        UpdateResult {
            event,
            next_service: debounce_timing.min(sm_timing),
        }
    }
}
