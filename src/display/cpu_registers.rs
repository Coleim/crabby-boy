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
            Cell::from(Line::from(vec!["AF".gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_af()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["BC".gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_bc()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["PC".gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.pc).bold().white()])),
        ]),
        Row::new(vec![
            Cell::from(Line::from(vec!["DE".gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_de()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["HL".gray()])),
            Cell::from(Line::from(vec![
                format!("{:04X}", cpu.get_hl()).bold().white(),
            ])),
            Cell::from(Line::from(vec!["SP".gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.sp).bold().white()])),
        ]),
        Row::new(vec![
            Cell::from(Line::from(vec!["FLAGS".gray()])),
            Cell::from(Line::from(vec![])),
            Cell::from(Line::from(vec![
                "Z".bold().fg(if cpu.get_z() {
                    Color::Green
                } else {
                    Color::Gray
                }),
                " N".bold().fg(if cpu.get_n() {
                    Color::Green
                } else {
                    Color::Gray
                }),
            ])),
            Cell::from(Line::from(vec![
                "H".bold().fg(if cpu.get_h() {
                    Color::Green
                } else {
                    Color::Gray
                }),
                " C".bold().fg(if cpu.get_c() {
                    Color::Green
                } else {
                    Color::Gray
                }),
            ])),
        ]),
        Row::new(vec![
            Cell::from(Line::from(vec!["IME".bold().fg(if cpu.ime {
                Color::Green
            } else {
                Color::Gray
            })])),
            Cell::from(Line::from(vec!["HALT".bold().fg(if cpu.halt {
                Color::Green
            } else {
                Color::Gray
            })])),
            Cell::from(Line::from(vec![])),
            Cell::from(Line::from(vec!["STOP".bold().fg(if cpu.stopped {
                Color::Green
            } else {
                Color::Gray
            })])),
        ]),
        Row::new(vec![
            Cell::from(Line::from(vec!["OP".gray()])),
            Cell::from(Line::from(vec![
                format!("0x{:02X}", cpu.current_opcode).bold().white(),
            ])),
            Cell::from(Line::from(vec!["CB".gray()])),
            Cell::from(Line::from(vec![
                format!("0xCB{:02X}", cpu.current_cbopcode).bold().white(),
            ])),
        ]),
    ];

    let widths = [
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(4),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths).block(block);
    frame.render_widget(table, area);
}
