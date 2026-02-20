use crate::{Config, Event, ServiceTiming, TimeDuration, TimeInstant};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Edge {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
enum State<I: TimeInstant> {
    Idle,
    Pressed {
        pressed_at: I,
        next_hold_at: I,
        click_count: u8,
        hold_level: u8,
    },
    WaitForMultiClick {
        click_count: u8,
        released_at: I,
    },
}

pub(crate) struct StateMachine<I: TimeInstant> {
    state: State<I>,
    config: &'static Config<I::Duration>,
}

impl<I: TimeInstant> StateMachine<I> {
    pub fn new(config: &'static Config<I::Duration>) -> Self {
        Self {
            state: State::Idle,
            config,
        }
    }

    /// Returns the instant the button was pressed if currently in the `Pressed`
    /// state, or `None` otherwise.
    pub fn pressed_at(&self) -> Option<I> {
        match self.state {
            State::Pressed { pressed_at, .. } => Some(pressed_at),
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        edge: Option<Edge>,
        now: I,
    ) -> (Option<Event<I::Duration>>, ServiceTiming<I::Duration>) {
        match self.state {
            State::Idle => match edge {
                Some(Edge::Press) => {
                    let next_hold_at = now.checked_add(self.config.hold_delay).unwrap_or(now);
                    self.state = State::Pressed {
                        pressed_at: now,
                        next_hold_at,
                        click_count: 0,
                        hold_level: 0,
                    };
                    (
                        Some(Event::Press),
                        ServiceTiming::Delay(self.config.hold_delay),
                    )
                }
                _ => (None, ServiceTiming::Idle),
            },

            State::Pressed {
                pressed_at,
                next_hold_at,
                click_count,
                hold_level,
            } => match edge {
                Some(Edge::Release) => {
                    let duration = now.duration_since(pressed_at);
                    if hold_level > 0 {
                        // A hold was emitted — this is not a click.
                        self.state = State::Idle;
                        (Some(Event::Release { duration }), ServiceTiming::Idle)
                    } else {
                        let new_count = click_count.saturating_add(1);
                        let at_max = self
                            .config
                            .max_click_count
                            .is_some_and(|max| new_count >= max);
                        self.state = State::WaitForMultiClick {
                            click_count: new_count,
                            released_at: now,
                        };
                        // If we've hit the cap, call back immediately so the
                        // Click event is emitted without waiting for the timeout.
                        let timing = if at_max {
                            ServiceTiming::Immediate
                        } else {
                            ServiceTiming::Delay(self.config.click_timeout)
                        };
                        (Some(Event::Release { duration }), timing)
                    }
                }
                _ => {
                    // No edge — check if the hold deadline has been reached.
                    let elapsed = now.duration_since(pressed_at);
                    let hold_elapsed = next_hold_at.duration_since(pressed_at);

                    if elapsed.as_millis() >= hold_elapsed.as_millis() {
                        let event = Event::Hold {
                            clicks_before: click_count,
                            level: hold_level,
                        };
                        let new_next_hold_at = next_hold_at
                            .checked_add(self.config.hold_interval)
                            .unwrap_or(next_hold_at);
                        self.state = State::Pressed {
                            pressed_at,
                            next_hold_at: new_next_hold_at,
                            click_count,
                            hold_level: hold_level.saturating_add(1),
                        };
                        (Some(event), ServiceTiming::Delay(self.config.hold_interval))
                    } else {
                        let remaining = hold_elapsed.saturating_sub(elapsed);
                        (None, ServiceTiming::Delay(remaining))
                    }
                }
            },

            State::WaitForMultiClick {
                click_count,
                released_at,
            } => match edge {
                Some(Edge::Press) => {
                    let next_hold_at = now.checked_add(self.config.hold_delay).unwrap_or(now);
                    self.state = State::Pressed {
                        pressed_at: now,
                        next_hold_at,
                        click_count,
                        hold_level: 0,
                    };
                    (
                        Some(Event::Press),
                        ServiceTiming::Delay(self.config.hold_delay),
                    )
                }
                _ => {
                    // Fire immediately if we've hit max_click_count.
                    let at_max = self
                        .config
                        .max_click_count
                        .is_some_and(|max| click_count >= max);
                    if at_max {
                        self.state = State::Idle;
                        return (
                            Some(Event::Click { count: click_count }),
                            ServiceTiming::Idle,
                        );
                    }

                    let elapsed = now.duration_since(released_at);
                    if elapsed.as_millis() >= self.config.click_timeout.as_millis() {
                        self.state = State::Idle;
                        (
                            Some(Event::Click { count: click_count }),
                            ServiceTiming::Idle,
                        )
                    } else {
                        let remaining = self.config.click_timeout.saturating_sub(elapsed);
                        (None, ServiceTiming::Delay(remaining))
                    }
                }
            },
        }
    }
}
