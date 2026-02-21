# Changelog

## [0.1.0] - Unreleased

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
