mod dialog_box;

pub use dialog_box::{DialogBox, DialogBoxType, DialogCallback, DialogState};

use std::net::TcpStream;

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::MsgType;

pub fn message_box(messages: &Vec<(MsgType, String)>) -> List {
    let message_listitem: Vec<ListItem> = messages
        .iter()
        .map(|(a, m)| -> ListItem {
            ListItem::new(match a {
                MsgType::Sent => vec![Spans::from(Span::raw(format!("You: {}", m)))],
                MsgType::Recv => vec![Spans::from(Span::raw(format!("Other: {}", m)))],
            })
        })
        .collect();

    List::new(message_listitem).block(Block::default().borders(Borders::ALL).title("Messages"))
}

pub fn input_box(input: &str) -> Paragraph {
    Paragraph::new(input)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter a command or message"),
        )
}

pub fn connection_status_message(client: &Option<TcpStream>) -> Paragraph {
    let span = if let Some(c) = client {
        let ip = c.peer_addr().unwrap().ip();
        Spans::from(vec![Span::styled(
            format!("Connected to {}", ip),
            Style::default().fg(Color::Green),
        )])
    } else {
        let red_style = Style::default().fg(Color::Red);
        Spans::from(vec![
            Span::styled("Not connected to a client. Use ?connect ", red_style),
            Span::styled("ip", red_style.add_modifier(Modifier::ITALIC)),
            Span::styled(" to connect", red_style),
        ])
    };
    Paragraph::new(span)
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
