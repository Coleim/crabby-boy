use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Cell, Padding, Paragraph, Row, Table},
};

use crate::cpu::cpu::CPU;
use ratatui::style::{Color, Stylize};

pub fn render(frame: &mut Frame, area: Rect, cpu: &CPU) {
    let block = Block::bordered()
        .title(Line::from("CPU"))
        .padding(Padding {
            left: 1,
            right: 0,
            top: 1,
            bottom: 0,
        })
        .style(Style::new().light_magenta());

    let rows = vec![
        Row::new(vec![
            Cell::from(Line::from(vec!["AF".bold().gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_af()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["BC".bold().gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_bc()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["PC".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.pc).bold().white()])),
        ]),
        Row::new(vec![
            Cell::from(Line::from(vec!["DE".bold().gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_de()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["HL".bold().gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_hl()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["SP".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.sp).bold().white()])),
        ]),
    ];

    let widths = [
        Constraint::Length(4),
        Constraint::Length(6),
        Constraint::Length(4),
        Constraint::Length(6),
        Constraint::Length(4),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths).block(block);
    frame.render_widget(table, area);
}
