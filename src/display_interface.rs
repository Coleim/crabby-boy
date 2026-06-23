use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use image::{DynamicImage, RgbaImage};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Padding, StatefulWidget, Widget},
};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};
use std::io;
use std::{cell::RefCell, time::Duration};

//TODO inclure ref vers le CPU / PPU / APU / ROM ETC
pub struct DisplayInterface {
    pub running: bool,
    // Le protocole d'image doit être mutable lors du render, mais Widget
    // ne nous donne que &self -> on passe par une RefCell.
    image_state: RefCell<StatefulProtocol>,
}

impl DisplayInterface {
    pub fn new() -> Self {
        // Picker interroge le terminal (taille de police, support Sixel/Kitty/etc).
        // Doit être appelé une seule fois.
        let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
        let img = generate_buffer();
        let protocol = picker.new_resize_protocol(img);

        DisplayInterface {
            running: true,
            image_state: RefCell::new(protocol),
        }
    }

    pub fn update(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        // self.handle_events()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.running = false,
                        _ => {}
                    }
                    // self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }
}

impl Widget for &DisplayInterface {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("CRABBY BOY".bold());
        let full_block = Block::bordered()
            .title(title.centered())
            .padding(Padding::top(10));
        full_block.render(area, buf);

        let horizontal = Layout::horizontal([Constraint::Percentage(20); 3]).spacing(1);
        let [left, middle, right] = area.layout(&horizontal);

        let vertical = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).spacing(1);
        let [top, main] = left.layout(&vertical);

        let block = Block::bordered().title("Bordered block");
        block.render(top, buf);

        // --- Affichage du buffer de pixels dans la zone "main" ---
        let image_widget = StatefulImage::default();
        let mut protocol = self.image_state.borrow_mut();
        image_widget.render(main, buf, &mut *protocol);
    }
}

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

/// Build a raw RGBA pixel buffer by hand and wrap it in a DynamicImage.
fn generate_buffer() -> DynamicImage {
    let mut raw_pixels: Vec<u8> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let r = ((x as f32 / WIDTH as f32) * 255.0) as u8;
            let g = ((y as f32 / HEIGHT as f32) * 255.0) as u8;
            let b = 128u8;
            let a = 255u8;
            raw_pixels.push(r);
            raw_pixels.push(g);
            raw_pixels.push(b);
            raw_pixels.push(a);
        }
    }
    let img_buf = RgbaImage::from_raw(WIDTH, HEIGHT, raw_pixels)
        .expect("la taille du buffer doit être width*height*4");
    DynamicImage::ImageRgba8(img_buf)
}

// use std::io;
//
// use crate::{bus::bus::Bus, display_interface};
//
// use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
// use ratatui::{
//     DefaultTerminal, Frame,
//     buffer::Buffer,
//     layout::{Constraint, Layout, Rect},
//     style::Stylize,
//     symbols::border,
//     text::{Line, Text},
//     widgets::{Block, Padding, Paragraph, Widget},
// };
//
// //TODO inclure ref vers le CPU / PPU / APU / ROM ETC
// #[derive(Debug, Default)]
// pub struct DisplayInterface {
//     pub running: bool,
//     pub img: DynamicImage,
// }
//
// impl DisplayInterface {
//     pub fn new() -> Self {
//         let img = generate_buffer();
//         DisplayInterface { running: true, img }
//     }
//
//     pub fn update(&mut self, terminal: &mut DefaultTerminal) {
//         terminal.draw(|frame| self.draw(frame));
//         self.handle_events();
//     }
//
//     fn draw(&self, frame: &mut Frame) {
//         frame.render_widget(self, frame.area());
//     }
//
//     fn handle_events(&mut self) -> io::Result<()> {
//         match event::read()? {
//             // it's important to check that the event is a key press event as
//             // crossterm also emits key release and repeat events on Windows.
//             Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
//                 match key_event.code {
//                     KeyCode::Char('q') => self.running = false,
//                     _ => {}
//                 }
//                 // self.handle_key_event(key_event)
//             }
//             _ => {}
//         };
//         Ok(())
//     }
// }
// //
// // impl Widget for &DisplayInterface {
// //     fn render(self, area: Rect, buf: &mut Buffer) {
// //         let title = Line::from("CRABBY BOY".bold());
// //
// //         let full_block = Block::bordered()
// //             .title(title.centered())
// //             .padding(Padding::top(10));
// //         // .border_set(border::THICK);
// //         full_block.render(area, buf);
// //         let horizontal = Layout::horizontal([Constraint::Percentage(20); 3]).spacing(1);
// //         let [left, middle, right] = area.layout(&horizontal);
// //
// //         let vertical = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).spacing(1);
// //         let [top, main] = left.layout(&vertical);
// //
// //         let block = Block::bordered().title("Bordered block");
// //         block.render(top, buf);
// //
// //     }
// // }
// //
//
// impl Widget for &DisplayInterface {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Line::from("CRABBY BOY".bold());
//         let full_block = Block::bordered()
//             .title(title.centered())
//             .padding(Padding::top(10));
//         full_block.render(area, buf);
//
//         let horizontal = Layout::horizontal([Constraint::Percentage(20); 3]).spacing(1);
//         let [left, middle, right] = area.layout(&horizontal);
//
//         let vertical = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).spacing(1);
//         let [top, main] = left.layout(&vertical);
//
//         let block = Block::bordered().title("Bordered block");
//         block.render(top, buf);
//
//         // --- Affichage du buffer de pixels dans la zone "main" ---
//         let image_widget = StatefulImage::default();
//         let mut protocol = self.image_state.borrow_mut();
//         image_widget.render(main, buf, &mut protocol);
//     }
// }
//
// const WIDTH: u32 = 160;
// const HEIGHT: u32 = 144;
// use image::{DynamicImage, RgbaImage};
// /// Build a raw RGBA pixel buffer by hand and wrap it in a DynamicImage.
// fn generate_buffer() -> DynamicImage {
//     let mut raw_pixels: Vec<u8> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
//
//     for y in 0..HEIGHT {
//         for x in 0..WIDTH {
//             let r = ((x as f32 / WIDTH as f32) * 255.0) as u8;
//             let g = ((y as f32 / HEIGHT as f32) * 255.0) as u8;
//             let b = 128u8;
//             let a = 255u8;
//
//             raw_pixels.push(r);
//             raw_pixels.push(g);
//             raw_pixels.push(b);
//             raw_pixels.push(a);
//         }
//     }
//
//     let img_buf = RgbaImage::from_raw(WIDTH, HEIGHT, raw_pixels)
//         .expect("la taille du buffer doit être width*height*4");
//     DynamicImage::ImageRgba8(img_buf)
// }
