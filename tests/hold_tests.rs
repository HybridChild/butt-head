mod common;

use butt_head::{Event, ServiceTiming, TimeDuration};
use common::{CONFIG, TestDuration, TestInstant, new_button};

// --- Basic hold ---

#[test]
fn hold_fires_at_hold_delay() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    // Service exactly at hold_delay (500ms)
    let result = button.update(true, TestInstant::ms(500));

    assert_eq!(result.event, Some(Event::Hold { clicks_before: 0, level: 0 }));
}

#[test]
fn hold_not_fired_before_hold_delay() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    // 1ms before hold_delay
    let result = button.update(true, TestInstant::ms(499));

    assert_eq!(result.event, None);
}

#[test]
fn hold_level_increments_with_each_repeat() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    button.update(true, TestInstant::ms(500)); // level 0
    let result = button.update(true, TestInstant::ms(700)); // level 1
    assert_eq!(result.event, Some(Event::Hold { clicks_before: 0, level: 1 }));

    let result = button.update(true, TestInstant::ms(900)); // level 2
    assert_eq!(result.event, Some(Event::Hold { clicks_before: 0, level: 2 }));
}

#[test]
fn hold_not_fired_between_intervals() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(true, TestInstant::ms(500)); // level 0

    // 1ms before next interval
    let result = button.update(true, TestInstant::ms(699));

    assert_eq!(result.event, None);
}

#[test]
fn release_after_hold_emits_release_with_correct_duration() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(true, TestInstant::ms(500)); // trigger hold

    let result = button.update(false, TestInstant::ms(800));

    assert_eq!(result.event, Some(Event::Release { duration: TestDuration(800) }));
}

// --- Click-and-hold ---

#[test]
fn click_then_hold_sets_clicks_before_to_1() {
    let mut button = new_button();
    // First click
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    // Second press — hold it
    button.update(true, TestInstant::ms(200));

    // hold_delay after second press: t=200+500=700
    let result = button.update(true, TestInstant::ms(700));

    assert_eq!(result.event, Some(Event::Hold { clicks_before: 1, level: 0 }));
}

#[test]
fn double_click_then_hold_sets_clicks_before_to_2() {
    let mut button = new_button();
    // Two clicks
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    button.update(true, TestInstant::ms(150));
    button.update(false, TestInstant::ms(200));
    // Third press — hold it
    button.update(true, TestInstant::ms(250));

    // hold_delay after third press: t=250+500=750
    let result = button.update(true, TestInstant::ms(750));

    assert_eq!(result.event, Some(Event::Hold { clicks_before: 2, level: 0 }));
}

#[test]
fn click_then_hold_does_not_emit_click_after_release() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    button.update(true, TestInstant::ms(200));
    button.update(true, TestInstant::ms(700)); // hold fires
    button.update(false, TestInstant::ms(900)); // release

    // No click should emerge after the timeout
    let result = button.update(false, TestInstant::ms(1300));

    assert_eq!(result.event, None);
}

// --- Service timing ---

#[test]
fn after_hold_fires_returns_hold_interval_timing() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    let result = button.update(true, TestInstant::ms(500));

    assert_eq!(result.next_service, ServiceTiming::Delay(CONFIG.hold_interval));
}

#[test]
fn remaining_time_decreases_as_time_advances_while_pressed() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));

    // 200ms into the 500ms hold_delay → 300ms remaining
    let result = button.update(true, TestInstant::ms(200));

    assert_eq!(result.next_service, ServiceTiming::Delay(CONFIG.hold_delay.saturating_sub(TestDuration(200))));
}

#[test]
fn release_after_hold_returns_idle_timing() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(true, TestInstant::ms(500));

    let result = button.update(false, TestInstant::ms(600));

    assert_eq!(result.next_service, ServiceTiming::Idle);
}
