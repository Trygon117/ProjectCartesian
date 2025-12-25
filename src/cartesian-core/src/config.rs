use std::time::Duration;

/// GLOBAL CONFIGURATION
/// The Single Source of Truth for Paths, Constants, and Tuning.

// --- PATHS ---
pub const MODEL_DIR: &str = "/usr/share/cartesian/models/";
pub const REGISTRY_PATH: &str = "process_registry.json";
pub const MEMORY_INDEX_PATH: &str = "hippocampus_index.bin";
pub const SHM_PATH: &str = "/dev/shm/cartesian_eye";

// --- MODELS ---
pub const MODEL_GOD: &str = "gemma-9b-it.gguf";
pub const MODEL_SIDEKICK: &str = "gemma-2b-it.gguf";
pub const MODEL_EMBEDDING: &str = "all-MiniLM-L6-v2.safetensors"; 

// --- TUNING ---
pub const TICK_RATE: Duration = Duration::from_millis(500);
pub const GOVERNOR_HYSTERESIS: Duration = Duration::from_secs(30);

// --- HEURISTICS ---
// Applications that trigger "Sidekick Mode" (Gaming)
pub const GAMES: &[&str] = &[
    "steam", 
    "lutris", 
    "heroic", 
    "wineserver", 
    "gamescope", 
    "yuzu", 
    "ryujinx",
    "dota2",
    "cs2",
    "factorio"
];

// Applications that trigger "Conscientious Mode" (High VRAM)
pub const CREATIVE_SUITE: &[&str] = &[
    "blender",
    "resolve", // DaVinci Resolve
    "obs",
    "gimp",
    "krita",
    "godot",
    "unity"
];

// Applications that trigger "God Mode" (High CPU/Dev)
pub const DEV_TOOLS: &[&str] = &[
    "code", // VS Code
    "zed",
    "nvim",
    "alacritty",
    "cargo"
];