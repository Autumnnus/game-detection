use std::collections::HashSet;
use std::path::PathBuf;
use windows::ApplicationModel::Package;
use windows::Management::Deployment::PackageManager;

pub fn discover_xbox_games() -> HashSet<String> {
    let mut games = HashSet::new();
    println!("🔍 Scanning for Xbox/Microsoft Store games...");

    let package_manager = match PackageManager::new() {
        Ok(pm) => pm,
        Err(e) => {
            eprintln!("⚠️ Failed to create PackageManager: {}", e);
            return games;
        }
    };

    // Use FindPackagesByUserSecurityId which finds packages for the specific user
    let user_security_id = windows::core::HSTRING::from("");
    let packages = match package_manager.FindPackagesByUserSecurityId(&user_security_id) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("⚠️ Failed to find packages: {}", e);
            return games;
        }
    };

    for package in packages {
        let display_name = match package.DisplayName() {
            Ok(name) => name.to_string(),
            Err(_) => "Unknown".to_string(),
        };

        match check_if_game(&package, &display_name) {
            Ok(Some(exe_name)) => {
                println!("✅ Found Xbox Game: {} (Exe: {})", display_name, exe_name);
                games.insert(exe_name);
            }
            Ok(None) => {
                // Not a game or system app
            }
            Err(reason) => {
                println!("⚠️ skipped {}: {}", display_name, reason);
            }
        }
    }

    games
}

fn check_if_game(package: &Package, name: &str) -> Result<Option<String>, String> {
    let installed_location = package.InstalledLocation()
        .map_err(|_| "No installed location access".to_string())?;

    let path_hstring = installed_location.Path()
        .map_err(|_| "Failed to get path string".to_string())?;

    let path = PathBuf::from(path_hstring.to_string());

    // Priority: If MicrosoftGame.config exists, it's a game
    let game_config = path.join("MicrosoftGame.config");
    if game_config.exists() {
        if let Some(exe) = find_real_game_exe(&path, name) {
            return Ok(Some(exe));
        }
    }

    // Manifest control (old UWP games)
    let manifest_path = path.join("AppxManifest.xml");
    if !manifest_path.exists() {
        return Err("Manifest not found".to_string());
    }

    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let is_game = content.contains("Category=\"windows.game\"") ||
                  content.contains("uap3:GameMode");

    if is_game {
        if let Some(exe) = find_real_game_exe(&path, name) {
            return Ok(Some(exe));
        }
    }

    Ok(None)
}
fn find_real_game_exe(package_path: &PathBuf, game_name: &str) -> Option<String> {
    let config_path = package_path.join("MicrosoftGame.config");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Some(start) = content.find("<ExecutableName>") {
                let rest = &content[start + 16..];
                if let Some(end) = rest.find("</ExecutableName>") {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }

    let skip_keywords = ["launcher", "helper", "crash", "server", "unity", "report", "redist"];
    let game_name_lower = game_name.to_lowercase().replace(" ", "");
    
    let mut candidates: Vec<(String, u64)> = Vec::new();
    
    for entry in walkdir::WalkDir::new(package_path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "exe") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let name_lower = name.to_lowercase();
                
                if skip_keywords.iter().any(|kw| name_lower.contains(kw)) {
                    continue;
                }
                
                if let Ok(meta) = std::fs::metadata(path) {
                    candidates.push((name.to_string(), meta.len()));
                }
            }
        }
    }

    if let Some((name, _)) = candidates.iter().find(|(n, _)| {
        n.to_lowercase().replace(" ", "").contains(&game_name_lower)
    }) {
        return Some(name.clone());
    }

    candidates.into_iter()
        .filter(|(_, size)| *size > 10_000_000)
        .max_by_key(|(_, size)| *size)
        .map(|(name, _)| name)
}