use std::time::Duration;

/// GLOBAL CONFIGURATION
/// The Single Source of Truth for Paths, Constants, and Tuning.

// --- PATHS (Cross-Platform) ---

pub fn get_model_dir() -> String {
    if cfg!(target_os = "windows") {
        ".\\models\\".to_string()
    } else {
        "/usr/share/cartesian/models/".to_string()
    }
}

pub fn get_shm_path() -> String {
    if cfg!(target_os = "windows") {
        std::env::temp_dir().join("cartesian_eye").to_string_lossy().to_string()
    } else {
        "/dev/shm/cartesian_eye".to_string()
    }
}

pub const REGISTRY_PATH: &str = "process_registry.json";

// --- MODELS ---

// DEFAULT: Gemma 3 4B (Optimized for 8GB-12GB GPUs)
pub const MODEL_GOD: &str = "gemma-3-4b-it-q4_0.gguf";
pub const MODEL_SIDEKICK: &str = "gemma-3-4b-it-q4_0.gguf";

// The Vision Projector (The "Eyes")
pub const MODEL_PROJECTOR: &str = "mmproj-model-f16-4B.gguf";

// Embedding model for memory
pub const MODEL_EMBEDDING: &str = "all-MiniLM-L6-v2.safetensors"; 

// --- TUNING ---
pub const TICK_RATE: Duration = Duration::from_millis(500);
pub const GOVERNOR_HYSTERESIS: Duration = Duration::from_secs(30);

// --- HEURISTICS ---
pub const GAMES: &[&str] = &[
    "steam", "lutris", "heroic", "wineserver", "gamescope", 
    "yuzu", "ryujinx", "dota2", "cs2", "factorio"
];

pub const CREATIVE_SUITE: &[&str] = &[
    "blender", "resolve", "obs", "gimp", "krita", "godot", "unity"
];

pub const DEV_TOOLS: &[&str] = &[
    "code", "zed", "nvim", "alacritty", "cargo", "powershell", "cmd"
];