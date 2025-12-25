/// THE AUDIO CONSOLE
/// Manages Virtual Sinks (Game, Voice, Music) via PipeWire.

#[derive(Debug, Clone)]
pub struct AudioState {
    pub game_vol: f32,
    pub voice_vol: f32,
    pub music_vol: f32,
    pub mic_muted: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            game_vol: 0.8,
            voice_vol: 1.0,
            music_vol: 0.5,
            mic_muted: false,
        }
    }
}

pub struct Mixer {
    state: AudioState,
    // pipewire_context: Option<Context>, // TODO: Phase 6
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            state: AudioState::default(),
        }
    }

    pub fn get_state(&self) -> AudioState {
        self.state.clone()
    }

    pub fn set_volume(&mut self, channel: &str, level: f32) {
        match channel {
            "GAME" => self.state.game_vol = level,
            "VOICE" => self.state.voice_vol = level,
            "MUSIC" => self.state.music_vol = level,
            _ => {}
        }
        // TODO: Send IPC command to PipeWire here
    }

    pub fn toggle_mic(&mut self) {
        self.state.mic_muted = !self.state.mic_muted;
    }
}