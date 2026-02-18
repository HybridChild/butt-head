use crate::{Config, Event, ServiceTiming, TimeInstant};

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

    pub fn update(
        &mut self,
        edge: Option<Edge>,
        now: I,
    ) -> (Option<Event<I::Duration>>, ServiceTiming<I::Duration>) {
        // TODO: implement transition table
        let _ = (edge, now);
        (None, ServiceTiming::Idle)
    }
}
