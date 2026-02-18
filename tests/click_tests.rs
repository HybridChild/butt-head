mod common;

use butt_head::{ButtHead, Config, Event, ServiceTiming, TimeDuration};
use common::{CONFIG, TestDuration, TestInstant, new_button};

static ACTIVE_LOW_CONFIG: Config<TestDuration> = Config {
    active_low: true,
    click_timeout: TestDuration(300),
    hold_delay: TestDuration(500),
    hold_interval: TestDuration(200),
};

// --- Single click ---

#[test]
fn press_emits_press_event() {
    let mut button = new_button();

    let result = button.update(true, TestInstant::ms(0));

    assert_eq!(result.event, Some(Event::Press));
}

#[test]
fn release_before_hold_delay_emits_release_with_correct_duration() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    let result = button.update(false, TestInstant::ms(100));

    assert_eq!(result.event, Some(Event::Release { duration: TestDuration(100) }));
}

#[test]
fn single_click_emits_click_count_1_after_timeout() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));

    // Advance to just after click_timeout (300ms from release at t=100)
    let result = button.update(false, TestInstant::ms(400));

    assert_eq!(result.event, Some(Event::Click { count: 1 }));
}

#[test]
fn click_not_emitted_before_timeout_expires() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));

    // Only 299ms have passed since release — timeout is 300ms
    let result = button.update(false, TestInstant::ms(399));

    assert_eq!(result.event, None);
}

// --- Multi-click ---

#[test]
fn double_click_emits_click_count_2_after_timeout() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    // Second press within click_timeout window
    button.update(true, TestInstant::ms(200));
    button.update(false, TestInstant::ms(300));

    // Advance past click_timeout (300ms from second release at t=300)
    let result = button.update(false, TestInstant::ms(600));

    assert_eq!(result.event, Some(Event::Click { count: 2 }));
}

#[test]
fn triple_click_emits_click_count_3_after_timeout() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    button.update(true, TestInstant::ms(150));
    button.update(false, TestInstant::ms(200));
    button.update(true, TestInstant::ms(250));
    button.update(false, TestInstant::ms(300));

    // Advance past click_timeout from last release
    let result = button.update(false, TestInstant::ms(600));

    assert_eq!(result.event, Some(Event::Click { count: 3 }));
}

#[test]
fn second_press_within_timeout_resets_click_timeout_window() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    // Second press at t=250 (150ms after first release — within timeout)
    button.update(true, TestInstant::ms(250));
    button.update(false, TestInstant::ms(350));

    // 300ms after the second release (t=350) puts us at t=650
    // At t=649, not yet expired
    let result = button.update(false, TestInstant::ms(649));
    assert_eq!(result.event, None);

    // At t=650, timeout expired
    let result = button.update(false, TestInstant::ms(650));
    assert_eq!(result.event, Some(Event::Click { count: 2 }));
}

// --- Long press suppresses click ---

#[test]
fn long_press_does_not_emit_click() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    // Service at hold_delay (500ms)
    button.update(true, TestInstant::ms(500));
    // Release after hold
    button.update(false, TestInstant::ms(600));
    // Advance well past click_timeout
    let result = button.update(false, TestInstant::ms(1000));

    assert_eq!(result.event, None);
}

// --- Service timing ---

#[test]
fn idle_returns_idle_timing() {
    let mut button = new_button();

    let result = button.update(false, TestInstant::ms(0));

    assert_eq!(result.next_service, ServiceTiming::Idle);
}

#[test]
fn press_returns_hold_delay_timing() {
    let mut button = new_button();

    let result = button.update(true, TestInstant::ms(0));

    assert_eq!(result.next_service, ServiceTiming::Delay(CONFIG.hold_delay));
}

#[test]
fn release_returns_click_timeout_timing() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    let result = button.update(false, TestInstant::ms(100));

    assert_eq!(result.next_service, ServiceTiming::Delay(CONFIG.click_timeout));
}

#[test]
fn remaining_time_decreases_as_time_advances_in_wait_for_multi_click() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100)); // released_at = 100ms

    // 100ms into the 300ms click_timeout → 200ms remaining
    let result = button.update(false, TestInstant::ms(200));

    assert_eq!(result.next_service, ServiceTiming::Delay(CONFIG.click_timeout.saturating_sub(TestDuration(100))));
}

// --- Active low ---

#[test]
fn active_low_treats_low_signal_as_pressed() {
    let mut button = ButtHead::new(&ACTIVE_LOW_CONFIG);

    // With active_low, false = pressed
    let result = button.update(false, TestInstant::ms(0));

    assert_eq!(result.event, Some(Event::Press));
}

#[test]
fn active_low_treats_high_signal_as_released() {
    let mut button = ButtHead::new(&ACTIVE_LOW_CONFIG);
    button.update(false, TestInstant::ms(0)); // press (active low)

    let result = button.update(true, TestInstant::ms(100)); // release

    assert_eq!(result.event, Some(Event::Release { duration: TestDuration(100) }));
}
