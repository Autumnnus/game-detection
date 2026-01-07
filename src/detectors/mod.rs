pub mod amazon;
pub mod epic;
pub mod registry;
pub mod steam;
pub mod ubisoft;
pub mod xbox;

use std::collections::HashSet;

const ENABLE_REGISTRY_SCAN: bool = false;

pub fn scan_all_games() -> HashSet<String> {
    println!("1. Preparing game list...");

    let steam_games = steam::discover_steam_games();
    println!("🎮 Steam Games: {:?}", steam_games);

    let epic_games = epic::discover_epic_games();
    println!("🎮 Epic Games: {:?}", epic_games);

    let ubisoft_games = ubisoft::discover_ubisoft_games();
    println!("🎮 Ubisoft Games: {:?}", ubisoft_games);

    let xbox_games = xbox::discover_xbox_games();
    println!("🎮 Xbox/Store Games: {:?}", xbox_games);

    let amazon_games = amazon::discover_amazon_games();
    println!("🎮 Amazon Games: {:?}", amazon_games);

    let mut all_games = HashSet::new();

    all_games.extend(steam_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(epic_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(ubisoft_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(xbox_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(amazon_games.into_iter().map(|s| s.to_lowercase()));

    if ENABLE_REGISTRY_SCAN {
        let registry_games = registry::discover_registry_games();
        println!("🎮 Registry Games: {:?}", registry_games);
        all_games.extend(registry_games.into_iter().map(|s| s.to_lowercase()));
    }

    all_games
}
