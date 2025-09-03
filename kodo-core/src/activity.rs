use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use anyhow::{Result, Context};
use chrono::Local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: u32,
    pub name: String,
    pub duration_minutes: u32,
    pub date: String,
}

impl Activity {
    pub fn new_with_id(id: u32, name: &str, duration_minutes: u32) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();

        Self {
            id,
            name: name.to_string(),
            duration_minutes,
            date: today,
        }
    }

    pub fn id(&self) -> u32 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn duration_minutes(&self) -> u32 { self.duration_minutes }

    pub fn save_all_to_file(activities: &[Activity], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(activities)?; 
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Vec<Activity>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if contents.trim().is_empty() {
        return Ok(Vec::new()); 
    }

    let acts: Vec<Activity> = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON in {:?}", path))?;

    Ok(acts)
}

}
