mod common;

use common::{TestDuration, TestInstant, new_button};

// --- is_pressed ---

#[test]
fn is_pressed_false_initially() {
    let button = new_button();
    assert!(!button.is_pressed());
}

#[test]
fn is_pressed_true_after_press() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    assert!(button.is_pressed());
}

#[test]
fn is_pressed_false_after_release() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    assert!(!button.is_pressed());
}

// --- pressed_duration ---

#[test]
fn pressed_duration_none_when_idle() {
    let button = new_button();
    assert_eq!(button.pressed_duration(TestInstant::ms(0)), None);
}

#[test]
fn pressed_duration_some_after_press() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    assert_eq!(
        button.pressed_duration(TestInstant::ms(200)),
        Some(TestDuration(200))
    );
}

#[test]
fn pressed_duration_none_after_release() {
    let mut button = new_button();
    button.update(true, TestInstant::ms(0));
    button.update(false, TestInstant::ms(100));
    assert_eq!(button.pressed_duration(TestInstant::ms(200)), None);
}
