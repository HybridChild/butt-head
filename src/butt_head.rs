use crate::TimeDuration;
use crate::config::Config;
use crate::event::Event;
use crate::service_timing::ServiceTiming;
use crate::state_machine::{Edge, StateMachine};
use crate::time::TimeInstant;

/// The result of a single `update()` call.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UpdateResult<D: TimeDuration, I: TimeInstant<Duration = D>> {
    /// The event produced by this update, if any.
    pub event: Option<Event<D, I>>,
    /// When to call `update()` again. See [`ServiceTiming`].
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
    /// Creates a new `ButtHead` instance with the given configuration.
    pub fn new(config: &'static Config<I::Duration>) -> Self {
        Self {
            prev_input: false,
            state_machine: StateMachine::new(config),
            config,
        }
    }

    /// Returns `true` if the button is currently physically pressed.
    pub fn is_pressed(&self) -> bool {
        self.prev_input
    }

    /// Returns the instant at which the button was last pressed, or `None` if
    /// the button is not currently pressed.
    pub fn press_instant(&self) -> Option<I> {
        self.state_machine.pressed_at()
    }

    /// Returns how long the button has been continuously held, or `None` if it
    /// is not currently pressed.
    pub fn pressed_duration(&self, now: I) -> Option<I::Duration> {
        self.state_machine
            .pressed_at()
            .map(|at| now.duration_since(at))
    }

    /// Cancels the pending `Click` event when the state machine is in
    /// `WaitForMultiClick`. Returns `true` if cancelled, `false` if the state
    /// machine was not waiting for a click (nothing to cancel).
    ///
    /// Call this from an `Event::Release { click_follows: true, .. }` handler
    /// to suppress the upcoming `Click` (e.g. when the release was part of a
    /// multi-button combo gesture). The state machine returns to `Idle` and no
    /// `Click` event will fire.
    pub fn cancel_pending_click(&mut self) -> bool {
        self.state_machine.cancel_pending_click()
    }

    /// Advances the state machine.
    ///
    /// `is_pressed` is the raw pin state (before active-low inversion).
    /// `now` is the current timestamp. Returns the resulting event and the
    /// recommended time for the next call.
    pub fn update(&mut self, is_pressed: bool, now: I) -> UpdateResult<I::Duration, I> {
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
