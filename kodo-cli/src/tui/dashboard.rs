use std::io;
use std::path::Path;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    Terminal,
};
use crate::tui::table::ActivityTable;
use crate::tui::input;
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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    activities: &mut Vec<Activity>,
    path: &Path,
) -> io::Result<()> {
    #[derive(PartialEq)]
    enum InputStage {
        Normal,
        AddingName,
        AddingDuration { name: String },
        EditingName { activity_id: u32 },
        EditingDuration { activity_id: u32 },
    }

    let mut input_stage = InputStage::Normal;
    let mut input_buffer = String::new();
    let mut selected = 0usize;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
                .split(size);

            // Header
            let header_text = " Kodo Dashboard - 'q' quit | 'a' add | 'd' delete | 'e' edit ";
            f.render_widget(input::header_block(header_text), chunks[0]);

            // Table
            ActivityTable::draw(f, chunks[1], activities, selected);

            // Bottom input / stats
            let bottom_text = match &input_stage {
                InputStage::Normal => format!(
                    "Total activities: {} | Total time: {} min",
                    activities.len(),
                    activities.iter().map(|a| a.duration_minutes).sum::<u32>()
                ),
                InputStage::AddingName => format!("Enter activity name: {}", input_buffer),
                InputStage::AddingDuration { name } => {
                    format!("Enter duration (minutes) for '{}': {}", name, input_buffer)
                }
                InputStage::EditingName { activity_id } => {
                    let act = activities.iter().find(|a| a.id == *activity_id);
                    match act {
                        Some(a) => format!("Editing name for '{}', leave empty to keep: {}", a.name, input_buffer),
                        None => "Activity not found.".to_string(),
                    }
                }
                InputStage::EditingDuration { activity_id } => {
                    let act = activities.iter().find(|a| a.id == *activity_id);
                    match act {
                        Some(a) => format!("Editing duration for '{}', leave empty to keep: {}", a.name, input_buffer),
                        None => "Activity not found.".to_string(),
                    }
                }
            };
            f.render_widget(Paragraph::new(bottom_text), chunks[2]);
        })?;

        // Input handling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match &mut input_stage {
                    InputStage::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('a') => {
                            input_stage = InputStage::AddingName;
                            input_buffer.clear();
                        }
                        KeyCode::Char('d') => {
                            if !activities.is_empty() {
                                activities.remove(selected);
                                if selected >= activities.len() && selected > 0 {
                                    selected -= 1;
                                }
                                Activity::save_all_to_file(activities, path).ok();
                            }
                        }
                        KeyCode::Char('e') => {
                            if let Some(act) = activities.get(selected) {
                                input_stage = InputStage::EditingName { activity_id: act.id };
                                input_buffer.clear();
                            }
                        }
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected + 1 < activities.len() {
                                selected += 1;
                            }
                        }
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
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; }
                        KeyCode::Backspace => { input_buffer.pop(); }
                        KeyCode::Char(c) => { input_buffer.push(c); }
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
                                Activity::save_all_to_file(activities, path).ok();
                                input_stage = InputStage::Normal;
                            }
                            input_buffer.clear();
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; }
                        KeyCode::Backspace => { input_buffer.pop(); }
                        KeyCode::Char(c) => { input_buffer.push(c); }
                        _ => {}
                    },
                    InputStage::EditingName { activity_id } => match key.code {
                        KeyCode::Enter => {
                            let new_name = if input_buffer.trim().is_empty() {
                                None
                            } else {
                                Some(input_buffer.trim().to_string())
                            };
                            if let Some(act) = activities.iter_mut().find(|a| a.id == *activity_id) {
                                if let Some(name) = new_name {
                                    act.name = name;
                                }
                            }
                            input_buffer.clear();
                            input_stage = InputStage::EditingDuration { activity_id: *activity_id };
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; }
                        KeyCode::Backspace => { input_buffer.pop(); }
                        KeyCode::Char(c) => { input_buffer.push(c); }
                        _ => {}
                    },
                    InputStage::EditingDuration { activity_id } => match key.code {
                        KeyCode::Enter => {
                            let new_duration = if input_buffer.trim().is_empty() {
                                None
                            } else {
                                input_buffer.trim().parse::<u32>().ok()
                            };
                            if let Some(act) = activities.iter_mut().find(|a| a.id == *activity_id) {
                                if let Some(duration) = new_duration {
                                    act.duration_minutes = duration;
                                }
                            }
                            Activity::save_all_to_file(activities, path).ok();
                            input_buffer.clear();
                            input_stage = InputStage::Normal;
                        }
                        KeyCode::Esc => { input_buffer.clear(); input_stage = InputStage::Normal; }
                        KeyCode::Backspace => { input_buffer.pop(); }
                        KeyCode::Char(c) => { input_buffer.push(c); }
                        _ => {}
                    },
                }
            }
        }
    }
}
