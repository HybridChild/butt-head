#![no_std]
#![no_main]

use core::cell::Cell;

use cortex_m_rt::entry;
use critical_section::Mutex;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f0xx_hal::{pac, prelude::*};

use butt_head::{ButtHead, Config, Event, ServiceTiming, TimeDuration, TimeInstant};

// --- Millisecond counter (SysTick fires every 1ms) ---

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[cortex_m_rt::exception]
fn SysTick() {
    critical_section::with(|cs| {
        let c = MILLIS.borrow(cs);
        c.set(c.get().wrapping_add(1));
    });
}

fn now() -> HalInstant {
    critical_section::with(|cs| HalInstant(MILLIS.borrow(cs).get()))
}

// --- Time types ---

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct HalDuration(u32);

impl TimeDuration for HalDuration {
    const ZERO: Self = HalDuration(0);

    fn as_millis(&self) -> u64 {
        self.0 as u64
    }

    fn from_millis(millis: u64) -> Self {
        HalDuration(millis as u32)
    }

    fn saturating_sub(self, other: Self) -> Self {
        HalDuration(self.0.saturating_sub(other.0))
    }
}

#[derive(Copy, Clone)]
struct HalInstant(u32);

impl TimeInstant for HalInstant {
    type Duration = HalDuration;

    fn duration_since(&self, earlier: Self) -> HalDuration {
        // wrapping_sub handles counter rollover (~49 days)
        HalDuration(self.0.wrapping_sub(earlier.0))
    }

    fn checked_add(self, duration: HalDuration) -> Option<Self> {
        Some(HalInstant(self.0.wrapping_add(duration.0)))
    }

    fn checked_sub(self, duration: HalDuration) -> Option<Self> {
        Some(HalInstant(self.0.wrapping_sub(duration.0)))
    }
}

// --- Config ---

static CONFIG: Config<HalDuration> = Config {
    active_low: false,
    click_timeout: HalDuration(120),
    hold_delay: HalDuration(500),
    hold_interval: HalDuration(300),
    max_click_count: None,
};

// --- Entry ---

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("=== butt-head stm32f0 example ===");

    let mut dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.configure().freeze(&mut dp.FLASH);

    // SysTick: 1ms interrupts
    let sysclk = rcc.clocks.sysclk().0;
    cp.SYST
        .set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    cp.SYST.set_reload((sysclk / 1_000) - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    cp.SYST.enable_interrupt();

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    // PA5 — onboard LED (active high)
    let mut led = cortex_m::interrupt::free(|cs| gpioa.pa5.into_push_pull_output(cs));

    // PC13 — user button (active low, internal pull-up)
    // is_low() returns true when button is pressed, which we pass directly as is_pressed.
    let button = cortex_m::interrupt::free(|cs| gpioc.pc13.into_pull_up_input(cs));

    let mut butt_head = ButtHead::<HalInstant>::new(&CONFIG);

    rprintln!("Ready — press the user button (PC13)");

    loop {
        let is_pressed = button.is_low().unwrap_or(false);
        let result = butt_head.update(is_pressed, now());

        if let Some(event) = result.event {
            rprintln!("{:?}", event);

            match event {
                Event::Press => led.set_high().ok().unwrap(),
                Event::Release { .. } => led.set_low().ok().unwrap(),
                _ => {}
            }
        }

        // Sleep until the next SysTick (1ms). The button is sampled at 1kHz,
        // which is well within the timing requirements of all configured delays.
        // For Idle, we rely on SysTick to keep us from spinning; in a
        // power-sensitive application you would disable SysTick during Idle
        // and wake on a button EXTI interrupt instead.
        if result.next_service == ServiceTiming::Immediate {
            continue;
        }
        cortex_m::asm::wfi();
    }
}
