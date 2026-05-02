use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: String,
    pub due: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TaskList {
    tasks: Vec<Task>,
}

pub fn get_storage_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".dotdo").join("tasks.json"))
}

pub fn fetch_active_monsters() -> Vec<Task> {
    let path = match get_storage_path() {
        Some(p) if p.exists() => p,
        _ => return vec![], // dotdo directory or file doesn't exist
    };

    let data = fs::read_to_string(path).unwrap_or_default();
    let task_list: TaskList = serde_json::from_str(&data).unwrap_or_default();
    let tasks = task_list.tasks;

    // Filter for monsters that are actually a threat
    tasks
        .into_iter()
        .filter(|t| t.status == "todo" || t.status == "doing")
        .collect()
}