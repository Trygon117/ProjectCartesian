mod lobotomy;
mod witness;
mod hippocampus;
mod inference;
mod audio;
mod ui; 
mod config; 
mod schema; 

use iced::{Element, Subscription, Task, Theme, time};
use lobotomy::{SystemMonitor, AppCategory};
use witness::Eye;
use hippocampus::MemorySystem;
use inference::{Governor, GovernorState, Engine, EmbeddingEngine};
use audio::Mixer;
use ui::chat::ChatMessage;

pub fn main() -> iced::Result {
    // FIXED: Iced 0.14 application builder pattern
    // 1. Pass the 'init' function (boot)
    // 2. Pass update
    // 3. Pass view
    iced::application(Cartesian::init, Cartesian::update, Cartesian::view)
        .subscription(Cartesian::subscription)
        .theme(Cartesian::theme)
        .window(iced::window::Settings {
            // Title is set here now
            // But iced::application usually infers title from the window settings
            ..Default::default()
        })
        .run()
}

// ... Cartesian Struct (Unchanged) ...
pub struct Cartesian {
    pub monitor: SystemMonitor,
    pub eye: Eye,
    pub memory: MemorySystem,
    pub governor: Governor,
    pub engine: Engine,
    pub embedder: EmbeddingEngine,
    pub mixer: Mixer,
    pub chat_history: Vec<ChatMessage>,
    pub input_value: String,
    pub status: String,
    pub vision_status: String,
    pub brain_state: String,
    pub current_context: AppCategory,
    pub debug_override: bool, 
    pub cpu_usage: f32,
    pub free_ram: f32,
    pub unknown_count: usize,
}

// ... Message Enum (Unchanged) ...
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleOverride,
    InputChanged(String),
    SendChat,
}

impl Cartesian {
    // FIXED: Changed from Default::default() to a proper init function returning (Self, Task)
    // This matches the BootFn signature required by iced::application
    fn init() -> (Self, Task<Message>) {
        let mut embedder = EmbeddingEngine::new();
        let _ = embedder.init(); 

        (Self {
            monitor: SystemMonitor::new(),
            eye: Eye::new(),
            memory: MemorySystem::new(),
            governor: Governor::new(),
            engine: Engine::new(),
            embedder,
            mixer: Mixer::new(),
            
            chat_history: vec![
                ChatMessage { 
                    sender: "CARTESIAN".to_string(), 
                    content: "System Online. Awaiting input.".to_string(),
                    timestamp: "00:00".to_string() 
                }
            ],
            input_value: String::new(),

            status: "SYSTEM IDLE".to_string(),
            vision_status: "NO SIGNAL".to_string(),
            brain_state: "INITIALIZING...".to_string(),
            current_context: AppCategory::System,
            debug_override: false,
            cpu_usage: 0.0,
            free_ram: 0.0,
            unknown_count: 0,
        }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // ... (Update Logic Unchanged) ...
        // Note: For brevity, I'm not repeating the huge update block unless requested.
        // It remains exactly as it was, just ensure the method signature matches.
        match message {
            Message::InputChanged(val) => {
                self.input_value = val;
            }
            Message::SendChat => {
                if self.input_value.trim().is_empty() { return Task::none(); }
                let user_msg = self.input_value.clone();
                self.chat_history.push(ChatMessage {
                    sender: "USER".to_string(),
                    content: user_msg.clone(),
                    timestamp: "Now".to_string(),
                });
                self.input_value.clear();
                
                let response_text = if self.engine.current_model().contains("MISSING") {
                    "Error: Neural weights not found in /usr/share/cartesian/models/".to_string()
                } else {
                    format!("I received: '{}'. (Inference Engine Stub)", user_msg)
                };

                self.chat_history.push(ChatMessage {
                    sender: "CARTESIAN".to_string(),
                    content: response_text,
                    timestamp: "Now".to_string(),
                });
            }
            Message::Tick => {
                let (cpu, ram) = self.monitor.get_vitals();
                self.cpu_usage = cpu;
                self.free_ram = ram;

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