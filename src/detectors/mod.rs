pub mod steam;
pub mod epic;
pub mod ubisoft;

use std::collections::HashSet;

pub fn scan_all_games() -> HashSet<String> {
    println!("1. Preparing game list...");
    
    let steam_games = steam::discover_steam_games();
    println!("🎮 Steam Games: {:?}", steam_games);
    
    let epic_games = epic::discover_epic_games();
    println!("🎮 Epic Games: {:?}", epic_games);
    
    let ubisoft_games = ubisoft::discover_ubisoft_games();
    println!("🎮 Ubisoft Games: {:?}", ubisoft_games);

    let mut all_games = HashSet::new();
    
    all_games.extend(steam_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(epic_games.into_iter().map(|s| s.to_lowercase()));
    all_games.extend(ubisoft_games.into_iter().map(|s| s.to_lowercase()));
    
    all_games
}
