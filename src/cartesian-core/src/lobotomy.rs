use sysinfo::{Pid, Process, System};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config; // Import Config

/// THE REGISTRY
/// Categorizes processes to determine User Context.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum AppCategory {
    Game,        // High Priority: Steam, CS2 -> Sidekick Mode
    Production,  // High VRAM: Blender, OBS -> Conscientious Mode
    Development, // High CPU: VS Code, Cargo -> God Mode
    Web,         // Low Priority: Firefox, Chrome
    Media,       // Low Priority: Spotify, VLC
    System,      // Ignored
    Unknown,     // Needs AI Classification
}

// ... ProcessRecord struct ...

pub struct SystemMonitor {
    sys: System,
    cached_pid: Option<Pid>,
    registry: HashMap<String, AppCategory>,
    // Removed registry_path field, using config directly
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            sys: System::new_all(),
            cached_pid: None,
            registry: HashMap::new(),
        };
        monitor.load_registry();
        monitor
    }

    fn load_registry(&mut self) {
        if Path::new(config::REGISTRY_PATH).exists() {
            if let Ok(data) = fs::read_to_string(config::REGISTRY_PATH) {
                if let Ok(records) = serde_json::from_str::<HashMap<String, AppCategory>>(&data) {
                    self.registry = records;
                    return;
                }
            }
        }
        // Defaults are now minimal, relying on Heuristics
        self.registry.insert("firefox".into(), AppCategory::Web);
    }

    pub fn save_registry(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.registry) {
            let _ = fs::write(config::REGISTRY_PATH, data);
        }
    }

    pub fn get_system_context(&mut self) -> (AppCategory, Vec<String>) {
        self.sys.refresh_processes();
        
        let mut active_categories = HashMap::new();
        let mut unknown_apps = Vec::new();

        for (_pid, process) in self.sys.processes() {
            let name = process.name().to_lowercase();
            
            // 1. Check Config Heuristics (Hardcoded Overrides)
            // This ensures new games work even if not in JSON registry yet
            if config::GAMES.iter().any(|&g| name.contains(g)) {
                *active_categories.entry(AppCategory::Game).or_insert(0) += 1;
                continue;
            }
            if config::CREATIVE_SUITE.iter().any(|&c| name.contains(c)) {
                *active_categories.entry(AppCategory::Production).or_insert(0) += 1;
                continue;
            }
            if config::DEV_TOOLS.iter().any(|&d| name.contains(d)) {
                *active_categories.entry(AppCategory::Development).or_insert(0) += 1;
                continue;
            }

            // 2. Check JSON Registry
            match self.registry.get(&name) {
                Some(cat) => {
                    *active_categories.entry(*cat).or_insert(0) += 1;
                },
                None => {
                    if process.memory() > 50_000_000 { 
                        unknown_apps.push(name.clone());
                    }
                }
            }
        }

        // Determine Dominance
        let dominant = if active_categories.contains_key(&AppCategory::Game) {
            AppCategory::Game
        } else if active_categories.contains_key(&AppCategory::Production) {
            AppCategory::Production
        } else if active_categories.contains_key(&AppCategory::Development) {
            AppCategory::Development
        } else {
            AppCategory::System
        };

        (dominant, unknown_apps)
    }

    // Pass-through
    pub fn get_vitals(&mut self) -> (f32, f32) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        let cpu = self.sys.global_cpu_info().cpu_usage();
        let ram = self.sys.available_memory() as f32 / 1_073_741_824.0;
        (cpu, ram)
    }
    
    // Legacy pass-through
    pub fn find_process(&mut self, name: &str) -> Option<Pid> {
        self.sys.refresh_processes();
        for (pid, process) in self.sys.processes() {
            if process.name().to_lowercase().contains(&name.to_lowercase()) {
                return Some(*pid);
            }
        }
        None
    }
}