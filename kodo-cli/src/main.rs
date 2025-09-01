use clap::{Parser, Subcommand};
use kodo_core::Activity;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(name = "kodo", about = "A dev activity tracker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {

    Add { name: String, minutes: u32 },
    Delete { id: u32 },
    Edit { id: u32, name: Option<String>, minutes: Option<u32> },
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = Path::new("activities.json");
    let mut activities = if path.exists() {
        Activity::load_from_file(path)
            .with_context(|| format!("Failed to load activities from {:?}", path))?
    } else {
        Vec::new()
    };

    match cli.command {
        Commands::Add { name, minutes } => add_activity(&mut activities, &name, minutes, path)?,
        Commands::Delete { id } => delete_activity(&mut activities, id, path)?,
        Commands::Edit { id, name, minutes } => edit_activity(&mut activities, id, name, minutes, path)?,
        Commands::List => list_activities(&activities),
    }

    Ok(())
}

fn add_activity(activities: &mut Vec<Activity>, name: &str, minutes: u32, path: &Path) -> Result<()> {
    let next_id = activities.iter().map(|a| a.id()).max().unwrap_or(0) + 1;
    let act = Activity::new_with_id(next_id, name, minutes);
    activities.push(act);
    Activity::save_all_to_file(activities, path)
        .with_context(|| format!("Failed to save activities to {:?}", path))?;
    println!("Activity added successfully!");
    Ok(())
}

fn delete_activity(activities: &mut Vec<Activity>, id: u32, path: &Path) -> Result<()> {
    let initial_len = activities.len();
    activities.retain(|a| a.id() != id);

    if activities.len() == initial_len {
        println!("No activity found with ID {}", id);
    } else {
        Activity::save_all_to_file(activities, path)
            .with_context(|| format!("Failed to save activities to {:?}", path))?;
        println!("Activity {} deleted successfully!", id);
    }

    Ok(())
}

fn edit_activity(
    activities: &mut Vec<Activity>,
    id: u32,
    name: Option<String>,
    minutes: Option<u32>,
    path: &Path,
) -> Result<()> {
    if let Some(act) = activities.iter_mut().find(|a| a.id() == id) {
        if let Some(new_name) = name { act.name = new_name; }
        if let Some(new_minutes) = minutes { act.duration_minutes = new_minutes; }

        Activity::save_all_to_file(activities, path)
            .with_context(|| format!("Failed to save activities to {:?}", path))?;
        println!("Activity {} updated successfully!", id);
    } else {
        println!("No activity found with ID {}", id);
    }
    Ok(())
}


fn list_activities(activities: &[Activity]) {
    if activities.is_empty() {
        println!("No activities recorded yet.");
    } else {
        println!("ID | Name     | Duration (mins)");
        println!("-------------------------------");
        for act in activities {
            println!("{:2} | {:8} | {:>3}", act.id(), act.name(), act.duration_minutes());
        }
    }
}
