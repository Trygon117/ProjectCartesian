use std::path::Path;
use std::time::Instant;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use crate::config;
use crate::schema::ActionSchema;

// CANDLE IMPORTS
use candle_core::Device;
use candle_transformers::models::bert::BertModel; 
use tokenizers::Tokenizer;

// --- THE GOVERNOR (State Machine) ---

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
        if free_ram_gb < 2.0 {
            return self.transition_checked(GovernorState::PotatoMode);
        }
        if is_gaming {
            return self.transition_checked(GovernorState::SidekickMode);
        }
        if vram_pressure {
            return self.transition_checked(GovernorState::Conscientious);
        }
        self.transition_checked(GovernorState::GodMode)
    }

    fn transition_checked(&mut self, target: GovernorState) -> GovernorState {
        if target == self.current_state {
            return self.current_state.clone();
        }
        
        let current_rank = self.rank(&self.current_state);
        let target_rank = self.rank(&target);

        if target_rank > current_rank {
            return self.commit_transition(target);
        }

        if self.last_state_change.elapsed() > config::GOVERNOR_HYSTERESIS {
            return self.commit_transition(target);
        }

        self.current_state.clone()
    }

    fn commit_transition(&mut self, new_state: GovernorState) -> GovernorState {
        println!("Governor: Transition {:?} -> {:?}", self.current_state, new_state);
        self.current_state = new_state.clone();
        self.last_state_change = Instant::now();
        new_state
    }

    fn rank(&self, state: &GovernorState) -> u8 {
        match state {
            GovernorState::GodMode => 0,
            GovernorState::Conscientious => 1,
            GovernorState::SidekickMode => 2,
            GovernorState::PotatoMode => 3,
        }
    }
}

// --- THE LLM ENGINE ---

pub struct Engine {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    current_model_name: String,
}

impl Engine {
    pub fn new() -> Self {
        let backend = LlamaBackend::init().unwrap();
        Self {
            backend,
            model: None,
            current_model_name: String::new(),
        }
    }

    pub fn apply_state(&mut self, state: &GovernorState) -> bool {
        let (target_model, use_gpu) = match state {
            GovernorState::GodMode => (config::MODEL_GOD, true),
            GovernorState::Conscientious => (config::MODEL_SIDEKICK, false),
            GovernorState::SidekickMode => (config::MODEL_SIDEKICK, false),
            GovernorState::PotatoMode => {
                self.model = None;
                self.current_model_name = "None".to_string();
                return true; 
            }
        };

        if self.current_model_name == target_model {
            return false;
        }

        println!("Engine: Swapping to model {} (GPU: {})", target_model, use_gpu);
        self.load_model(target_model, use_gpu);
        true
    }

    fn load_model(&mut self, model_name: &str, use_gpu: bool) {
        let base_dir = config::get_model_dir();
        let full_path = format!("{}{}", base_dir, model_name);
        let path = Path::new(&full_path);

        if !path.exists() {
            eprintln!("Engine Error: Model file not found at {:?}", path);
            self.model = None;
            self.current_model_name = "ERROR_MISSING_FILE".to_string();
            return;
        }

        let mut params = LlamaModelParams::default();
        if use_gpu {
            params = params.with_n_gpu_layers(99); 
        } else {
            params = params.with_n_gpu_layers(0);
        }

        match LlamaModel::load_from_file(&self.backend, path, &params) {
            Ok(model) => {
                self.model = Some(model);
                self.current_model_name = model_name.to_string();
                println!("Engine: Model loaded successfully.");
            },
            Err(e) => {
                eprintln!("Engine Error: Failed to load model: {}", e);
                self.model = None;
                self.current_model_name = "ERROR_LOAD_FAILED".to_string();
            }
        }
    }
    
    pub fn infer_action(&self, _prompt: &str) -> Option<ActionSchema> {
        if self.model.is_none() {
            return None;
        }
        println!("Engine: Inference requested (Stub)");
        None
    }
    
    pub fn current_model(&self) -> String {
        self.current_model_name.clone()
    }
}

// --- THE EMBEDDING ENGINE ---

pub struct EmbeddingEngine {
    _model: Option<BertModel>,
    _tokenizer: Option<Tokenizer>,
    ready: bool,
}

impl EmbeddingEngine {
    pub fn new() -> Self {
        Self {
            _model: None,
            _tokenizer: None,
            ready: false,
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        let base_dir = config::get_model_dir();
        let model_path = format!("{}{}", base_dir, config::MODEL_EMBEDDING);
        
        let _device = Device::Cpu; 
        self.ready = true;
        println!("EmbeddingEngine: Loaded (Stubbed for {:?})", model_path);
        Ok(())
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
        if !self.ready {
            return vec![0.0; 384];
        }
        let seed = text.len() as f32;
        let mut vec = Vec::with_capacity(384);
        for i in 0..384 {
            vec.push((i as f32 * seed).sin()); 
        }
        vec
    }
}