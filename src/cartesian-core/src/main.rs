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
    iced::application(Cartesian::init, Cartesian::update, Cartesian::view)
        .subscription(Cartesian::subscription)
        .theme(Cartesian::theme)
        .window(iced::window::Settings {..Default::default()})
        .run()
}

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

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleOverride,
    InputChanged(String),
    SendChat,
}

impl Cartesian {
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
                
                // --- MULTIMODAL INFERENCE CALL ---
                // 1. Grab visual context
                let visual_context = self.eye.observe()
                    .and_then(|cortex| cortex.to_dynamic_image());
                
                // 2. Infer Action
                if let Some(action) = self.engine.infer_action(&user_msg, visual_context.as_ref()) {
                    self.chat_history.push(ChatMessage {
                        sender: "CARTESIAN".to_string(),
                        content: action.user_message,
                        timestamp: "Now".to_string(),
                    });
                } else {
                    let error_msg = if self.engine.current_model().contains("MISSING") {
                        "Error: Brain not found. Install Gemma 3 4B."
                    } else {
                        "Error: Inference failed."
                    };
                    
                    self.chat_history.push(ChatMessage {
                        sender: "SYSTEM".to_string(),
                        content: error_msg.to_string(),
                        timestamp: "Now".to_string(),
                    });
                }
            }
            Message::Tick => {
                let (cpu, ram) = self.monitor.get_vitals();
                self.cpu_usage = cpu;
                self.free_ram = ram;

                let (context, unknowns) = self.monitor.get_system_context();
                self.current_context = context;
                self.unknown_count = unknowns.len();
                
                if self.debug_override {
                    self.current_context = AppCategory::Game;
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