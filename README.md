# butt-head

A `no_std` Rust library for processing button inputs in embedded systems. Transforms clean boolean pin states into button gesture events through a configurable state machine. Pure logic — no I/O, no HAL, no interrupts. The button counterpart to [pot-head](https://crates.io/crates/pot-head).

```toml
[dependencies]
butt-head = "0.1"
```

## What You Get

Feed in a pin state and a timestamp. Get back a structured event and a hint for when to call again.

```rust
let result = button.update(pin.is_high(), now());

match result.event {
    Some(Event::Click { count: 1 })                   => single_click(),
    Some(Event::Click { count: 2 })                   => double_click(),
    Some(Event::Hold { clicks_before: 0, level: 0 })  => hold_started(),
    Some(Event::Hold { clicks_before: 1, .. })         => click_then_hold(),
    Some(Event::Press)                                 => led_on(),
    Some(Event::Release { duration })                  => led_off(),
    _ => {}
}
```

Match on exactly the gestures you care about and ignore the rest. All gesture semantics are handled for you:

- A double-click fires a single `Click { count: 2 }` — never two separate single-clicks.
- A long press fires `Hold` events — never a `Click`.
- Click-then-hold is distinguished from plain hold via `clicks_before`.

## Events

| Event | When it fires |
| ----- | ------------- |
| `Press` | Immediately on every press edge |
| `Release { duration }` | Immediately on every release edge |
| `Click { count }` | After `click_timeout` with no further press; `count` reflects multi-clicks |
| `Hold { clicks_before, level }` | Repeatedly while held; `level` increments on each repeat |

## Power-Efficient Scheduling

Every call to `update()` returns a `ServiceTiming` hint telling you exactly when to call again:

```rust
match result.next_service {
    ServiceTiming::Delay(d) => timer.set_alarm(d),  // timer-driven wakeup
    ServiceTiming::Idle     => wait_for_interrupt(), // sleep until pin changes
    ServiceTiming::Immediate => {}                   // call again immediately
}
```

During idle your firmware sleeps until a pin interrupt fires. During a gesture the timer wakes you up at the exact moment the next event could fire. No polling loops, no wasted CPU cycles.

## Works Everywhere

butt-head is HAL-agnostic. Wire it up to your platform's time type by implementing two small traits:

```rust
pub trait TimeDuration: Copy + PartialEq + 'static {
    const ZERO: Self;
    fn as_millis(&self) -> u64;
    fn from_millis(millis: u64) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
}

pub trait TimeInstant: Copy {
    type Duration: TimeDuration;
    fn duration_since(&self, earlier: Self) -> Self::Duration;
    fn checked_add(self, duration: Self::Duration) -> Option<Self>;
    fn checked_sub(self, duration: Self::Duration) -> Option<Self>;
}
```

See [`examples/`](examples/README.md) for complete integrations with `std::time`, STM32 SysTick, and Embassy.

## Configuration

```rust
static CONFIG: Config<MyDuration> = Config {
    active_low: true,                              // pin low = pressed
    click_timeout: MyDuration::from_millis(300),   // multi-click window
    hold_delay: MyDuration::from_millis(500),      // time until first Hold fires
    hold_interval: MyDuration::from_millis(200),   // time between subsequent Holds
};

let mut button = ButtHead::new(&CONFIG);
```

Config lives as a `&'static` reference — zero runtime overhead, sits in flash on embedded targets.

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
