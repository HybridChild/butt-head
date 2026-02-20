use butt_head::{ButtHead, Config, TimeDuration, TimeInstant};

// --- Time types ---

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TestDuration(pub u64);

impl TimeDuration for TestDuration {
    const ZERO: Self = TestDuration(0);

    fn as_millis(&self) -> u64 {
        self.0
    }

    fn from_millis(millis: u64) -> Self {
        TestDuration(millis)
    }

    fn saturating_sub(self, other: Self) -> Self {
        TestDuration(self.0.saturating_sub(other.0))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TestInstant(pub u64);

impl TestInstant {
    pub fn ms(millis: u64) -> Self {
        TestInstant(millis)
    }
}

impl TimeInstant for TestInstant {
    type Duration = TestDuration;

    fn duration_since(&self, earlier: Self) -> TestDuration {
        TestDuration(self.0 - earlier.0)
    }

    fn checked_add(self, duration: TestDuration) -> Option<Self> {
        self.0.checked_add(duration.0).map(TestInstant)
    }

    fn checked_sub(self, duration: TestDuration) -> Option<Self> {
        self.0.checked_sub(duration.0).map(TestInstant)
    }
}

// --- Configs ---

pub static CONFIG: Config<TestDuration> = Config {
    active_low: false,
    click_timeout: TestDuration(300),
    hold_delay: TestDuration(500),
    hold_interval: TestDuration(200),
    max_click_count: None,
};

// --- Helpers ---

pub fn new_button() -> ButtHead<TestInstant> {
    ButtHead::new(&CONFIG)
}
