use crate::TimeDuration;
use crate::config::Config;
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
///
/// Expects clean, debounced input. If your button is subject to mechanical
/// bounce, debounce the signal before passing it to `update()`.
pub struct ButtHead<I: TimeInstant> {
    prev_input: bool,
    state_machine: StateMachine<I>,
    config: &'static Config<I::Duration>,
}

impl<I: TimeInstant> ButtHead<I> {
    pub fn new(config: &'static Config<I::Duration>) -> Self {
        Self {
            prev_input: false,
            state_machine: StateMachine::new(config),
            config,
        }
    }

    pub fn update(&mut self, is_pressed: bool, now: I) -> UpdateResult<I::Duration> {
        let input = if self.config.active_low {
            !is_pressed
        } else {
            is_pressed
        };

        let edge = if input != self.prev_input {
            self.prev_input = input;
            Some(if input { Edge::Press } else { Edge::Release })
        } else {
            None
        };

        let (event, next_service) = self.state_machine.update(edge, now);

        UpdateResult {
            event,
            next_service,
        }
    }
}
