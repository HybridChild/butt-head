use crate::TimeDuration;

/// Configuration for a `ButtHead` instance.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<D: TimeDuration> {
    /// Invert input polarity (true = pin low means pressed).
    pub active_low: bool,

    /// Maximum gap between clicks to count as multi-click.
    pub click_timeout: D,

    /// Time before the first Hold event fires.
    pub hold_delay: D,

    /// Time between subsequent Hold events while held.
    pub hold_interval: D,
}
