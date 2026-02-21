//! `no_std` button input processing for embedded systems.
//!
//! Transforms clean boolean pin states into gesture events — clicks, multi-clicks,
//! and holds — through a configurable state machine. Pure logic: no I/O, no HAL,
//! no interrupts required.

#![no_std]

mod time;
pub use time::{TimeDuration, TimeInstant};

mod event;
pub use event::Event;

mod service_timing;
pub use service_timing::ServiceTiming;

mod config;
pub use config::Config;

mod state_machine;

mod butt_head;
pub use butt_head::{ButtHead, UpdateResult};
