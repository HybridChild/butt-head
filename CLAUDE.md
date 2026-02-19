# butt-head

A `no_std` button input processing library for embedded systems. HAL-agnostic: pure logic, no I/O.

## Project structure

- `src/` — Core library (state machine, events, config, time traits)
- `examples/` — Native (std), STM32 bare-metal, STM32 Embassy
- `tests/` — Integration tests with mock time (`TestInstant`, `TestDuration`)

## Commands

```sh
scripts/format.sh        # fmt all subprojects
scripts/cleanup.sh       # cargo clean all subprojects
cargo fmt --all
cargo clippy --lib --all-features -- -D warnings
cargo test --no-default-features
cargo doc --no-deps --all-features
# no_std check:
cargo build --target thumbv6m-none-eabi --lib --release --no-default-features
```

## Conventions

- `no_std` by default; `defmt` is an optional feature
- Config uses `&'static` references (lives in flash, zero runtime overhead)
- Edition: 2024
- `button.update(pin_state, now)` returns `UpdateResult { event, next_service }`
- `ServiceTiming` tells the caller when to call again — no polling required
