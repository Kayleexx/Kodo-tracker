use clap::{Parser, Subcommand};
use kodo_core::Activity;
use std::path::Path;
use anyhow::{Result, Context};

mod git;
mod tui;
mod cli_actions;
use crate::cli_actions::*;

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
    Dashboard,
    Commits {
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
    Sync { repo: String },
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
        Commands::Add { name, minutes } => {
            add_activity(&mut activities, &name, minutes, path)?
        }
        Commands::Delete { id } => delete_activity(&mut activities, id, path)?,
        Commands::Edit { id, name, minutes } => {
            edit_activity(&mut activities, id, name, minutes, path)?
        }
        Commands::List => list_activities(&activities),
        Commands::Filter { min, max } => filter_activities(&activities, min, max),
        Commands::Dashboard => {
            tui::run(&mut activities, path)?;
        }
        Commands::Commits { limit } => {
            let commits = git::get_github_activities(Path::new("."), limit)
                .context("Failed to fetch GitHub commits")?;
            for act in commits {
                println!("{} - {}", act.date, act.name);
            }
        }
        Commands::Sync { repo } => {
            git::sync_commits_to_file(Path::new(&repo), path, 50)
                .context("Failed to sync commits")?;
            println!("Commits synced into activities.json!");
        }
    }

    Ok(())
}