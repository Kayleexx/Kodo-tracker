use clap::{Parser, Subcommand};
use kodo_core::Activity;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(name = "kodo", about = "A dev activity tracker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add { name: String, minutes: u32 },
    Delete { id: u32 },
    Edit {
    id: u32,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    minutes: Option<u32>,
},
    List,
    Filter {
    #[arg(long)]
    min: Option<u32>,

    #[arg(long)]
    max: Option<u32>,
},
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_string = cli.file.clone().unwrap_or_else(|| "activities.json".to_string());
    let path = Path::new(&file_string);


let mut activities = if path.exists() {
    let acts = Activity::load_from_file(path)
        .with_context(|| format!("Failed to load activities from {:?}", path))?;

    if acts.is_empty() {
        println!("No activities found. Initializing empty list.");
    }
    acts
} else {
 
    println!("activities.json not found. Creating a new one...");
    let acts = Vec::new();
    Activity::save_all_to_file(&acts, path)
        .with_context(|| format!("Failed to create {:?}", path))?;
    acts
};

    match cli.command {
    Commands::Add { name, minutes } => add_activity(&mut activities, &name, minutes, path)?,
    Commands::Delete { id } => delete_activity(&mut activities, id, path)?,
    Commands::Edit { id, name, minutes } => edit_activity(&mut activities, id, name, minutes, path)?,
    Commands::List => list_activities(&activities),
    Commands::Filter { min, max } => filter_activities(&activities, min, max),

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
        Activity::save_all_to_file(activities, path)?;
        println!("Activity {} deleted successfully!", id);
    }
    Ok(())
}

fn edit_activity(
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

fn list_activities(activities: &[Activity]) {
    if activities.is_empty() {
        println!("No activities recorded yet.");
    } else {
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
}

fn filter_activities(
    activities: &[Activity],
    min: Option<u32>,
    max: Option<u32>
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



