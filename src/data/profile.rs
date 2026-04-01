use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub best_wave: u32,
    pub games_played: u32,
    pub total_waves: u64,
}

impl UserProfile {
    pub fn new(username: String) -> Self {
        Self {
            username,
            best_wave: 0,
            games_played: 0,
            total_waves: 0,
        }
    }

    pub fn register_run(&mut self, wave_reached: u32) {
        self.games_played += 1;
        self.total_waves += wave_reached as u64;
        self.best_wave = self.best_wave.max(wave_reached);
    }

    pub fn average_wave(&self) -> f32 {
        if self.games_played == 0 {
            0.0
        } else {
            self.total_waves as f32 / self.games_played as f32
        }
    }
}
