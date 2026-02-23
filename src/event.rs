use crate::{TimeDuration, TimeInstant};

/// A button event produced by the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event<D: TimeDuration, I: TimeInstant<Duration = D>> {
    /// The button was pressed. Fires immediately on press edge.
    /// `at` is the timestamp of the press, identical to what
    /// [`crate::ButtHead::press_instant`] returns during the pressed state.
    Press { at: I },

    /// The button was released. `duration` is the total time it was held.
    /// `click_follows` is `true` when a [`Event::Click`] will follow (the
    /// release ends a click gesture), and `false` when it ends a hold gesture.
    Release { duration: D, click_follows: bool },

    /// A complete click gesture (press + release, no hold).
    /// `count` starts at 1. Fires once after `click_timeout` expires with no
    /// further press. A double-click produces a single `Click { count: 2 }`.
    Click { count: u8 },

    /// The button is being held. Fires repeatedly at a configured interval.
    /// `clicks_before` is the number of clicks that preceded this hold
    /// (0 = plain hold, 1 = click+hold, 2 = double-click+hold, ...).
    /// `level` increments on each repeat (0 = first hold event, 1 = second, ...).
    Hold { clicks_before: u8, level: u8 },
}
