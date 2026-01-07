use rusqlite::Connection;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn discover_amazon_games() -> HashSet<String> {
    let mut games = HashSet::new();

    let db_paths = get_all_amazon_db_paths();

    for db_path in db_paths {
        if db_path.exists() {
            if let Ok(games_list) = parse_amazon_database(&db_path) {
                games.extend(games_list);
            }
        }
    }

    println!("-> Total {} Amazon Games cached.", games.len());
    games
}

fn get_all_amazon_db_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(path) = get_amazon_path() {
        paths.push(PathBuf::from(path).join("GameInstallInfo.sqlite"));
    }

    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(localappdata)
                .join("Amazon Games")
                .join("Data")
                .join("Games")
                .join("Sql")
                .join("GameInstallInfo.sqlite")
        );
    }

    if let Some(fuel_paths) = get_fuel_library_paths() {
        for fuel_path in fuel_paths {
            paths.push(
                PathBuf::from(fuel_path)
                    .join("Data")
                    .join("Games")
                    .join("Sql")
                    .join("GameInstallInfo.sqlite")
            );
        }
    }

    paths.into_iter().collect::<HashSet<_>>().into_iter().collect()
}

fn get_fuel_library_paths() -> Option<Vec<String>> {
    use serde_json::Value;

    let localappdata = std::env::var("LOCALAPPDATA").ok()?;
    let fuel_path = PathBuf::from(localappdata)
        .join("Amazon Games")
        .join("Data")
        .join("fuel.json");

    let content = std::fs::read_to_string(fuel_path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    let paths: Vec<String> = json
        .get("LibraryLocations")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    Some(paths)
}

fn get_amazon_path() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(amazon_key) = hkcu.open_subkey(r"SOFTWARE\Amazon Games") {
        if let Ok(install_path) = amazon_key.get_value::<String, _>("InstallPath") {
            return Some(format!(r"{}\Data\Games\Sql", install_path));
        }
    }

    None
}

fn parse_amazon_database(db_path: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    use rusqlite::Connection;

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT InstallDirectory FROM DbSet WHERE Installed = 1")?;

    let games: HashSet<String> = stmt
        .query_map([], |row| {
            let install_dir: String = row.get(0)?;
            Ok(find_main_exe(&install_dir))
        })?
        .filter_map(|r| r.ok().flatten())
        .collect();

    Ok(games)
}

fn find_main_exe(game_dir: &str) -> Option<String> {
    std::fs::read_dir(game_dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("exe"))
        .max_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .map(|e| e.file_name().to_string_lossy().to_string())
}

#[cfg(target_os = "windows")]
fn get_amazon_db_path() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|local_app_data| {
        Path::new(&local_app_data)
            .join("Amazon Games")
            .join("Data")
            .join("Games")
            .join("Sql")
            .join("GameInstallInfo.sqlite")
    })
}
