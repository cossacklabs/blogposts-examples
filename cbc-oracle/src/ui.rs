use crate::machine::{DecryptingMachine, BLOCK_SIZE};

use std::time::Duration;
use tui::layout::Rect;
use tui::text::Spans;
use tui::widgets::{Clear, List, ListItem};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::machine::State;

const STYLE_KNOWN: fn() -> Style = || Style::default().fg(Color::Green);
const STYLE_IV: fn() -> Style = || Style::default().fg(Color::LightCyan);
const STYLE_PADDING: fn() -> Style = || Style::default().fg(Color::White);
const STYLE_CIPHERTEXT: fn() -> Style = || Style::default().fg(Color::Yellow);
const STYLE_COUNTER: fn() -> Style = || Style::default().fg(Color::Cyan);

pub fn ui<B: Backend>(
    f: &mut Frame<B>,
    machine: &DecryptingMachine,
    timeout: Duration,
    advance: bool,
) {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(size.height - 1), Constraint::Length(1)])
        .split(size);

    let (top, footer) = (layout[0], layout[1]);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Percentage(20),
        ])
        .split(top);
    let top_panel = vertical[0];
    let oracle_panel = vertical[1];
    let computations_panel = vertical[2];
    let decrypted_panel = vertical[3];

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(top_panel);

    let left = horizontal[0];
    let right = horizontal[1];

    render_ciphertext_panel(f, machine, left);
    render_variable_table(f, machine, right);
    render_decrypted_panel(f, machine, decrypted_panel);
    render_oracle_panel(f, machine, oracle_panel);

    render_computations(f, machine, computations_panel);
    render_footer(f, machine, timeout, advance, footer);

    if let State::Start = machine.state {
        render_press_any_key(f, f.size())
    }
}

fn render_ciphertext_panel<B: Backend>(f: &mut Frame<B>, machine: &DecryptingMachine, panel: Rect) {
    let cipherblocks = [&machine.oracle.iv()[..], &machine.ciphertext].concat();

    // Blocks are numerated from 0, but since we also add iv, all cipherblock
    // indexes should start with a one
    let iv_num = machine.block_num;
    let block_num = machine.block_num.map(|i| i + 1);

    let cipherblocks = cipherblocks
        .chunks(BLOCK_SIZE)
        .enumerate()
        .map(|(i, block)| (i, format!("{}. {}  ", i, hex::encode(block))))
        .map(|(i, block)| match i {
            i if Some(i) == iv_num => ListItem::new(block).style(STYLE_IV()),
            i if Some(i) == block_num => ListItem::new(block).style(STYLE_CIPHERTEXT()),
            _ => ListItem::new(block),
        })
        .collect::<Vec<_>>();
    let cipher_blocks = List::new(cipherblocks)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Cipherblocks")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(STYLE_CIPHERTEXT());

    f.render_widget(cipher_blocks, panel);
}

fn render_decrypted_panel<B: Backend>(f: &mut Frame<B>, machine: &DecryptingMachine, panel: Rect) {
    let decrypted = String::from_utf8_lossy(&machine.decrypted).to_string();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Decrypted")
        .title_alignment(Alignment::Center);

    let inner = block.inner(panel);
    f.render_widget(block, panel);

    let decrypted_blocks = Paragraph::new(decrypted).style(Style::default().fg(Color::Green));

    f.render_widget(decrypted_blocks, inner);
}

fn render_variable_table<B: Backend>(f: &mut Frame<B>, machine: &DecryptingMachine, panel: Rect) {
    let padding = [machine.padding].repeat(machine.padding as usize);

    let block = format_block(&machine.block, STYLE_CIPHERTEXT());
    let iv = format_block(&machine.iv, STYLE_IV());
    let known = format_block(&machine.known, STYLE_KNOWN());
    let padding = format_block(&padding, STYLE_PADDING());
    let counter = format_block(&machine.counter, STYLE_COUNTER());

    let rows = [
        Row::new([Cell::from("cipherblock"), Cell::from(block)]),
        Row::new([Cell::from("iv or previous block"), Cell::from(iv)]),
        Row::new([Cell::from("known"), Cell::from(known)]),
        Row::new([Cell::from("padding"), Cell::from(padding)]),
        Row::new([Cell::from("counter"), Cell::from(counter)]),
    ];

    let table = Table::new(rows)
        .widths(&[Constraint::Percentage(35), Constraint::Percentage(65)])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Left)
                .title(format!("{:?}", machine.state)),
        );

    f.render_widget(table, panel);
}

fn render_oracle_panel<B: Backend>(f: &mut Frame<B>, machine: &DecryptingMachine, panel: Rect) {
    let block = Block::default()
        .style(Style::default())
        .borders(Borders::ALL)
        .title("Oracle")
        .title_alignment(Alignment::Center);
    f.render_widget(block.clone(), panel);

    let ok = if let Some(ok) = machine.ok {
        ok
    } else {
        return;
    };

    let ok = if ok {
        Span::styled("✓", Style::default().fg(Color::Green))
    } else {
        Span::styled("X", Style::default().fg(Color::Red))
    };
    let query = Spans::from(vec![
        Span::raw("query("),
        Span::styled(hex::encode(&machine.counter), STYLE_COUNTER()),
        Span::styled(hex::encode(&machine.block), STYLE_CIPHERTEXT()),
        Span::raw(") = "),
        ok,
    ]);

    let paragrah = Paragraph::new(query)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(paragrah, panel);
}

fn render_computations<B: Backend>(f: &mut Frame<B>, machine: &DecryptingMachine, panel: Rect) {
    let block = Block::default().borders(Borders::ALL);
    if machine.state != State::CalculatingPlainByte {
        f.render_widget(block, panel);
        return;
    }

    let block = block
        .title("Plaintext byte")
        .title_alignment(Alignment::Center);

    let idx = BLOCK_SIZE - machine.padding as usize;
    let iv = machine.iv[idx];
    let counter = machine.counter[idx];
    let padding = machine.padding;
    let plain = iv ^ counter ^ padding;

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::styled(format!("{plain:02x}(plain)"), STYLE_KNOWN()),
        Span::raw(" = "),
        Span::styled(format!("{iv:02x}(iv)"), STYLE_IV()),
        Span::raw(" xor "),
        Span::styled(format!("{counter:02x}(counter)"), STYLE_COUNTER()),
        Span::raw(" xor "),
        Span::styled(format!("{padding:02x}(padding)"), STYLE_PADDING()),
    ]))
    .alignment(Alignment::Center)
    .block(block);

    f.render_widget(paragraph, panel);
}

fn render_footer<B: Backend>(
    f: &mut Frame<B>,
    machine: &DecryptingMachine,
    timeout: Duration,
    advance: bool,
    panel: Rect,
) {
    let cycles = machine.oracle.counter();
    let processing = if advance { "[running]" } else { "[paused]" };
    let paragraph = Paragraph::new(format!("{processing} {timeout:?} {cycles} queries "))
        .style(Style::default().bg(Color::DarkGray))
        .alignment(Alignment::Right);

    f.render_widget(paragraph, panel);
}

fn format_block(data: &[u8], style: Style) -> Spans {
    let padding = BLOCK_SIZE.saturating_sub(data.len());
    let mut spans = vec![];

    let mut delim = "";

    for _ in 0..padding {
        spans.push(Span::styled(
            format!("{delim}xx"),
            Style::default().fg(Color::DarkGray),
        ));

        delim = " ";
    }

    for b in data {
        spans.push(Span::styled(format!("{delim}{b:<02x}"), style));
        delim = " ";
    }

    Spans::from(spans)
}

fn render_press_any_key<B: Backend>(f: &mut Frame<B>, screen: Rect) {
    const PRESS_ANY_KEY: &str = "Press c to continue";

    // Border + text + border
    let height = 3;
    // Border + whitespace + text + whitespace + border
    let width = PRESS_ANY_KEY.len() as u16 + 4;

    let area = centered_rect(width, height, screen);

    let paragrah = Paragraph::new(PRESS_ANY_KEY)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Blue)),
        );

    f.render_widget(Clear, area);
    f.render_widget(paragrah, area);
}

fn centered_rect(size_x: u16, size_y: u16, r: Rect) -> Rect {
    let height = r.height.saturating_sub(size_y) / 2;
    let width = r.width.saturating_sub(size_x) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(height),
            Constraint::Length(size_y),
            Constraint::Length(height),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(width),
            Constraint::Length(size_x),
            Constraint::Length(width),
        ])
        .split(popup_layout[1])[1]
}
