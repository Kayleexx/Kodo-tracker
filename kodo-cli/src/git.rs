use anyhow::{Context, Result};
use git2::Repository;
use std::path::Path;
use chrono::{DateTime, Utc, Local};
use kodo_core::Activity;

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::open(path)
        .with_context(|| format!("Failed to open git repository at {:?}", path))
}

pub fn get_github_activities(repo_path: &Path, max: usize) -> Result<Vec<Activity>> {
    let repo = open_repo(repo_path)?;
    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;
    revwalk.push_head().context("Failed to push HEAD")?;

    let mut commits = Vec::new();
    for oid_result in revwalk.take(max) {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        let msg = commit.message().unwrap_or("no message").to_string();
        let timestamp = commit.time().seconds();
        let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        commits.push((msg, datetime));
    }

    commits.sort_by(|a, b| b.1.cmp(&a.1));

    let mut activities = Vec::new();
    for i in 0..commits.len() {
        let duration_minutes = if i + 1 < commits.len() {
            let diff = commits[i].1 - commits[i + 1].1;
            std::cmp::max(diff.num_minutes() as u32, 1) // at least 1 min
        } else {
            1 // last commit fallback
        };

        activities.push(Activity {
            id: (i + 1) as u32,
            name: commits[i].0.clone(),
            duration_minutes,
            date: commits[i].1.with_timezone(&Local).format("%Y-%m-%d").to_string(),
        });
    }

    Ok(activities)
}

pub fn sync_commits_to_file(repo_path: &Path, activities_path: &Path, max: usize) -> Result<()> {
    let commits = get_github_activities(repo_path, max)?;

    let mut existing = if activities_path.exists() {
        Activity::load_from_file(activities_path)?
    } else {
        Vec::new()
    };

    for commit in commits {
        if !existing.iter().any(|a| a.name == commit.name && a.date == commit.date) {
            let next_id = existing.iter().map(|a| a.id).max().unwrap_or(0) + 1;
            existing.push(Activity {
                id: next_id,
                name: commit.name.clone(),
                duration_minutes: commit.duration_minutes,
                date: commit.date.clone(),
            });
        }
    }

    Activity::save_all_to_file(&existing, activities_path)?;
    Ok(())
}