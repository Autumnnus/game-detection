use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use wmi::{COMLibrary, WMIConnection};
use winreg::enums::*;
use winreg::RegKey;

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
    let steam_games = discover_steam_games();
    println!("🎮 Steam Games: {:?}", steam_games);
    let mut game_cache: HashSet<String> = steam_games
        .into_iter()
        .map(|name| name.to_lowercase())
        .collect();

    let epic_games = discover_epic_games();
    println!("🎮 Epic Games: {:?}", epic_games);
    game_cache.extend(
        epic_games
            .into_iter()
            .map(|name| name.to_lowercase())
    );
    
    let ubisoft_games = discover_ubisoft_games();
    println!("🎮 Ubisoft Games: {:?}", ubisoft_games);
    game_cache.extend(
        ubisoft_games
            .into_iter()
            .map(|name| name.to_lowercase())
    );



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

fn discover_epic_games() -> HashSet<String> {
    let mut games = HashSet::new();
    #[cfg(target_os = "windows")]
    let epic_path = r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";

    if std::path::Path::new(epic_path).exists() {
        if let Ok(entries) = std::fs::read_dir(epic_path) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("item") {
                    if let Some(game_name) = parse_epic_manifest(&entry.path()) {
                        games.insert(game_name);
                    }
                }
            }
        }
    }
    println!("-> Total {} Epic Games cached.", games.len());
    games
}

fn parse_epic_manifest(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let exe_path = json.get("LaunchExecutable")?.as_str()?;
    let exe_name = std::path::Path::new(exe_path)
        .file_name()?
        .to_string_lossy()
        .to_string();

    Some(exe_name)
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

// Ubisoft

fn discover_ubisoft_games() -> HashSet<String> {
    let mut games = HashSet::new();

    #[cfg(target_os = "windows")]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // 64-bit ve 32-bit registry yollarını dene
        let registry_paths = vec![
            r"SOFTWARE\Ubisoft\Launcher\Installs",
            r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs",
        ];

        for reg_path in registry_paths {
            if let Ok(installs_key) = hklm.open_subkey(reg_path) {
                for install_id in installs_key.enum_keys().flatten() {
                    if let Ok(game_key) = installs_key.open_subkey(&install_id) {
                        if let Ok(install_dir) = game_key.get_value::<String, _>("InstallDir") {
                            if let Some(exe_name) = find_largest_exe_in_dir(&install_dir) {
                                games.insert(exe_name);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("-> Total {} Ubisoft games cached.", games.len());
    games
}

fn find_largest_exe_in_dir(dir_path: &str) -> Option<String> {
    let path = std::path::Path::new(dir_path);
    if !path.exists() {
        return None;
    }

    let excluded_patterns = [
        "_ds", "dedicated", "server",
        "launcher", "setup", "install", "unins",
        "crash", "reporter", "redist", "prerequisite",
        "update", "patcher", "config"
    ];

    let mut exes: Vec<(String, u64)> = std::fs::read_dir(path).ok()?
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("exe"))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();

            if excluded_patterns.iter().any(|&pattern| name_lower.contains(pattern)) {
                return None;
            }

            let size = e.metadata().ok()?.len();
            Some((name, size))
        })
        .collect();

    exes.sort_by(|a, b| b.1.cmp(&a.1));
    exes.first().map(|(name, _)| name.clone())
}