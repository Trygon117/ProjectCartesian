mod lobotomy;
mod witness;
mod hippocampus;
mod inference;
mod audio;
mod ui; 
mod config; 

use iced::{Element, Subscription, Task, Theme, time};

use lobotomy::{SystemMonitor, AppCategory};
use witness::Eye;
use hippocampus::MemorySystem;
use inference::{Governor, GovernorState, Engine, EmbeddingEngine}; // Import EmbeddingEngine
use audio::Mixer;

pub fn main() -> iced::Result {
    iced::application("Cartesian Core", Cartesian::update, Cartesian::view)
        .subscription(Cartesian::subscription)
        .theme(Cartesian::theme)
        .run()
}

pub struct Cartesian {
    // SUBSYSTEMS
    pub monitor: SystemMonitor,
    pub eye: Eye,
    pub memory: MemorySystem,
    pub governor: Governor,
    pub engine: Engine,          // Chat (LLM)
    pub embedder: EmbeddingEngine, // Memory (BERT) - NEW
    pub mixer: Mixer,

    // STATE
    pub status: String,
    pub vision_status: String,
    pub brain_state: String,
    pub current_context: AppCategory,
    
    // FLAGS
    pub debug_override: bool, 
    
    // METRICS
    pub cpu_usage: f32,
    pub free_ram: f32,
    pub unknown_count: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleOverride,
}

impl Default for Cartesian {
    fn default() -> Self {
        // Init the Embedder (Lazy Load)
        let mut embedder = EmbeddingEngine::new();
        // In a real app, do this async so UI doesn't hang
        let _ = embedder.init(); 

        Self {
            monitor: SystemMonitor::new(),
            eye: Eye::new(),
            memory: MemorySystem::new(),
            governor: Governor::new(),
            engine: Engine::new(),
            embedder, // NEW
            mixer: Mixer::new(),
            
            status: "SYSTEM IDLE".to_string(),
            vision_status: "NO SIGNAL".to_string(),
            brain_state: "INITIALIZING...".to_string(),
            current_context: AppCategory::System,
            
            debug_override: false,
            
            cpu_usage: 0.0,
            free_ram: 0.0,
            unknown_count: 0,
        }
    }
}

impl Cartesian {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // 1. TELEMETRY
                let (cpu, ram) = self.monitor.get_vitals();
                self.cpu_usage = cpu;
                self.free_ram = ram;

                // 2. CONTEXT
                let (context, unknowns) = self.monitor.get_system_context();
                self.current_context = context;
                self.unknown_count = unknowns.len();
                
                self.status = match self.current_context {
                    AppCategory::Game => "CONTEXT: GAMING".to_string(),
                    AppCategory::Production => "CONTEXT: CREATIVE".to_string(),
                    AppCategory::Development => "CONTEXT: DEV".to_string(),
                    _ => "CONTEXT: GENERAL".to_string(),
                };

                if self.debug_override {
                    self.current_context = AppCategory::Game;
                    self.status = "SIMULATION: GAMING".to_string();
                }

                // 3. BRAIN
                let is_gaming = self.current_context == AppCategory::Game;
                let vram_pressure = self.current_context == AppCategory::Production;

                let state = self.governor.decide_state(self.free_ram, is_gaming, vram_pressure);
                self.engine.apply_state(&state);

                self.brain_state = format!(
                    "{} [{}]", 
                    match state {
                        GovernorState::GodMode => "GOD MODE",
                        GovernorState::Conscientious => "CONSCIENTIOUS",
                        GovernorState::SidekickMode => "SIDEKICK",
                        GovernorState::PotatoMode => "POTATO",
                    },
                    self.engine.current_model()
                );

                // 4. VISION
                match self.eye.observe() {
                    Some(frame) => self.vision_status = format!("INPUT [{}x{}]", frame.width, frame.height),
                    None => if self.eye.observe().is_none() { self.vision_status = "NO SIGNAL".to_string() }
                }
            }
            Message::ToggleOverride => {
                self.debug_override = !self.debug_override;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        ui::dashboard::view(self)
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(config::TICK_RATE).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}