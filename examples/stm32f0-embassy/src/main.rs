#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_time::{Duration, Instant, with_timeout};
use {defmt_rtt as _, panic_probe as _};

use butt_head::{ButtHead, Config, Event, ServiceTiming, TimeDuration, TimeInstant};

// --- Time wrappers ---
//
// Examples compile as a separate crate, so the orphan rule prevents implementing
// foreign traits (TimeDuration, TimeInstant) for foreign types (embassy_time::*).
// Newtype wrappers around the embassy_time types sidestep this.

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
struct EmbassyDuration(Duration);

impl TimeDuration for EmbassyDuration {
    const ZERO: Self = EmbassyDuration(Duration::from_ticks(0));

    fn as_millis(&self) -> u64 {
        self.0.as_millis()
    }

    fn from_millis(millis: u64) -> Self {
        EmbassyDuration(Duration::from_millis(millis))
    }

    fn saturating_sub(self, other: Self) -> Self {
        EmbassyDuration(Duration::from_ticks(
            self.0.as_ticks().saturating_sub(other.0.as_ticks()),
        ))
    }
}

#[derive(Copy, Clone)]
struct EmbassyInstant(Instant);

impl TimeInstant for EmbassyInstant {
    type Duration = EmbassyDuration;

    fn duration_since(&self, earlier: Self) -> EmbassyDuration {
        EmbassyDuration(self.0 - earlier.0)
    }

    fn checked_add(self, duration: EmbassyDuration) -> Option<Self> {
        Some(EmbassyInstant(self.0 + duration.0))
    }

    fn checked_sub(self, duration: EmbassyDuration) -> Option<Self> {
        self.0.checked_sub(duration.0).map(EmbassyInstant)
    }
}

// --- Config ---

static CONFIG: Config<EmbassyDuration> = Config {
    active_low: false,
    click_timeout: EmbassyDuration(Duration::from_millis(120)),
    hold_delay: EmbassyDuration(Duration::from_millis(500)),
    hold_interval: EmbassyDuration(Duration::from_millis(300)),
    max_click_count: None,
};

// --- Main ---

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("=== butt-head stm32f0-embassy example ===");

    // PA5 — onboard LED (active high)
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);

    // PC13 — user button (active low, internal pull-up).
    // ExtiInput lets us sleep until a pin edge rather than polling.
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up);

    let mut butt_head = ButtHead::<EmbassyInstant>::new(&CONFIG);

    info!("Ready — press the user button (PC13)");

    loop {
        // Button is active low: is_low() == true when physically pressed.
        let is_pressed = button.is_low();
        let result = butt_head.update(is_pressed, EmbassyInstant(Instant::now()));

        if let Some(event) = result.event {
            info!("{:?}", event);

            match event {
                Event::Press => led.set_high(),
                Event::Release { .. } => led.set_low(),
                _ => {}
            }
        }

        // Use ServiceTiming to sleep exactly as long as needed.
        //
        // Delay: sleep until the next ButtHead deadline, but wake early
        //        if the button changes state (so edges are never missed).
        // Idle:  no deadline pending — sleep until the button changes state.
        // Immediate: call update() again without yielding.
        match result.next_service {
            ServiceTiming::Immediate => {}
            ServiceTiming::Delay(d) => {
                let _ = with_timeout(d.0, button.wait_for_any_edge()).await;
            }
            ServiceTiming::Idle => {
                button.wait_for_any_edge().await;
            }
        }
    }
}
