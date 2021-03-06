use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    game::Coords,
    sys::{Focus, State},
    Frame,
};

pub type Map = Rect;

pub fn inner(r: Rect) -> Rect {
    Block::default().borders(Borders::all()).inner(r)
}

pub fn map_size(screen: Rect) -> Map {
    let input_height = 3;
    let map_height = screen.height.saturating_sub(input_height);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(map_height),
            Constraint::Length(input_height),
        ])
        .split(screen);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vertical[0]);

    horizontal[0]
}

struct CharWidget {
    c: char,
    style: Style,
    coords: Coords,
}

impl tui::widgets::Widget for CharWidget {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let mut tmp = [0; 4];
        let str = char::encode_utf8(self.c, &mut tmp);
        let x = area.left() + self.coords.x;
        let y = area.top() + self.coords.y;
        buf.set_string(x, y, str, self.style);
    }
}

fn draw_map(frame: &mut Frame<'_>, state: &State, map: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default())
        .title("Map")
        .title_alignment(Alignment::Center);

    let inner_map = block.inner(map);

    let line = ".".repeat(inner_map.width as usize);
    let dot_style = Style::default().fg(Color::DarkGray);
    let spans = vec![Span::styled(line, dot_style); inner_map.height as usize];

    let paragraph = Paragraph::new(Spans::from(spans))
        .wrap(Wrap { trim: true })
        .block(block);

    frame.render_widget(paragraph, map);

    let game = state.game();
    let robot = game.robot();
    let base = game.base();

    let robot_color = if let Focus::Input = state.focus() {
        Color::Green
    } else {
        Color::Red
    };

    frame.render_widget(
        CharWidget {
            c: '$',
            style: Style::default().fg(Color::Green),
            coords: base,
        },
        inner_map,
    );

    frame.render_widget(
        CharWidget {
            c: '@',
            style: Style::default()
                .fg(robot_color)
                .add_modifier(Modifier::BOLD),
            coords: robot,
        },
        inner_map,
    );
}

fn draw_logs(frame: &mut Frame<'_>, state: &mut State, logs_plane: Rect) {
    let block = Block::default()
        .borders(Borders::all())
        .border_style(Style::default())
        .title("Logs");

    let logs: Vec<ListItem> = state
        .logs()
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let color = if log.contains("CONNECTED") | log.contains("SENT") {
                Color::Green
            } else if log.contains("ERROR") {
                Color::Red
            } else {
                Color::White
            };
            let content = Span::styled(format!("{:>3}: {}", i, log), Style::default().fg(color));
            ListItem::new(content)
        })
        .collect();

    let logs = List::new(logs).block(block);
    let mut list_state = ListState::default();

    let logs = if let Some(selected) = state.log_selected() {
        list_state.select(Some(selected));
        logs.highlight_symbol("> ")
    } else {
        list_state.select(state.logs().len().checked_sub(1));
        logs
    };

    frame.render_stateful_widget(logs, logs_plane, &mut list_state);
}

fn draw_input(frame: &mut Frame<'_>, state: &State, input: Rect) {
    let block = Block::default()
        .borders(Borders::all())
        .border_style(Style::default());

    if let Focus::Input = state.focus() {
        const PROMPT: &str = "> ";
        let str = format!("{PROMPT}{}", state.input().line());
        let paragraph = Paragraph::new(str)
            .style(Style::default().fg(Color::Green))
            .block(block.title("[CONNECTED]"));

        let y = input.y + 1; // border
        let x = input.x + 1  // border
            + PROMPT.len() as u16
            + state.input().cursor() as u16;

        frame.render_widget(paragraph, input);
        frame.set_cursor(x, y);
    } else {
        frame.render_widget(block, input)
    }
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let (grid_width, grid_height) = (r.width, r.height);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(grid_height / 2 - height / 2),
            Constraint::Length(height),
            Constraint::Length(grid_height / 2 - height / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(grid_width / 2 - width / 2),
            Constraint::Length(width),
            Constraint::Length(grid_width / 2 - width / 2),
        ])
        .split(vertical[1])[1]
}

fn draw_screen_too_small(frame: &mut Frame<'_>, screen: Rect) {
    let block = Block::default()
        .borders(Borders::all())
        .style(Style::default().bg(Color::Blue));
    frame.render_widget(block, screen);

    let center = centered_rect(screen.width, 1, screen);
    let paragraph = Paragraph::new("Sorry, your screen is too small").alignment(Alignment::Center);
    frame.render_widget(paragraph, center);
}

fn draw_win_screen(frame: &mut Frame<'_>, screen: Rect) {
    let spans = vec![
        Spans::from("Congratulations!\n"),
        Spans::from("You've solved the game using the replay attacks.\n"),
        Spans::from("Despite the fact that integrity of the ciphertext was provided,\n"),
        Spans::from("it wasn't enough to protect against such attacks.\n"),
        Spans::from("\n"),
        Spans::from("Press [q] to quit.\n"),
    ];

    let block = Block::default()
        .borders(Borders::all())
        .style(Style::default().bg(Color::Blue));
    frame.render_widget(block, screen);

    let center = centered_rect(screen.width, spans.len() as u16, screen);
    let paragraph = Paragraph::new(spans)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, center);
}

pub fn draw_state(frame: &mut Frame<'_>, state: &mut State) {
    if state.game().is_finished() {
        draw_win_screen(frame, frame.size());
        return;
    }
    let game_map = state.game().map();
    let input_height = 3;
    let map_height = game_map.height;
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(map_height),
            Constraint::Length(input_height),
        ])
        .split(frame.size());

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(game_map.width),
            Constraint::Percentage(100),
        ])
        .split(vertical[0]);

    let map = horizontal[0];

    if map.width < game_map.width || map.height < game_map.height {
        draw_screen_too_small(frame, frame.size())
    } else {
        draw_map(frame, state, horizontal[0]);
        draw_logs(frame, state, horizontal[1]);
        draw_input(frame, state, vertical[1]);
    }
}
