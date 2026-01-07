use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use serde_json::to_string_pretty;
use crate::models::GameSession;

pub fn save_sessions_to_json(sessions: &HashMap<u32, GameSession>) -> std::io::Result<()> {
    let session_list: Vec<&GameSession> = sessions.values().collect();
    let json_data = to_string_pretty(&session_list)?;

    let mut file = File::create("active_sessions.json")?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}
