use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;
use anyhow::Result;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> Result<Option<AppEvent>> {
        if event::poll(self.tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return Ok(Some(AppEvent::Key(key)));
                }
            }
        } else {
            return Ok(Some(AppEvent::Tick));
        }
        Ok(None)
    }
}
