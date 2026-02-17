#![no_std]

mod time;
pub use time::{TimeDuration, TimeInstant};

mod event;
pub use event::Event;

mod service_timing;
pub use service_timing::ServiceTiming;

mod config;
pub use config::Config;

mod debouncer;

mod state_machine;

mod butt_head;
pub use butt_head::{ButtHead, UpdateResult};
