use std::collections::HashSet;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

const KNOWN_GAME_PUBLISHERS: &[&str] = &[
    "ubisoft", "ea games", "electronic arts", "riot games", "rockstar games",
    "bethesda", "activision", "blizzard", "valve", "epic games", "2k games",
    "capcom", "square enix", "bandai namco", "sega", "konami", "thq",
    "cd projekt", "paradox", "devolver", "focus entertainment", "deep silver",
];

const GAME_ENGINE_FILES: &[&str] = &[
    "unityplayer.dll", "ue4prerequisites", "unrealengine", "cryengine",
    "fmod.dll", "bink2w64.dll", "steam_api.dll", "steam_api64.dll",
    "eossdk-win64-shipping.dll", "galaxydll.dll", "galaxy64.dll",
];

const BLACKLIST_APPS: &[&str] = &[
    "chrome", "firefox", "edge", "microsoft", "office", "visual studio",
    "discord", "spotify", "steam client", "epic games launcher", "adobe",
    "nvidia", "amd ", "intel", "realtek", "logitech", "razer", "corsair",
    "java", "python", "node", "git", "7-zip", "winrar", "vlc", "k-lite",
    "directx", "visual c++", "redistributable", ".net", "framework",
];

pub fn discover_registry_games() -> HashSet<String> {
    let mut games = HashSet::new();

    let registry_paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (root, path) in registry_paths.iter() {
        if let Ok(key) = RegKey::predef(*root).open_subkey(path) {
            for subkey_name in key.enum_keys().flatten() {
                if let Ok(subkey) = key.open_subkey(&subkey_name) {
                    if let Some(game_exe) = process_registry_entry(&subkey) {
                        games.insert(game_exe);
                    }
                }
            }
        }
    }

    println!("-> Total {} Registry games detected.", games.len());
    games
}

fn process_registry_entry(key: &RegKey) -> Option<String> {
    println!("Processing registry entry: {:?}", key);
    let display_name: String = key.get_value("DisplayName").ok()?;
    let install_location: String = key.get_value("InstallLocation").unwrap_or_default();
println!("Display Name: {}", display_name);
println!("Install Location: {}", install_location);

    let publisher: String = key.get_value("Publisher").unwrap_or_default();

    if !is_game(&display_name, &install_location, &publisher) {
        return None;
    }

    find_game_executable(&install_location).or_else(|| {
        let uninstall_string: String = key.get_value("UninstallString").ok()?;
        extract_exe_from_uninstall(&uninstall_string)
    })
}

fn is_game(name: &str, install_path: &str, publisher: &str) -> bool {
    let name_lower = name.to_lowercase();
    let publisher_lower = publisher.to_lowercase();

    if BLACKLIST_APPS.iter().any(|&app| name_lower.contains(app)) {
        return false;
    }

    if KNOWN_GAME_PUBLISHERS.iter().any(|&pub_name| publisher_lower.contains(pub_name)) {
        return true;
    }

    if !install_path.is_empty() && has_game_engine_files(install_path) {
        return true;
    }

    false
}

fn has_game_engine_files(install_path: &str) -> bool {
    let path = Path::new(install_path);
    if !path.exists() {
        return false;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            if GAME_ENGINE_FILES.iter().any(|&engine_file| file_name.contains(engine_file)) {
                return true;
            }
        }
    }

    false
}

fn find_game_executable(install_path: &str) -> Option<String> {
    if install_path.is_empty() {
        return None;
    }

    let path = Path::new(install_path);
    if !path.exists() {
        return None;
    }

    let mut exes: Vec<(String, u64)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().and_then(|s| s.to_str()) == Some("exe") {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    if !name_lower.contains("unins") 
                        && !name_lower.contains("setup") 
                        && !name_lower.contains("launcher")
                        && !name_lower.contains("crash")
                        && !name_lower.contains("helper")
                        && !name_lower.contains("update")
                        && !name_lower.contains("redist") {
                        exes.push((name, metadata.len()));
                    }
                }
            }
        }
    }

    exes.sort_by(|a, b| b.1.cmp(&a.1));
    exes.first().map(|(name, _)| name.clone())
}

fn extract_exe_from_uninstall(uninstall_string: &str) -> Option<String> {
    let cleaned = uninstall_string.trim_matches('"');
    let path = Path::new(cleaned);
    
    if let Some(parent) = path.parent() {
        return find_game_executable(parent.to_str()?);
    }
    
    None
}
