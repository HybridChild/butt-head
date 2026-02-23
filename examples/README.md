# Examples

Three complete integrations showing how to wire butt-head into different environments.

---

## [`native`](native/)

Desktop demo using `std::time` and [`crossterm`](https://crates.io/crates/crossterm). Hold the left mouse button to simulate a button press. Events are printed to the terminal as they fire.

Good starting point for understanding the API without any embedded toolchain.

```sh
cd examples/native/
cargo run --release
```

---

## [`stm32f0`](stm32f0/)

Bare-metal STM32F0 (Nucleo-F072RB) using `cortex-m-rt` and a hand-rolled SysTick millisecond counter. The user button on PC13 drives the state machine; events and LED state are reported over RTT.

Demonstrates the minimal time trait implementation needed for a `no_std` target with no OS or async runtime.

```sh
cd examples/stm32f0/
cargo build --release
```

---

## [`stm32f0-embassy`](stm32f0-embassy/)

Same hardware as above, but using [Embassy](https://embassy.dev/) for async scheduling. `ExtiInput` replaces SysTick polling — each task sleeps until the next button edge or ButtHead deadline, whichever comes first, via `with_timeout`.

Demonstrates power-efficient scheduling with `ServiceTiming`: no busy-wait, no timer interrupts while idle.

Two binaries are provided:

### `single_button`

Single button on PC13 (on-board user button). Direct port of the bare-metal example to async Embassy.

```sh
cd examples/stm32f0-embassy/
cargo run --bin single_button --release
```

### `dual_button`

Two buttons coordinated across two independent tasks. Presses within 50 ms of each other are treated as a simultaneous combo, emitting `ButtonBothClick` / `ButtonBothHold` instead of the individual events.

**Wiring for button B:**

Connect a momentary push-button between **PA0** (Arduino header A0) and **GND**. The internal pull-up is enabled in firmware, so no external resistor is needed.

Button A (PC13) is the on-board user button — no extra wiring required.

```sh
cd examples/stm32f0-embassy/
cargo run --bin dual_button --release
```
