use std::collections::HashSet;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

pub fn discover_ubisoft_games() -> HashSet<String> {
    let mut games = HashSet::new();

    #[cfg(target_os = "windows")]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

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
    let path = Path::new(dir_path);
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
