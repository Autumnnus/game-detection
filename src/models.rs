use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct GameSession {
    pub game_name: String,
    pub process_id: u32,
    pub start_time: DateTime<Local>,
    pub last_seen: DateTime<Local>,
    pub duration_seconds: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}
