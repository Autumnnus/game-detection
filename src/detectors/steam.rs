use std::collections::HashSet;
use std::path::Path;

pub fn discover_steam_games() -> HashSet<String> {
    let mut games = HashSet::new();
    #[cfg(target_os = "windows")]
    let steam_path = r"C:\Program Files (x86)\Steam\steamapps";

    if Path::new(steam_path).exists() {
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

fn parse_acf_file(path: &Path) -> Option<String> {
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
