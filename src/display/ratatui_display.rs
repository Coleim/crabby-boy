use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, text::Line, widgets::Block};

use crate::{crabby_boy::CrabbyBoy, display::display::Display};

pub struct RatatuiDisplay {
    running: bool,
    terminal: DefaultTerminal,
}

impl RatatuiDisplay {
    pub fn new() -> Self {
        RatatuiDisplay {
            running: true,
            terminal: ratatui::init(),
        }
    }

    fn draw_ui(frame: &mut Frame, emulator: &CrabbyBoy) {
        let title = Line::from("CRABBY BOY");
        let block = Block::bordered().title(title.centered());
        frame.render_widget(block, frame.area());
    }
}
impl Display for RatatuiDisplay {
    fn draw(&mut self, emulator: &crate::crabby_boy::CrabbyBoy) {
        self.terminal
            .draw(|frame| Self::draw_ui(frame, emulator))
            .ok();
    }

    fn handle_events(&mut self) {
        if event::poll(Duration::from_millis(0)).ok().unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => self.running = false,
                        _ => {}
                    }
                }
            }
        }
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

impl Drop for RatatuiDisplay {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
