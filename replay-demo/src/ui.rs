use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    sys::{Coords, State},
    Frame,
};

pub type Map = Rect;
pub type Input = Rect;
pub type Logs = Rect;

pub fn split_screen(screen: Rect) -> (Map, Logs, Input) {
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
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical[0]);

    (horizontal[0], horizontal[1], vertical[1])
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
        .border_style(Style::default());

    let inner_map = block.inner(map);

    let line = ".".repeat(inner_map.width as usize);
    let dot_style = Style::default().fg(Color::DarkGray);
    let spans = vec![Span::styled(line, dot_style); inner_map.height as usize];

    let paragraph = Paragraph::new(Spans::from(spans))
        .wrap(Wrap { trim: true })
        .block(block);

    frame.render_widget(paragraph, map);

    let robot = state.game().robot();
    let enemy_base = state.game().enemy_base();
    let friend_base = state.game().friend_base();
    frame.render_widget(
        CharWidget {
            c: '@',
            style: Style::default().fg(Color::Yellow),
            coords: robot,
        },
        inner_map,
    );
    frame.render_widget(
        CharWidget {
            c: 'x',
            style: Style::default().fg(Color::Red),
            coords: enemy_base,
        },
        inner_map,
    );
    frame.render_widget(
        CharWidget {
            c: '$',
            style: Style::default().fg(Color::Green),
            coords: friend_base,
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
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{:>3}: {}", i, m)))];
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

fn draw_input(frame: &mut Frame<'_>, _: &State, input: Rect) {
    let block = Block::default()
        .borders(Borders::all())
        .border_style(Style::default());
    frame.render_widget(block, input);
}

pub fn draw_state(frame: &mut Frame<'_>, state: &mut State) {
    let (map, logs, input) = split_screen(state.screen());
    draw_map(frame, state, map);
    draw_logs(frame, state, logs);
    draw_input(frame, state, input);
}
