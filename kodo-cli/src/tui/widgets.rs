use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, BarChart},
    Frame,
};
use kodo_core::Activity;
use crate::tui::table::ActivityTable;

pub fn draw_dashboard(
    f: &mut Frame,
    area: Rect,
    activities: &[Activity],
    selected: usize,
    show_stats: bool,
) {
    if show_stats {
        // Split area: top for table, bottom for stats
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Table
        ActivityTable::draw(f, chunks[0], activities, selected);

        // Stats: simple bar chart (text-based)
        draw_stats(f, chunks[1], activities);
    } else {
        // Table only
        ActivityTable::draw(f, area, activities, selected);
    }
}

// Simple stats drawer
fn draw_stats(f: &mut Frame, area: Rect, activities: &[Activity]) {
    if activities.is_empty() {
        return;
    }

    let mut data: Vec<(&str, u64)> = activities
        .iter()
        .map(|a| (a.name.as_str(), a.duration_minutes as u64))
        .collect();

    // sort descending by duration
    data.sort_by(|a, b| b.1.cmp(&a.1));

    let barchart = BarChart::default()
        .block(Block::default().title("Activity Duration Stats").borders(Borders::ALL))
        .data(&data)
        .bar_width(7)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::Yellow))
        .max(activities.iter().map(|a| a.duration_minutes).max().unwrap_or(1) as u64);

    f.render_widget(barchart, area);
}
