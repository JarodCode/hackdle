use macroquad::prelude::*;
use macroquad::audio::{load_sound, Sound};

pub struct GameAssets {
    pub player: Texture2D,
    pub virus_fast: Texture2D,
    pub virus_classic: Texture2D,
    pub virus_heavy: Texture2D,
    pub boss_python: Texture2D,
    pub boss_rust: Texture2D,
    pub boss_c: Texture2D,
    pub font: Font,
    pub sound_laser: Sound,
    pub sound_error: Sound,
    pub sound_game_over: Sound,
    pub sound_hit: Sound,
    pub bg_music: Sound,
}

impl GameAssets {
    // Charge toutes les ressources statiques utilisées par la partie.
    pub async fn load() -> Self {
        Self {
            player: load_texture("assets/Player.png").await.unwrap(),
            virus_fast: load_texture("assets/virus_fast.png").await.unwrap(),
            virus_classic: load_texture("assets/virus_classic.png").await.unwrap(),
            virus_heavy: load_texture("assets/virus_heavy.png").await.unwrap(),
            boss_python: load_texture("assets/boss_python.png").await.unwrap(),
            boss_rust: load_texture("assets/boss_rust.png").await.unwrap(),
            boss_c: load_texture("assets/boss_c.png").await.unwrap(),
            font: load_ttf_font("assets/fonts/Hack.ttf").await.unwrap(),
            sound_laser: load_sound("assets/laser.wav").await.unwrap(),
            sound_error: load_sound("assets/error.wav").await.unwrap(),
            sound_game_over: load_sound("assets/game_over.wav").await.unwrap(),
            sound_hit: load_sound("assets/hit.wav").await.unwrap(),
            bg_music: load_sound("assets/bg_music.ogg").await.unwrap(),
        }
    }
}
