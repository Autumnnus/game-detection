use std::collections::{HashMap, HashSet};
use chrono::Local;
use crate::models::{GameSession, ProcessInfo};

pub struct SessionTracker {
    active_sessions: HashMap<u32, GameSession>,
    game_cache: HashSet<String>,
}

impl SessionTracker {
    pub fn new(game_cache: HashSet<String>) -> Self {
        Self {
            active_sessions: HashMap::new(),
            game_cache,
        }
    }

    pub fn update(&mut self, processes: &[ProcessInfo]) -> &HashMap<u32, GameSession> {
        let current_pids: HashSet<u32> = processes.iter().map(|p| p.pid).collect();
        let now = Local::now();

        for process in processes {
            let p_name_lower = process.name.to_lowercase();

            if self.game_cache.contains(&p_name_lower) {
                self.active_sessions.entry(process.pid)
                    .and_modify(|session| {
                        session.last_seen = now;
                        session.duration_seconds = (now - session.start_time).num_seconds();
                    })
                    .or_insert_with(|| {
                        println!("🚀 GAME STARTED: {} (PID: {})", process.name, process.pid);
                        GameSession {
                            game_name: process.name.clone(),
                            process_id: process.pid,
                            start_time: now,
                            last_seen: now,
                            duration_seconds: 0,
                            is_active: true,
                        }
                    });
            }
        }

        self.active_sessions.retain(|pid, session| {
            if !current_pids.contains(pid) {
                session.is_active = false;
                println!("🛑 GAME ENDED: {} (Duration: {} sec)", session.game_name, session.duration_seconds);
                return false; 
            }
            true
        });

        &self.active_sessions
    }
    
}
