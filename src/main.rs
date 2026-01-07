mod models;
mod detectors;
mod tracker;
mod storage;
mod system;

use std::thread;
use std::time::Duration;
use crate::tracker::SessionTracker;
use crate::system::SystemScanner;
use crate::storage::save_sessions_to_json;
use crate::detectors::scan_all_games;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_cache = scan_all_games();

    println!("2. Session Tracking Started. (Writing to active_sessions.json)");

    let mut session_tracker = SessionTracker::new(game_cache);
    let system_scanner = SystemScanner::new()?;

    loop {
        match system_scanner.get_running_processes() {
            Ok(processes) => {
                let active_sessions = session_tracker.update(&processes);
                
                if let Err(e) = save_sessions_to_json(active_sessions) {
                    eprintln!("Error saving sessions: {}", e);
                }
            },
            Err(e) => eprintln!("Error querying processes: {}", e),
        }

        thread::sleep(Duration::from_secs(1));
    }
}