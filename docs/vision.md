# butt-head — Vision

A `no_std` Rust library for processing button inputs in embedded systems. Transforms raw boolean pin states into rich, debounced button events through a configurable processing pipeline. Pure logic — no I/O, no HAL, no interrupts. The button counterpart to [pot-head](https://crates.io/crates/pot-head).

## Design Philosophy

- **Pure abstraction** — no hardware coupling. Takes a `bool` + an instant, returns events.
- **Testable** — unit test with `true`/`false` and fake timestamps, no mocks needed.
- **Static config** — `&'static Config` stored in flash, compile-time validated via `const fn validate()`.
- **Minimal footprint** — documented RAM/flash usage, benchmarked on target platforms.
- **Feature-gated dependencies** — platform integrations (`std`, `embassy`, `defmt`) behind feature flags; all gesture logic always included.

## Time Abstraction

Borrows the trait pattern from [rgb-sequencer](https://crates.io/crates/rgb-sequencer). The user passes the current instant on each `update()` call. The crate never touches hardware timers or clocks.

```rust
pub trait TimeDuration: Copy + PartialEq {
    const ZERO: Self;
    fn as_millis(&self) -> u64;
    fn from_millis(millis: u64) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
}

pub trait TimeInstant: Copy {
    type Duration: TimeDuration;
    fn duration_since(&self, earlier: Self) -> Self::Duration;
}
```

Built-in implementations behind feature flags for `std::time::Instant` and `embassy_time::Instant`.

## Processing Pipeline

```
Raw bool → Debounce → State Machine → Events
```

The debouncer and state machine are decoupled stages. The debouncer filters raw input into clean press/release edges. The state machine only sees stable, debounced transitions and handles all gesture logic.

## Events

Four event types cover all button interactions:

```rust
pub enum Event<D: TimeDuration> {
    /// The button was pressed (fires immediately on debounced press edge).
    Press,

    /// The button was released. `duration` is the total time the button was held.
    Release { duration: D },

    /// A complete click gesture (press + release, no hold).
    /// `count` starts at 1. Fires immediately on each debounced release —
    /// no delay waiting for multi-click timeout. A double-click produces
    /// Click{1} on first release, then Click{2} on second release.
    Click { count: u8 },

    /// The button is being held. Fires repeatedly at a configured interval.
    /// `clicks_before` is the number of clicks that preceded this hold
    /// (0 = plain hold, 1 = click+hold, 2 = double-click+hold, ...).
    /// `level` increments on each repeat (0 = first hold event, 1 = second, ...).
    Hold { clicks_before: u8, level: u8 },
}
```

### Event semantics

- **Click fires immediately** — no waiting for a multi-click timeout. The first release
  emits `Click { count: 1 }`, the second `Click { count: 2 }`, etc. Zero added latency.
- **Hold suppresses Click** — if any `Hold` event was emitted during a press, the
  subsequent release does NOT emit `Click`. A long press is not a click.
- **Hold unifies press-and-hold with click-and-hold** — `clicks_before` distinguishes
  them. The user matches on the value they care about.
- **Hold level encodes duration** — the user can compute real duration as
  `hold_delay + level * hold_interval` if needed.

## Service Timing

Inspired by [rgb-sequencer](https://crates.io/crates/rgb-sequencer), `update()` returns both the event and a hint telling the caller when to call again:

```rust
pub struct UpdateResult<D: TimeDuration> {
    pub event: Option<Event<D>>,
    pub next_service: ServiceTiming<D>,
}

pub enum ServiceTiming<D> {
    /// Call back as soon as possible (mid-debounce, state needs re-evaluation).
    Immediate,
    /// Nothing will happen until at least this delay elapses.
    Delay(D),
    /// Button is idle — only call again when the input changes.
    Idle,
}
```

The contract: call `update()` when the pin changes state OR when the `next_service` delay expires. Missing a deadline is not catastrophic — the state machine catches up on the next call. Extra calls are harmless — unchanged input returns `Idle` with no event.

This enables optimal power usage: sleep during idle, wake on interrupt, precise timer-driven wakeups during interaction.

## Configuration

```rust
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
```

Compile-time validated via `const fn validate()`. Stored as `&'static Config` in flash.

## API

```rust
pub struct ButtHead<I: TimeInstant> {
    // ...
}

impl<I: TimeInstant> ButtHead<I> {
    pub fn new(config: &'static Config<I::Duration>) -> Self;
    pub fn update(&mut self, is_pressed: bool, now: I) -> UpdateResult<I::Duration>;

    // State introspection
    pub fn is_pressed(&self) -> bool;
    pub fn is_idle(&self) -> bool;
    pub fn current_hold_duration(&self, now: I) -> Option<I::Duration>;
}
```

### Usage example

```rust
static CONFIG: Config<MyDuration> = Config {
    active_low: true,
    debounce: MyDuration::from_millis(20),
    click_timeout: MyDuration::from_millis(300),
    hold_delay: MyDuration::from_millis(500),
    hold_interval: MyDuration::from_millis(200),
};

let mut button = ButtHead::new(&CONFIG);

loop {
    let result = button.update(pin.is_high(), now());

    if let Some(event) = result.event {
        match event {
            Event::Press => { /* immediate feedback, e.g. LED on */ }
            Event::Release { duration } => { /* e.g. LED off */ }
            Event::Click { count: 1 } => { /* single click */ }
            Event::Click { count: 2 } => { /* double click */ }
            Event::Hold { clicks_before: 0, level: 0 } => { /* hold started */ }
            Event::Hold { clicks_before: 0, level: 10.. } => { /* long hold */ }
            Event::Hold { clicks_before: 1, .. } => { /* click then hold */ }
            _ => {}
        }
    }

    match result.next_service {
        ServiceTiming::Delay(d) => sleep(d),
        ServiceTiming::Idle => wait_for_pin_interrupt(),
        ServiceTiming::Immediate => continue,
    }
}
```

## Architecture

### Debouncer (separate stage)

Filters raw boolean input into stable edges. Decoupled from the state machine so that
every press and release is uniformly debounced regardless of which state machine state
is active.

Uses a lockout approach: input changes are accepted immediately, then all further
changes are ignored for the debounce duration. This gives zero-latency response on the
first edge while rejecting subsequent bounce.

```rust
struct Debouncer<I: TimeInstant> {
    state: bool,
    lockout_until: Option<I>,
    duration: I::Duration,
}
```

The debouncer outputs a stable `bool`. The orchestrator detects edges by comparing the
current debounced value against the previous one.

### State Machine (gesture logic)

Operates on clean, debounced edges. Three states:

```
         Edge::Press              Edge::Release
 ┌──────┐ ──────────► ┌─────────┐ ───────────────► ┌───────────────────┐
 │ Idle │             │ Pressed │                  │ WaitForMultiClick │
 └──────┘ ◄────────   └─────────┘                  └───────────────────┘
     ▲    timeout │       ▲    hold delay/interval       │   │
     │            │       │    (emit Hold)               │   │
     │            │       └──────────────────────────────┘   │
     │            │                Edge::Press               │
     │            └──────────────────────────────────────────┘
     │                           timeout (finalize)
     │
     └─── Also: Release from Pressed goes directly to Idle
          when hold_level > 0 (long press is not a click)
```

#### Transition table

**Idle**

| Input        | Guard | Action       | Next State                               | next_service              |
| ------------ | ----- | ------------ | ---------------------------------------- | ------------------------- |
| Edge::Press  | —     | emit `Press` | Pressed { click_count: 0, hold_level: 0} | `Delay(hold_delay)`       |
| —            | —     | —            | Idle                                     | `Idle`                    |

**Pressed**

| Input         | Guard                           | Action                                      | Next State                                         | next_service              |
| ------------- | ------------------------------- | ------------------------------------------- | -------------------------------------------------- | ------------------------- |
| Edge::Release | hold_level > 0                  | emit `Release { duration }`                 | Idle                                               | `Idle`                    |
| Edge::Release | hold_level == 0                 | emit `Release { duration }`, emit `Click { count + 1 }` | WaitForMultiClick { count + 1 }         | `Delay(click_timeout)`    |
| —             | hold deadline reached           | emit `Hold { clicks_before: count, level }` | hold_level += 1                                    | `Delay(hold_interval)`    |
| —             | no deadline reached             | —                                           | (unchanged)                                        | `Delay(remaining)`        |

**WaitForMultiClick**

| Input        | Guard               | Action       | Next State                                 | next_service           |
| ------------ | ------------------- | ------------ | ------------------------------------------ | ---------------------- |
| Edge::Press  | —                   | emit `Press` | Pressed { click_count: count, hold_level: 0 } | `Delay(hold_delay)` |
| —            | timeout elapsed     | —            | Idle                                       | `Idle`                 |
| —            | timeout not elapsed | —            | (unchanged)                                | `Delay(remaining)`     |

## Feature Flags

| Feature   | What it enables                             | Dependency   |
| --------- | ------------------------------------------- | ------------ |
| `std`     | `TimeInstant` impl for `std::time::Instant` | std          |
| `embassy` | `TimeInstant` impl for `embassy_time`       | embassy-time |
| `defmt`   | Structured logging for events and state      | defmt        |
