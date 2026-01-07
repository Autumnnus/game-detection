use std::collections::HashSet;
use std::path::Path;

#[cfg(target_os = "windows")]
fn get_steam_path() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(steam_key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam") {
        if let Ok(install_path) = steam_key.get_value::<String, _>("InstallPath") {
            return Some(format!(r"{}\steamapps", install_path));
        }
    }

    Some(r"C:\Program Files (x86)\Steam\steamapps".to_string())
}

pub fn discover_steam_games() -> HashSet<String> {
    let mut games = HashSet::new();

    let steam_path = get_steam_path()
        .unwrap_or_else(|| r"C:\Program Files (x86)\Steam\steamapps".to_string());

    let library_folders = get_steam_library_folders(&steam_path);

    for library_path in library_folders {
        if let Ok(entries) = std::fs::read_dir(&library_path) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("acf") {
                    let steam_common = Path::new(&library_path).join("common");
                    if let Some(game_name) = parse_acf_file(&entry.path(), &steam_common) {
                        games.insert(game_name);
                    }
                }
            }
        }
    }

    println!("-> Total {} Steam games cached.", games.len());
    games
}

fn get_steam_library_folders(steam_path: &str) -> Vec<String> {
    let mut folders = vec![steam_path.to_string()];

    let vdf_path = Path::new(steam_path).join("libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(vdf_path) {
        for line in content.lines() {
            if line.contains("\"path\"") {
                if let Some(path) = line.split('"').nth(3) {
                    let library_path = format!(r"{}\steamapps", path.replace("\\\\", "\\"));
                    folders.push(library_path);
                }
            }
        }
    }
    // println!("Steam folders count: {}", folders.len());
    // for f in &folders {
    //     println!("Steam folder: {}", f);
    // }
    folders
}

fn parse_acf_file(path: &Path, steam_common: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let install_dir = content.lines()
        .find(|l| l.to_lowercase().contains("\"installdir\""))?
        .split('"').nth(3)?;

    let game_folder = steam_common.join(install_dir);

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