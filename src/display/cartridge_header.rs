use crate::{cpu::header::CartdrigeHeader, crabby_boy::CrabbyBoy};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Padding;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, header: &CartdrigeHeader) {
    let inner_block = Block::bordered()
        .title(Line::from("Cartridge").centered())
        .padding(Padding {
            left: 1,
            right: 0,
            top: 1,
            bottom: 0,
        })
        .style(Style::new().light_blue());

    // frame.render_widget(inner_block, area);

    let lines = vec![
        Line::from_iter([
            "Title: ".bold().into(),
            header.title.clone().italic().yellow(),
        ]),
        Line::from_iter([
            "Cartridge Type: ".bold().into(),
            header.cartridge_type.clone().italic(),
        ]),
        Line::from_iter(["Color: ".bold().into(), header.cgb_flag.clone().italic()]),
        Line::from_iter(["SGB: ".bold().into(), header.sgb_flag.clone().italic()]),
        Line::from_iter([
            "Manufacturer: ".bold().into(),
            header.manufacturer_code.clone().italic(),
        ]),
        Line::from_iter(["Licensee: ".bold().into(), header.licensee.clone().italic()]),
        Line::from_iter([
            "Destination: ".bold().into(),
            header.destination_code.clone().italic(),
        ]),
        Line::from_iter(["ROM Size: ".bold().into(), header.rom_size.clone().italic()]),
        Line::from_iter(["RAM Size: ".bold().into(), header.ram_size.clone().italic()]),
    ];

    let title = Paragraph::new(lines).block(inner_block);

    frame.render_widget(title, area);
}
