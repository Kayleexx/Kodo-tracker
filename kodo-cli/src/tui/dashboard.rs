use std::path::Path;
use std::io;

use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Terminal,
};
use ratatui::prelude::{Backend, Style, Color, Modifier}; 
use crate::tui::{input, widgets};
use crate::git;
use crate::tui::widgets::draw_dashboard;
use kodo_core::Activity;

pub fn run(activities: &mut Vec<Activity>, path: &Path) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, activities, path);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    res
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    activities: &mut Vec<Activity>,
    activities_path: &Path,
) -> io::Result<()> {
    #[derive(PartialEq)]
    enum InputStage {
        Normal,
        AddingName,
        AddingDuration { name: String },
        FilteringMin,
        FilteringMax { min: u32 },
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum SortMode {
        ByDate,
        ByDuration,
        ByName,
    }

    let mut input_stage = InputStage::Normal;
    let mut input_buffer = String::new();
    let mut selected = 0usize;

    let mut filter_min: Option<u32> = None;
    let mut filter_max: Option<u32> = None;
    let mut sort_mode = SortMode::ByDate;
    let mut show_stats = false;
    let mut show_github = false;
    let mut github_activities: Vec<Activity> = Vec::new();

    loop {
        // Determine current view
        let view: Vec<Activity> = if show_github {
            github_activities.clone()
        } else {
            let mut filtered: Vec<Activity> = activities
                .iter()
                .filter(|a| filter_min.map_or(true, |m| a.duration_minutes >= m)
                    && filter_max.map_or(true, |m| a.duration_minutes <= m))
                .cloned()
                .collect();

            match sort_mode {
                SortMode::ByDate => filtered.sort_by(|a, b| b.date.cmp(&a.date)),
                SortMode::ByDuration => filtered.sort_by(|a, b| b.duration_minutes.cmp(&a.duration_minutes)),
                SortMode::ByName => filtered.sort_by(|a, b| a.name.cmp(&b.name)),
            }

            filtered
        };

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // title
                    Constraint::Length(1), // subtitle
                    Constraint::Length(2), // commands
                    Constraint::Min(0),    // table/stats
                    Constraint::Length(2), // footer/input
                ])
                .split(size);

            // Title
            let title = Paragraph::new(if show_github { "Kodo - GitHub Commits" } else { "Kodo" })
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::ITALIC))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(title, chunks[0]);

            // Subtitle
            let subtitle = Paragraph::new(if show_github { "Recent commits synced from your repo" } else { "Track your dev activities easily" })
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(subtitle, chunks[1]);

            // Commands
            let cmds = Paragraph::new(if show_github {
                "q: quit | r: return to activities | v: toggle stats"
            } else {
                "q: quit | a: add | d: delete | f: filter | r: reset filters | s: sort | v: toggle stats | g: git sync"
            })
            .style(Style::default().fg(Color::Yellow))
            .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(cmds, chunks[2]);

            // Main dashboard
            draw_dashboard(f, chunks[3], &view, selected, show_stats);

            // Footer / input stage
            let footer_text = match input_stage {
                InputStage::Normal => {
                    let total: u32 = view.iter().map(|a| a.duration_minutes).sum();
                    if show_github {
                        format!("GitHub commits: {} | Total est. time: {} min", view.len(), total)
                    } else {
                        format!("Total shown: {} | Total time: {} min | filter(min={:?}, max={:?})", view.len(), total, filter_min, filter_max)
                    }
                }
                InputStage::AddingName => format!("Enter activity name: {}", input_buffer),
                InputStage::AddingDuration { ref name } => format!("Enter duration (minutes) for '{}': {}", name, input_buffer),
                InputStage::FilteringMin => format!("Enter min duration filter: {}", input_buffer),
                InputStage::FilteringMax { min } => format!("Enter max duration filter (min={}): {}", min, input_buffer),
            };
            f.render_widget(Paragraph::new(footer_text), chunks[4]);
        })?;

        // Input handling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match &mut input_stage {
                    InputStage::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('a') => { input_stage = InputStage::AddingName; input_buffer.clear(); },
                        KeyCode::Char('d') => {
                            if !activities.is_empty() && !show_github {
                                activities.remove(selected);
                                if selected >= activities.len() && selected > 0 { selected -= 1; }
                                Activity::save_all_to_file(activities, activities_path).ok();
                            }
                        }
                        KeyCode::Char('f') => { input_stage = InputStage::FilteringMin; input_buffer.clear(); },
                        KeyCode::Char('r') => {
                            if show_github { show_github = false; } else { filter_min = None; filter_max = None; }
                        }
                        KeyCode::Char('s') => {
                            sort_mode = match sort_mode {
                                SortMode::ByDate => SortMode::ByDuration,
                                SortMode::ByDuration => SortMode::ByName,
                                SortMode::ByName => SortMode::ByDate,
                            };
                        }
                        KeyCode::Char('g') => {
                            let repo_path = std::env::current_dir().unwrap();
                            github_activities = git::get_github_activities(&repo_path, 50).unwrap_or_default();
                            show_github = true;
                            selected = 0;
                        }
                        KeyCode::Up => { if selected > 0 { selected -= 1; } },
                        KeyCode::Down => { if selected + 1 < view.len() { selected += 1; } },
                        _ => {}
                    },
                    InputStage::AddingName => match key.code {
                        KeyCode::Enter => {
                            if !input_buffer.trim().is_empty() {
                                let name = input_buffer.trim().to_string();
                                input_buffer.clear();
                                input_stage = InputStage::AddingDuration { name };
                            }
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Char(c) => { input_buffer.push(c); },
                        _ => {}
                    },
                    InputStage::AddingDuration { name } => match key.code {
                        KeyCode::Enter => {
                            let duration: u32 = input_buffer.trim().parse().unwrap_or(0);
                            if duration > 0 {
                                let next_id = activities.iter().map(|a| a.id).max().unwrap_or(0) + 1;
                                activities.push(Activity {
                                    id: next_id,
                                    name: name.clone(),
                                    duration_minutes: duration,
                                    date: Local::now().format("%Y-%m-%d").to_string(),
                                });
                                Activity::save_all_to_file(activities, activities_path).ok();
                            }
                            input_buffer.clear();
                            input_stage = InputStage::Normal;
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Char(c) => { input_buffer.push(c); },
                        _ => {}
                    },
                    InputStage::FilteringMin => match key.code {
                        KeyCode::Enter => {
                            let min = input_buffer.trim().parse().unwrap_or(0);
                            filter_min = if min > 0 { Some(min) } else { None };
                            input_buffer.clear();
                            input_stage = InputStage::FilteringMax { min };
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Char(c) => { input_buffer.push(c); },
                        _ => {}
                    },
                    InputStage::FilteringMax { min: _ } => match key.code {
                        KeyCode::Enter => {
                            let max = input_buffer.trim().parse().unwrap_or(0);
                            filter_max = if max > 0 { Some(max) } else { None };
                            input_buffer.clear();
                            input_stage = InputStage::Normal;
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Char(c) => { input_buffer.push(c); },
                        _ => {}

                    },
                }
            }
        }
    }
}