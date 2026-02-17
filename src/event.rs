use crate::TimeDuration;

/// A button event produced by the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event<D: TimeDuration> {
    /// The button was pressed. Fires immediately on debounced press edge.
    Press,

    /// The button was released. `duration` is the total time it was held.
    Release { duration: D },

    /// A complete click gesture (press + release, no hold).
    /// `count` starts at 1. Fires immediately on each debounced release.
    /// A double-click produces `Click { count: 1 }` on first release,
    /// then `Click { count: 2 }` on second release.
    Click { count: u8 },

    /// The button is being held. Fires repeatedly at a configured interval.
    /// `clicks_before` is the number of clicks that preceded this hold
    /// (0 = plain hold, 1 = click+hold, 2 = double-click+hold, ...).
    /// `level` increments on each repeat (0 = first hold event, 1 = second, ...).
    Hold { clicks_before: u8, level: u8 },
}
