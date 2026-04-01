use macroquad::prelude::*;
use macroquad::audio::{load_sound, Sound};

pub struct GameAssets {
    pub player: Texture2D,
    pub virus_fast: Texture2D,
    pub virus_classic: Texture2D,
    pub virus_heavy: Texture2D,
    pub virus_boss: Texture2D,
    pub font: Font,
    pub sound_laser: Sound,
    pub sound_error: Sound,
    pub sound_game_over: Sound,
}

impl GameAssets {
    pub async fn load() -> Self {
        Self {
            player: load_texture("assets/Player.png").await.unwrap(),
            virus_fast: load_texture("assets/virus_fast.png").await.unwrap(),
            virus_classic: load_texture("assets/virus_classic.png").await.unwrap(),
            virus_heavy: load_texture("assets/virus_heavy.png").await.unwrap(),
            virus_boss: load_texture("assets/virus_boss_1773656496889.png").await.unwrap(),
            font: load_ttf_font("assets/fonts/Hack.ttf").await.unwrap(),
            sound_laser: load_sound("assets/laser.wav").await.unwrap(),
            sound_error: load_sound("assets/error.wav").await.unwrap(),
            sound_game_over: load_sound("assets/game_over.wav").await.unwrap(),
        }
    }
}
