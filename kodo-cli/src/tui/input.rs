use ratatui::widgets::{Block, Borders};
use ratatui::style::{Color, Modifier, Style};

pub fn header_block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
}
