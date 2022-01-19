use std::fmt::Debug;

use anyhow::Result;

use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::App;

type DialogCallback = Box<dyn Fn(&mut App) -> Result<()>>;

/// DialogBoxState is associate type used for stateful render of Dialogbox
pub struct DialogState {
    is_yes: bool,
    pub yes_fn: DialogCallback,
    pub no_fn: DialogCallback,
}

impl DialogState {
    pub fn is_yes(&self) -> bool {
        self.is_yes
    }

    pub fn toggle(&mut self) {
        self.is_yes = !self.is_yes
    }

    pub fn new(yes_fn: DialogCallback, no_fn: DialogCallback) -> Self {
        Self {
            is_yes: false,
            yes_fn,
            no_fn,
        }
    }
}

impl Debug for DialogState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogState")
            .field("is_yes", &self.is_yes)
            .finish()
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            is_yes: false,
            yes_fn: Box::new(|_| Ok(())),
            no_fn: Box::new(|_| Ok(())),
        }
    }
}

/// Custom widget that opens a popup to get user input
pub struct DialogBox {
    msg: String,
}

impl DialogBox {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl StatefulWidget for DialogBox {
    type State = DialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .borders(Borders::all())
            .border_type(BorderType::Rounded);

        let render_area = block.inner(area);
        block.render(area, buf);

        if render_area.height < 1 || render_area.width < 1 {
            return;
        }

        let splitted_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(render_area);
        let msg_area = splitted_area[0];
        let input_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(splitted_area[1]);

        let positive_input_area = input_areas[1];
        let negative_input_area = input_areas[2];

        let msg = Paragraph::new(Text::from(self.msg)).alignment(Alignment::Center);
        msg.render(msg_area, buf);

        let positive_msg = Paragraph::new(vec![Spans::from(vec![
            Span::styled("Y", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw("es"),
        ])])
        .alignment(Alignment::Center);
        positive_msg.render(positive_input_area, buf);

        let negative_msg = Paragraph::new(vec![Spans::from(vec![
            Span::styled("N", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw("o"),
        ])])
        .alignment(Alignment::Center);
        negative_msg.render(negative_input_area, buf);

        let highlight_style = Style::default().fg(Color::Black);
        if state.is_yes {
            buf.set_style(positive_input_area, highlight_style.bg(Color::LightGreen));
        } else {
            buf.set_style(negative_input_area, highlight_style.bg(Color::LightRed));
        }
    }
}
