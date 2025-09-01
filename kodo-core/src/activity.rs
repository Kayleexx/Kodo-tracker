use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: u32,
    pub name: String,
    pub duration_minutes: u32,
}

impl Activity {
    pub fn new_with_id(id: u32, name: &str, duration_minutes: u32) -> Self {
        Self {
            id,
            name: name.to_string(),
            duration_minutes,
        }
    }

    pub fn id(&self) -> u32 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn duration_minutes(&self) -> u32 { self.duration_minutes }

    pub fn save_all_to_file(activities: &[Activity], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(activities)?;  // pretty JSON
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Vec<Activity>> {
        let contents = std::fs::read_to_string(path)?;
        let acts: Vec<Activity> = serde_json::from_str(&contents)?;
        Ok(acts)
    }
}
