# butt-head

[![Platform](https://img.shields.io/badge/platform-no_std-blue)](https://github.com/HybridChild/butt-head)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](https://github.com/HybridChild/butt-head#license)

A `no_std` Rust library for processing button inputs in embedded systems. Transforms clean boolean pin states into button gesture events through a configurable state machine. Pure logic — no I/O, no HAL, no interrupts. The button counterpart to [pot-head](https://crates.io/crates/pot-head).

## Scope

**butt-head** handles gesture recognition: single and multi-click detection, hold detection, and multi-click sequences. It operates on clean boolean pin states and does **not** perform debouncing. Debounce is a hardware-level concern that depends on sampling rate and electrical characteristics — it belongs in your HAL or input driver, before the state reaches this library.

## What You Get

Feed in a pin state and a timestamp. Get back a structured event and a hint for when to call again.

```rust
let result = button.update(pin.is_high(), now());

match result.event {
    Some(Event::Click { count: 1 })                   => single_click(),
    Some(Event::Click { count: 2 })                   => double_click(),
    Some(Event::Hold { clicks_before: 0, level: 0 })  => hold_started(),
    Some(Event::Hold { clicks_before: 1, .. })        => click_then_hold(),
    Some(Event::Press)                                => led_on(),
    Some(Event::Release { duration })                 => led_off(),
    _ => {}
}
```

Match on exactly the gestures you care about and ignore the rest. All gesture semantics are handled for you:

- A double-click fires a single `Click { count: 2 }` — never two separate single-clicks.
- A long press fires `Hold` events — never a `Click`.
- Click-then-hold is distinguished from plain hold via `clicks_before`.

For multi-button combos, `is_pressed()` and `pressed_duration(now)` let you query button state directly without waiting for an event.

## Configuration

```rust
static CONFIG: Config<MyDuration> = Config {
    active_low: true,                              // pin low = pressed
    click_timeout: MyDuration::from_millis(300),   // multi-click window
    hold_delay: MyDuration::from_millis(500),      // time until first Hold fires
    hold_interval: MyDuration::from_millis(200),   // time between subsequent Holds
    max_click_count: None,                         // None = always wait for click_timeout
};

let mut button = ButtHead::new(&CONFIG);
```

Config lives as a `&'static` reference — zero runtime overhead, sits in flash on embedded targets.

`max_click_count` lets you short-circuit the `click_timeout` wait:

- `None` — always wait for `click_timeout` to expire before emitting (default).
- `Some(1)` — emit `Click` on every release immediately, with no timeout wait.
- `Some(n)` — emit immediately once the n-th click in a sequence lands.

## Events

| Event | When it fires |
| ----- | ------------- |
| `Press` | Immediately on every press edge |
| `Release { duration }` | Immediately on every release edge |
| `Click { count }` | After `click_timeout` with no further press, or immediately when `max_click_count` is reached; `count` reflects multi-clicks |
| `Hold { clicks_before, level }` | Repeatedly while held; `level` increments on each repeat |

## Power-Efficient Scheduling

Every call to `update()` returns a `ServiceTiming` hint telling you exactly when to call again:

```rust
match result.next_service {
    ServiceTiming::Delay(d) => timer.set_alarm(d),   // timer-driven wakeup
    ServiceTiming::Idle     => wait_for_interrupt(), // sleep until pin changes
    ServiceTiming::Immediate => {}                   // call again immediately
}
```

During idle your firmware sleeps until a pin interrupt fires. During a gesture the timer wakes you up at the exact moment the next event could fire. No polling loops, no wasted CPU cycles.

## Works Everywhere

**butt-head** is HAL-agnostic. Integrate it by implementing two small traits — `TimeDuration` and `TimeInstant` — for your platform's time types. See [`examples/`](examples/README.md) for complete integrations with `std::time`, STM32 SysTick, and Embassy.

## Feature Flags

| Feature | What it enables |
| ------- | --------------- |
| `defmt` | `defmt::Format` on all public types for structured RTT logging |

## Examples

| Example | Description |
| ------- | ----------- |
| [`examples/native`](examples/native) | Desktop demo using `std::time` and `crossterm` |
| [`examples/stm32f0`](examples/stm32f0) | Bare-metal STM32F0 with SysTick timer |
| [`examples/stm32f0-embassy`](examples/stm32f0-embassy) | STM32F0 with Embassy async/await |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
