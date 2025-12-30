use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
struct Win32_Process {
    Name: String,
    ProcessId: u32,
}

#[derive(Serialize, Debug, Clone)]
struct GameSession {
    game_name: String,
    process_id: u32,
    start_time: DateTime<Local>,
    last_seen: DateTime<Local>,
    duration_seconds: i64,
    is_active: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Preparing game list...");
    let game_cache: HashSet<String> = discover_steam_games()
        .into_iter()
        .map(|name| name.to_lowercase())
        .collect();

    println!("2. Session Tracking Started. (Writing to active_sessions.json)");

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let mut active_sessions: HashMap<u32, GameSession> = HashMap::new();

    loop {
        let processes: Vec<Win32_Process> = wmi_con.query()?;

        let current_pids: HashSet<u32> = processes.iter().map(|p| p.ProcessId).collect();
        let now = Local::now();

        for process in &processes {
            let p_name_lower = process.Name.to_lowercase();

            if game_cache.contains(&p_name_lower) {
                active_sessions.entry(process.ProcessId)
                    .and_modify(|session| {
                        session.last_seen = now;
                        session.duration_seconds = (now - session.start_time).num_seconds();
                    })
                    .or_insert_with(|| {
                        println!("🚀 GAME STARTED: {} (PID: {})", process.Name, process.ProcessId);
                        GameSession {
                            game_name: process.Name.clone(),
                            process_id: process.ProcessId,
                            start_time: now,
                            last_seen: now,
                            duration_seconds: 0,
                            is_active: true,
                        }
                    });
            }
        }

        let mut closed_sessions = Vec::new();

        active_sessions.retain(|pid, session| {
            if !current_pids.contains(pid) {
                session.is_active = false;
                closed_sessions.push(session.clone());
                println!("🛑 GAME ENDED: {} (Duration: {} sec)", session.game_name, session.duration_seconds);
                return false;
            }
            true
        });

        save_sessions_to_json(&active_sessions)?;

        thread::sleep(Duration::from_secs(1));
    }
}

fn save_sessions_to_json(sessions: &HashMap<u32, GameSession>) -> std::io::Result<()> {
    let session_list: Vec<&GameSession> = sessions.values().collect();
    let json_data = to_string_pretty(&session_list)?;

    let mut file = File::create("active_sessions.json")?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

fn discover_steam_games() -> HashSet<String> {
    let mut games = HashSet::new();
    #[cfg(target_os = "windows")]
    let steam_path = r"C:\Program Files (x86)\Steam\steamapps";

    if std::path::Path::new(steam_path).exists() {
        if let Ok(entries) = std::fs::read_dir(steam_path) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("acf") {
                    if let Some(game_name) = parse_acf_file(&entry.path()) {
                        games.insert(game_name);
                    }
                }
            }
        }
    }
    println!("-> Total {} Steam games cached.", games.len());
    games
}

fn parse_acf_file(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let install_dir = content.lines()
        .find(|l| l.to_lowercase().contains("\"installdir\""))?
        .split('"').nth(3)?;

    let game_folder = format!(r"C:\Program Files (x86)\Steam\steamapps\common\{}", install_dir);

    if let Ok(entries) = std::fs::read_dir(&game_folder) {
        let mut exes: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("exe"))
            .filter_map(|e| {
                let size = e.metadata().ok()?.len();
                let name = e.file_name().to_string_lossy().to_string();
                Some((name, size))
            })
            .collect();

        exes.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((exe_name, _)) = exes.first() {
            return Some(exe_name.clone());
        }
    }
    None
}