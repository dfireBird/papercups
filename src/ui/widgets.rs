use anyhow::Result;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::MsgType;

pub fn message_box(messages: &Vec<(MsgType, String)>) -> List {
    let message_listitem: Vec<ListItem> = messages
        .iter()
        .map(|(a, m)| -> ListItem {
            ListItem::new(match a {
                MsgType::Recv => vec![Spans::from(Span::raw(format!("You: {}", m)))],
                MsgType::Sent => vec![Spans::from(Span::raw(format!("Other: {}", m)))],
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
