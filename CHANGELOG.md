# Changelog

## [0.2.0] - 2026-02-23

### Added

- `ButtHead::press_instant()` — returns the `TimeInstant` at which the button was last pressed, or `None` if not currently pressed; enables multi-button combo detection
- `Event::Release::click_follows: bool` — `true` when a `Click` event will follow (click gesture), `false` when it ends a hold gesture
- `ButtHead::cancel_pending_click()` — cancels the pending `Click` event when the state machine is in `WaitForMultiClick`, transitioning back to `Idle`; call from an `Event::Release { click_follows: true }` handler to suppress a click that was part of a combo gesture
- `Event::Press { at: I }` — the press timestamp is now carried directly in the event, avoiding the need to call `press_instant()` immediately after handling a `Press` event

### Changed

- `Event<D>` is now `Event<D, I>` — the `Press` variant carries the press instant, so `I` is part of the event type; match arms that previously used `Event::Press` must now use `Event::Press { .. }` or `Event::Press { at }`
- `UpdateResult<D>` is now `UpdateResult<D, I>` to match the updated `Event` type
- `TimeInstant` now requires `PartialEq`

## [0.1.0] - 2026-02-21

Initial public release.

### Added

- `ButtHead` state machine processing boolean pin states into gesture events
- `Event` enum: `Press`, `Release { duration }`, `Click { count }`, `Hold { clicks_before, level }`
- `Config` with `active_low`, `click_timeout`, `hold_delay`, `hold_interval`, `max_click_count`
- `max_click_count` — emit `Click` immediately when a click cap is reached, without waiting for `click_timeout`
- `ButtHead::is_pressed()` and `pressed_duration()` for multi-button combo support
- `ServiceTiming` scheduling hint (`Idle`, `Delay`, `Immediate`) for power-efficient firmware loops
- `TimeDuration` and `TimeInstant` traits for HAL-agnostic time integration
- Optional `defmt` feature for structured RTT logging
- `no_std` by default; verified on `thumbv6m-none-eabi` and `thumbv7em-none-eabihf`
- Examples: native (`std::time`), STM32 bare-metal (SysTick), STM32 Embassy
