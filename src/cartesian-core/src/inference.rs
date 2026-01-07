use std::path::Path;
use std::time::Instant;
// REMOVED: unused imports (Arc, Mutex)
use anyhow::{Result, Error as E}; // REMOVED: unused Context

// CANDLE IMPORTS
// REMOVED: unused DType
use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file; // ADDED: Required for GGUF parsing
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama as model;
use model::ModelWeights;
use tokenizers::Tokenizer;

use crate::config;
use crate::schema::ActionSchema;
use image::DynamicImage;

#[derive(Debug, Clone, PartialEq)]
pub enum GovernorState {
    GodMode,
    Conscientious,
    SidekickMode,
    PotatoMode,
}

pub struct Governor {
    current_state: GovernorState,
    last_state_change: Instant,
}

impl Governor {
    pub fn new() -> Self {
        Self {
            current_state: GovernorState::GodMode,
            last_state_change: Instant::now(),
        }
    }

    pub fn decide_state(&mut self, free_ram_gb: f32, is_gaming: bool, vram_pressure: bool) -> GovernorState {
        if free_ram_gb < 2.0 { return self.transition_checked(GovernorState::PotatoMode); }
        if is_gaming { return self.transition_checked(GovernorState::SidekickMode); }
        if vram_pressure { return self.transition_checked(GovernorState::Conscientious); }
        self.transition_checked(GovernorState::GodMode)
    }

    fn transition_checked(&mut self, target: GovernorState) -> GovernorState {
        if target == self.current_state { return self.current_state.clone(); }
        if self.last_state_change.elapsed() > config::GOVERNOR_HYSTERESIS {
            return self.commit_transition(target);
        }
        self.current_state.clone()
    }

    fn commit_transition(&mut self, new_state: GovernorState) -> GovernorState {
        println!("Governor: Transition {:?} -> {:?}", self.current_state, new_state);
        self.current_state = new_state;
        self.last_state_change = Instant::now();
        self.current_state.clone()
    }
}

// --- THE ENGINE (Candle) ---

pub struct Engine {
    model: Option<ModelWeights>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    current_model_name: String,
    // Cache for GGUF handling
    logits_processor: LogitsProcessor,
}

impl Engine {
    pub fn new() -> Self {
        // Auto-detect CUDA. If fail, fall back to CPU.
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        println!("Engine: Initialized on Device: {:?}", device);

        Self {
            model: None,
            tokenizer: None,
            device,
            current_model_name: String::new(),
            logits_processor: LogitsProcessor::new(42, Some(0.9), Some(1.1)), // Seed, Temp, Top-P
        }
    }

    pub fn apply_state(&mut self, state: &GovernorState) -> bool {
        let target_model = match state {
            GovernorState::GodMode => config::MODEL_GOD,
            GovernorState::Conscientious => config::MODEL_SIDEKICK,
            GovernorState::SidekickMode => config::MODEL_SIDEKICK,
            GovernorState::PotatoMode => {
                self.unload();
                "None"
            }
        };

        if self.current_model_name == target_model {
            return false;
        }

        if target_model != "None" {
            if let Err(e) = self.load_model(target_model) {
                eprintln!("Engine Error: Failed to load {}: {}", target_model, e);
                return false;
            }
        }
        true
    }

    fn unload(&mut self) {
        self.model = None;
        self.tokenizer = None;
        self.current_model_name = "None".to_string();
        println!("Engine: Brain unloaded.");
    }

    fn load_model(&mut self, model_name: &str) -> Result<()> {
        let base_dir = config::get_model_dir();
        let model_path = Path::new(&base_dir).join(model_name);
        
        let tokenizer_path = Path::new(&base_dir).join("tokenizer.json");

        println!("Engine: Loading GGUF from {:?}...", model_path);
        
        // 1. Load GGUF Content (Header Parsing)
        let mut file = std::fs::File::open(&model_path)?;
        // FIX: Parse content first using the candle_core::quantized::gguf_file module
        let content = gguf_file::Content::read(&mut file)?;
        
        // 2. Load Weights using the content
        // FIX: Pass content as the first argument
        let model = model::ModelWeights::from_gguf(content, &mut file, &self.device)?;
        
        // 3. Load Tokenizer
        if !tokenizer_path.exists() {
            return Err(anyhow::anyhow!("Tokenizer not found at {:?}", tokenizer_path));
        }
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(E::msg)?;

        self.model = Some(model);
        self.tokenizer = Some(tokenizer);
        self.current_model_name = model_name.to_string();
        
        println!("Engine: Brain Loaded Successfully.");
        Ok(())
    }
    
    pub fn infer_action(&mut self, prompt: &str, _image: Option<&DynamicImage>) -> Option<ActionSchema> {
        let model = self.model.as_mut()?;
        let tokenizer = self.tokenizer.as_ref()?;

        // --- PRE-PROCESSING ---
        let formatted_prompt = format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt);
        
        let tokens = tokenizer.encode(formatted_prompt, true).ok()?;
        let prompt_tokens = tokens.get_ids();
        
        // --- INFERENCE LOOP ---
        let input = Tensor::new(prompt_tokens, &self.device).ok()?.unsqueeze(0).ok()?;
        let logits = model.forward(&input, 0).ok()?;
        let _logits = logits.squeeze(0).ok()?; // FIX: Renamed to _logits to silence unused warning

        // Sample logic stub (Commented out in original, kept commented)
        // let next_token = self.logits_processor.sample(&logits).ok()?;
        
        // --- MOCK RESPONSE (For Stability Testing) ---
        Some(ActionSchema {
            chain_of_thought: "Candle Inference Logic Active.".to_string(),
            needs_information: None,
            user_message: format!("(Candle Engine) You said: '{}'", prompt),
            tool_calls: vec![],
            status: crate::schema::TaskStatus::Active,
        })
    }
    
    pub fn current_model(&self) -> String {
        self.current_model_name.clone()
    }
}

// Simple Embedding Engine Wrapper
pub struct EmbeddingEngine {
    ready: bool,
}
impl EmbeddingEngine {
    pub fn new() -> Self { Self { ready: false } }
    pub fn init(&mut self) -> Result<(), String> { 
        self.ready = true; 
        Ok(()) 
    }
    pub fn embed(&self, _text: &str) -> Vec<f32> {
        vec![0.0; 384] 
    }
}