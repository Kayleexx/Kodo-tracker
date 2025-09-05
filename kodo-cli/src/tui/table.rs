use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use kodo_core::Activity;

pub struct ActivityTable;

impl ActivityTable {
    pub fn draw(f: &mut Frame, area: Rect, activities: &[Activity], selected: usize) {
        let rows: Vec<Row> = activities
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let style = if i == selected {
                    Style::default().bg(Color::Blue)
                } else if i % 2 == 0 {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    Cell::from(a.id.to_string()),
                    Cell::from(a.name.clone()),
                    Cell::from(format!("{} min", a.duration_minutes)),
                    Cell::from(a.date.clone()),
                ])
                .height(1)
                .style(style)
            })
            .collect();

        let widths: &[Constraint] = &[
            Constraint::Length(5),
            Constraint::Percentage(40),
            Constraint::Length(10),
            Constraint::Length(12),
        ];
        let table = Table::new(rows, widths)
            .header(
                Row::new(vec!["ID", "Name", "Duration", "Date"])
                    .height(1)
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            )
            .block(Block::default().title(" Activities ").borders(Borders::ALL));

        f.render_widget(table, area);
    }
}