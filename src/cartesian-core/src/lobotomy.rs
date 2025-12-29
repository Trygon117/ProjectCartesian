use sysinfo::{Pid, System, ProcessesToUpdate};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config; 

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum AppCategory {
    Game,        
    Production,  
    Development, 
    Web,         
    Media,       
    System,      
    Unknown,     
}

pub struct SystemMonitor {
    sys: System,
    cached_pid: Option<Pid>,
    registry: HashMap<String, AppCategory>,
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
        self.registry.insert("firefox".into(), AppCategory::Web);
    }

    pub fn save_registry(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.registry) {
            let _ = fs::write(config::REGISTRY_PATH, data);
        }
    }

    pub fn get_system_context(&mut self) -> (AppCategory, Vec<String>) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        
        let mut active_categories = HashMap::new();
        let mut unknown_apps = Vec::new();

        for (_pid, process) in self.sys.processes() {
            // FIXED: Convert OsString to String immediately using lossy conversion
            // This fixes E0599 (no method contains) and E0308 (mismatched types)
            let name = process.name().to_string_lossy().to_ascii_lowercase();
            
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

    pub fn get_vitals(&mut self) -> (f32, f32) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        let cpu = self.sys.global_cpu_usage();
        let ram = self.sys.available_memory() as f32 / 1_073_741_824.0;
        (cpu, ram)
    }
    
    pub fn find_process(&mut self, name: &str) -> Option<Pid> {
        if let Some(pid) = self.cached_pid {
            self.sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            if self.sys.processes().contains_key(&pid) {
                return Some(pid);
            } else {
                self.cached_pid = None; 
            }
        }

        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        for (pid, process) in self.sys.processes() {
            // FIXED: name() -> to_string_lossy() -> to_ascii_lowercase
            if process.name().to_string_lossy().to_ascii_lowercase().contains(&name.to_ascii_lowercase()) {
                self.cached_pid = Some(*pid);
                return Some(*pid);
            }
        }
        None
    }
}