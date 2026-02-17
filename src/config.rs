use crate::TimeDuration;

/// Configuration for a `ButtHead` instance.
pub struct Config<D: TimeDuration> {
    /// Invert input polarity (true = pin low means pressed).
    pub active_low: bool,

    /// Lockout duration after accepting a press or release.
    /// Input changes are accepted immediately, then ignored for this duration.
    pub debounce: D,

    /// Maximum gap between clicks to count as multi-click.
    pub click_timeout: D,

    /// Time before the first Hold event fires.
    pub hold_delay: D,

    /// Time between subsequent Hold events while held.
    pub hold_interval: D,
}
