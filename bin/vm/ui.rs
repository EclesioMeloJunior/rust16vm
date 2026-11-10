// Rendering code for the split-screen TUI:
//   +------------------------+----------------------+
//   |                        |      Registers       |
//   |    Program Output      +----------------------+
//   |    (mapped terminal)   |       Memory         |
//   +------------------------+----------------------+
//   | status bar: run/pause/step, PC, controls        |
//   +-------------------------------------------------+

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use rust16vm::machine::Register;

use super::App;

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let [body, status_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

    let [output_area, side_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    let [regs_area, mem_area] =
        Layout::vertical([Constraint::Length(11), Constraint::Fill(1)]).areas(side_area);

    draw_output(frame, app, output_area);
    draw_registers(frame, app, regs_area);
    draw_memory(frame, app, mem_area);
    draw_status(frame, app, status_area);
}

fn draw_output(frame: &mut Frame, app: &App, area: Rect) {
    let (cursor_x, cursor_y, lines): (usize, usize, Vec<Line>) = {
        let grid = app.grid.lock().unwrap();
        let (cx, cy) = grid.cursor();
        let lines = (0..grid.height())
            .map(|y| {
                let text = grid.line(y);
                if y != cy {
                    return Line::from(text);
                }

                let chars: Vec<char> = text.chars().collect();
                if chars.is_empty() {
                    return Line::from(text);
                }

                let cx = cx.min(chars.len() - 1);
                let before: String = chars[..cx].iter().collect();
                let current: String = chars[cx].to_string();
                let after: String = chars.get(cx + 1..).map_or_default(|s| s.iter().collect());

                Line::from(vec![
                    Span::raw(before),
                    Span::styled(
                        current,
                        Style::default().fg(Color::Black).bg(Color::White),
                    ),
                    Span::raw(after),
                ])
            })
            .collect();

        (cx, cy, lines)
    };

    let title = format!(" Program Output  (cursor {cursor_x},{cursor_y}) ");
    let output = Paragraph::new(lines).block(Block::bordered().title(title));
    frame.render_widget(output, area);
}

fn draw_registers(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::with_capacity(8);
    for idx in 0..8 {
        let Ok(reg) = Register::try_from(idx) else {
            continue;
        };

        let value = app.machine.register(reg);
        let highlight = matches!(reg, Register::PC | Register::FLAGS);

        let name_style = if highlight {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{:<5}", reg.to_string()), name_style),
            Span::raw(format!("{value:#06x}")),
            Span::raw("  "),
            Span::styled(
                format!("0b{value:016b}"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::raw(format!("{value:>5}")),
        ]));
    }

    let block = Block::bordered().title(" Registers  (hex / binary / dec) ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_memory(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows = area.height.saturating_sub(2) as usize;
    let mut lines = Vec::with_capacity(rows);
    let mut addr = app.mem_offset;

    for _ in 0..rows {
        let mut spans = vec![Span::styled(
            format!("{addr:#06x}: "),
            Style::default().fg(Color::DarkGray),
        )];

        let mut ascii = String::new();
        for i in 0..8u16 {
            let byte = app
                .machine
                .read_from_memory(addr.wrapping_add(i), 1)
                .first()
                .copied()
                .unwrap_or(0);

            spans.push(Span::raw(format!("{byte:02x} ")));
            ascii.push(if (0x20..=0x7e).contains(&byte) {
                byte as char
            } else {
                '.'
            });
        }

        spans.push(Span::styled(ascii, Style::default().fg(Color::Green)));
        lines.push(Line::from(spans));

        addr = addr.wrapping_add(8);
    }

    let title = format!(" Memory  @ {:#06x}  [Up/Down to scroll] ", app.mem_offset);
    let block = Block::bordered().title(title);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let (label, color) = if app.halted {
        (" HALTED ", Color::Red)
    } else if app.running {
        (" RUNNING ", Color::Green)
    } else {
        (" PAUSED ", Color::Yellow)
    };

    let line = Line::from(vec![
        Span::styled(
            label,
            Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "  PC={:#06x}  steps={}  {}",
            app.machine.register(Register::PC),
            app.instr_count,
            app.status
        )),
        Span::styled(
            "   [Space] run/pause  [S] step  [Up/Down] mem  [R] reset  [Q] quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
