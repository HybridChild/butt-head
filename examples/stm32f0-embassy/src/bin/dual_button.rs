//! Dual-button example for NUCLEO-F072RB.
//!
//! Each button runs in its own embassy task and reports events to a shared
//! `input_handler_task` via a channel.  Presses that occur within 50 ms of
//! each other are treated as a simultaneous "both" press.
//!
//! Wiring
//! ------
//!  - Button A : PC13  — on-board user button (active low, internal pull-up)
//!  - Button B : PA0   — Arduino header A0; connect a button between PA0 and GND

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, with_timeout};
use {defmt_rtt as _, panic_probe as _};

use butt_head::{ButtHead, Config, Event, ServiceTiming, TimeDuration, TimeInstant};

// ---------------------------------------------------------------------------
// Time wrappers
// ---------------------------------------------------------------------------

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

#[derive(Copy, Clone, PartialEq)]
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

// ---------------------------------------------------------------------------
// InputEvent
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, defmt::Format)]
enum InputEvent {
    ButtonAClick,
    ButtonBClick,
    /// Fired once when both buttons are pressed within the combo window.
    ButtonBothClick,
    ButtonAHold,
    ButtonBHold,
    /// Fired on every hold interval while both buttons remain pressed.
    ButtonBothHold,
}

// ---------------------------------------------------------------------------
// Channel and combo-detection statics
// ---------------------------------------------------------------------------

static INPUT_CHANNEL: Channel<CriticalSectionRawMutex, InputEvent, 8> = Channel::new();

// Lower 32 bits of the embassy tick counter at the most recent press, or
// u32::MAX as a sentinel meaning "not pressed".  Written on Press, cleared on
// Release.  AtomicU64 is unavailable on Cortex-M0; the lower 32 bits overflow
// every ~131 s at 32 768 Hz, making a false positive during the 50 ms combo
// window negligibly unlikely.
static BUTTON_A_PRESS_TICKS: AtomicU32 = AtomicU32::new(u32::MAX);
static BUTTON_B_PRESS_TICKS: AtomicU32 = AtomicU32::new(u32::MAX);

// Set by whichever task detects the combo (presses second within the window).
// Cleared when both buttons have been released.
static COMBINED: AtomicBool = AtomicBool::new(false);

// Two presses within this window are treated as simultaneous.
const COMBO_WINDOW_TICKS: u32 = Duration::from_millis(50).as_ticks() as u32;

// ---------------------------------------------------------------------------
// Button config
// ---------------------------------------------------------------------------

static BUTTON_CONFIG: Config<EmbassyDuration> = Config {
    active_low: true,
    click_timeout: EmbassyDuration(Duration::from_millis(120)),
    hold_delay: EmbassyDuration(Duration::from_millis(500)),
    hold_interval: EmbassyDuration(Duration::from_millis(300)),
    max_click_count: Some(1),
};

// ---------------------------------------------------------------------------
// Interrupt bindings
// ---------------------------------------------------------------------------

bind_interrupts!(struct Irqs {
    EXTI0_1  => exti::InterruptHandler<interrupt::typelevel::EXTI0_1>;
    EXTI4_15 => exti::InterruptHandler<interrupt::typelevel::EXTI4_15>;
});

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

/// Button A — PC13 (on-board user button, active low).
#[embassy_executor::task]
async fn button_a_task(mut button: ExtiInput<'static>) {
    let sender = INPUT_CHANNEL.sender();
    let mut bh: ButtHead<EmbassyInstant> = ButtHead::new(&BUTTON_CONFIG);

    loop {
        let result = bh.update(button.is_low(), EmbassyInstant(Instant::now()));

        match result.event {
            Some(Event::Press { at }) => {
                let ticks = at.0.as_ticks() as u32;
                BUTTON_A_PRESS_TICKS.store(ticks, Ordering::Relaxed);
                let other = BUTTON_B_PRESS_TICKS.load(Ordering::Relaxed);
                if other != u32::MAX && ticks.abs_diff(other) <= COMBO_WINDOW_TICKS {
                    COMBINED.store(true, Ordering::Relaxed);
                    sender.send(InputEvent::ButtonBothClick).await;
                }
            }
            Some(Event::Release { click_follows, .. }) => {
                if COMBINED.load(Ordering::Relaxed) && click_follows {
                    bh.cancel_pending_click();
                }
                BUTTON_A_PRESS_TICKS.store(u32::MAX, Ordering::Relaxed);
                if BUTTON_B_PRESS_TICKS.load(Ordering::Relaxed) == u32::MAX {
                    COMBINED.store(false, Ordering::Relaxed);
                }
            }
            Some(Event::Click { .. }) => sender.send(InputEvent::ButtonAClick).await,
            Some(Event::Hold { .. }) => {
                // Button A is responsible for emitting ButtonBothHold so that
                // button B's task can simply skip it and avoid duplicates.
                if COMBINED.load(Ordering::Relaxed) {
                    sender.send(InputEvent::ButtonBothHold).await;
                } else {
                    sender.send(InputEvent::ButtonAHold).await;
                }
            }
            None => {}
        }

        match result.next_service {
            ServiceTiming::Immediate => {}
            ServiceTiming::Idle => button.wait_for_any_edge().await,
            ServiceTiming::Delay(d) => {
                let _ = with_timeout(d.0, button.wait_for_any_edge()).await;
            }
        }
    }
}

/// Button B — PA0 (Arduino A0, active low).
#[embassy_executor::task]
async fn button_b_task(mut button: ExtiInput<'static>) {
    let sender = INPUT_CHANNEL.sender();
    let mut bh: ButtHead<EmbassyInstant> = ButtHead::new(&BUTTON_CONFIG);

    loop {
        let result = bh.update(button.is_low(), EmbassyInstant(Instant::now()));

        match result.event {
            Some(Event::Press { at }) => {
                let ticks = at.0.as_ticks() as u32;
                BUTTON_B_PRESS_TICKS.store(ticks, Ordering::Relaxed);
                let other = BUTTON_A_PRESS_TICKS.load(Ordering::Relaxed);
                if other != u32::MAX && ticks.abs_diff(other) <= COMBO_WINDOW_TICKS {
                    COMBINED.store(true, Ordering::Relaxed);
                    sender.send(InputEvent::ButtonBothClick).await;
                }
            }
            Some(Event::Release { click_follows, .. }) => {
                if COMBINED.load(Ordering::Relaxed) && click_follows {
                    bh.cancel_pending_click();
                }
                BUTTON_B_PRESS_TICKS.store(u32::MAX, Ordering::Relaxed);
                if BUTTON_A_PRESS_TICKS.load(Ordering::Relaxed) == u32::MAX {
                    COMBINED.store(false, Ordering::Relaxed);
                }
            }
            Some(Event::Click { .. }) => sender.send(InputEvent::ButtonBClick).await,
            Some(Event::Hold { .. }) => {
                // Button A handles ButtonBothHold; button B skips it to avoid
                // duplicate events on the channel.
                if !COMBINED.load(Ordering::Relaxed) {
                    sender.send(InputEvent::ButtonBHold).await;
                }
            }
            None => {}
        }

        match result.next_service {
            ServiceTiming::Immediate => {}
            ServiceTiming::Idle => button.wait_for_any_edge().await,
            ServiceTiming::Delay(d) => {
                let _ = with_timeout(d.0, button.wait_for_any_edge()).await;
            }
        }
    }
}

/// Receives InputEvents and logs them.  Replace with application logic.
#[embassy_executor::task]
async fn input_handler_task() {
    let receiver = INPUT_CHANNEL.receiver();
    loop {
        let event = receiver.receive().await;
        info!("input: {:?}", event);
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("=== butt-head dual-button example ===");
    info!("Button A: PC13 (user button)");
    info!("Button B: PA0  (Arduino A0 — connect button between PA0 and GND)");

    // PC13 — on-board user button (active low, pull-up built in to NUCLEO).
    let button_a = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);
    // PA0 — Arduino A0 header; drive low when pressed via an external button.
    let button_b = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);

    spawner.spawn(button_a_task(button_a)).unwrap();
    spawner.spawn(button_b_task(button_b)).unwrap();
    spawner.spawn(input_handler_task()).unwrap();
}
