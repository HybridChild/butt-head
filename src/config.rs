use crate::TimeDuration;

/// Configuration for a `ButtHead` instance.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<D: TimeDuration> {
    /// Input polarity (true = pin low means pressed).
    pub active_low: bool,

    /// Maximum gap between clicks to count as multi-click.
    pub click_timeout: D,

    /// Time before the first Hold event fires.
    pub hold_delay: D,

    /// Time between subsequent Hold events while held.
    pub hold_interval: D,

    /// Maximum number of clicks to accumulate before emitting a `Click` event
    /// immediately (without waiting for `click_timeout` to expire).
    ///
    /// - `None` — unbounded; always wait for `click_timeout` (default behaviour).
    /// - `Some(1)` — emit `Click` on every release with no timeout wait.
    /// - `Some(n)` — emit immediately once the n-th click in a sequence lands.
    pub max_click_count: Option<u8>,
}
