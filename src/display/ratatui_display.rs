use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::Block,
};

use crate::{
    crabby_boy::CrabbyBoy,
    display::{cartridge_header, cpu_registers, display::Display, game, vram_registers},
};

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
        let fps = emulator.fps.fps();
        let title = Line::from(format!("CRABBY BOY ({}) FPS", fps));
        let block = Block::bordered()
            .title(title.centered())
            .style(Style::new().red());

        let inner_area = block.inner(frame.area());
        let horizontal = Layout::horizontal([Constraint::Percentage(33); 3]).spacing(1);
        let vertical = Layout::vertical([Constraint::Percentage(33); 3]);
        let [left, middle, right] = inner_area.layout(&horizontal);

        let [l_top, l_vmiddle, l_bottom] = left.layout(&vertical);
        let [m_top, m_vmiddle, m_bottom] = middle.layout(&vertical);

        frame.render_widget(block, frame.area());
        cartridge_header::render(frame, l_top, &emulator.header);
        game::render(frame, l_vmiddle);
        cpu_registers::render(frame, l_bottom, &emulator.cpu);

        vram_registers::render(frame, m_top, &emulator.bus.vram);
    }
}

impl Display for RatatuiDisplay {
    fn draw(&mut self, emulator: &crate::crabby_boy::CrabbyBoy) {
        self.terminal
            .draw(|frame| Self::draw_ui(frame, emulator))
            .ok();
    }

    fn handle_events(&mut self) {
        if event::poll(Duration::from_millis(0)).ok().unwrap_or(false)
            && let Ok(Event::Key(key_event)) = event::read()
            && key_event.kind == KeyEventKind::Press
        {
            match key_event.code {
                KeyCode::Char('q') => self.running = false,
                _ => {}
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
