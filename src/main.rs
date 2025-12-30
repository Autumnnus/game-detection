use serde::Deserialize;
use std::collections::HashSet;
use wmi::{COMLibrary, WMIConnection};
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct Win32_Process {
    Name: String,
    ProcessId: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Oyunlar taranıyor...");
    let game_cache: HashSet<String> = discover_steam_games()
        .into_iter()
        .map(|name| name.to_lowercase())
        .collect();

    println!("2. İzleme Başlatıldı (Polling Mode).");

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let mut seen_pids: HashSet<u32> = HashSet::new();

    loop {
        let processes: Vec<Win32_Process> = wmi_con.query()?;

        for process in &processes {
            if !seen_pids.contains(&process.ProcessId) {
                let p_name = process.Name.to_lowercase();
                if game_cache.contains(&p_name) {
                    println!("🚀 OYUN BAŞLADI: {} (PID: {})", process.Name, process.ProcessId);
                }
                seen_pids.insert(process.ProcessId);
            }
        }

        seen_pids.retain(|pid| processes.iter().any(|p| p.ProcessId == *pid));

        thread::sleep(Duration::from_millis(500));
    }
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

    println!("-> Toplam {} Steam oyunu önbelleğe alındı.", games.len());
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

        if let Some((exe_name, size)) = exes.first() {
            println!("   Bulunan: {} ({} MB)", exe_name, size / 1024 / 1024);
            return Some(exe_name.clone());
        }
    }
    None
}