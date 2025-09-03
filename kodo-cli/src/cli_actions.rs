use anyhow::{Context, Result};
use std::path::Path;

use kodo_core::Activity;

pub fn add_activity(
    activities: &mut Vec<Activity>,
    name: &str,
    minutes: u32,
    path: &Path,
) -> Result<()> {
    let next_id = activities.iter().map(|a| a.id()).max().unwrap_or(0) + 1;
    let act = Activity::new_with_id(next_id, name, minutes);
    activities.push(act);
    Activity::save_all_to_file(activities, path)
        .with_context(|| format!("Failed to save activities to {:?}", path))?;
    println!("Activity added successfully!");
    Ok(())
}

pub fn delete_activity(
    activities: &mut Vec<Activity>,
    id: u32,
    path: &Path,
) -> Result<()> {
    let initial_len = activities.len();
    activities.retain(|a| a.id() != id);

    if activities.len() == initial_len {
        println!("No activity found with ID {}", id);
    } else {
        Activity::save_all_to_file(activities, path)?;
        println!("Activity {} deleted successfully!", id);
    }
    Ok(())
}

pub fn edit_activity(
    activities: &mut Vec<Activity>,
    id: u32,
    new_name: Option<String>,
    new_minutes: Option<u32>,
    path: &Path,
) -> Result<()> {
    if let Some(act) = activities.iter_mut().find(|a| a.id() == id) {
        if let Some(name) = new_name {
            act.name = name;
        }
        if let Some(minutes) = new_minutes {
            act.duration_minutes = minutes;
        }
        Activity::save_all_to_file(activities, path)?;
        println!("Activity {} updated successfully!", id);
    } else {
        println!("No activity found with ID {}", id);
    }
    Ok(())
}

pub fn list_activities(activities: &[Activity]) {
    if activities.is_empty() {
        println!("No activities recorded yet.");
        return;
    }

    let mut sorted = activities.to_vec();
    sorted.sort_by(|a, b| b.duration_minutes().cmp(&a.duration_minutes()));
    let total: u32 = activities.iter().map(|a| a.duration_minutes()).sum();

    println!("ID | Name     | Duration (mins)");
    println!("-------------------------------");
    for act in &sorted {
        println!("{:2} | {:8} | {:>3}", act.id(), act.name(), act.duration_minutes());
    }
    println!("-------------------------------");
    println!("Total minutes: {}", total);
}

pub fn filter_activities(
    activities: &[Activity],
    min: Option<u32>,
    max: Option<u32>,
) {
    let filtered: Vec<&Activity> = activities.iter()
        .filter(|a| {
            (min.map_or(true, |min_val| a.duration_minutes() >= min_val)) &&
            (max.map_or(true, |max_val| a.duration_minutes() <= max_val))
        })
        .collect();

    if filtered.is_empty() {
        println!("No activities match the filter criteria.");
        return;
    }

    println!("Filtered activities:");
    println!("ID | Name     | Duration (mins)");
    println!("-------------------------------");
    for act in &filtered {
        println!("{:2} | {:8} | {:>3}", act.id(), act.name(), act.duration_minutes());
    }

    let total: u32 = filtered.iter().map(|a| a.duration_minutes()).sum();
    let average: f32 = total as f32 / filtered.len() as f32;
    println!("\nTotal minutes: {}", total);
    println!("Average minutes: {:.2}", average);
}
