use std::collections::HashSet;
use std::path::Path;
use serde_json::Value;

pub fn discover_epic_games() -> HashSet<String> {
    let mut games = HashSet::new();
    #[cfg(target_os = "windows")]
    let epic_path = r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";

    if Path::new(epic_path).exists() {
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

fn parse_epic_manifest(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    let exe_path = json.get("LaunchExecutable")?.as_str()?;
    let exe_name = Path::new(exe_path)
        .file_name()?
        .to_string_lossy()
        .to_string();

    Some(exe_name)
}
