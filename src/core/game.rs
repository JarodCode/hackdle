use macroquad::prelude::*;

use crate::core::player::Player;
use crate::core::wave::Wave;
use crate::ui::renderer;

// Tous les états possibles du jeu
pub enum GameState {
    MainMenu,
    InWave,
    BetweenWaves,
    Shop,
    GameOver,
}

pub struct Game {
    state: GameState,
    player: Player,
    wave: Option<Wave>,
    wave_number: u32,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            state: GameState::MainMenu,
            player: Player::new(),
            wave: None,
            wave_number: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::MainMenu => self.update_menu(dt),
            GameState::InWave => self.update_wave(dt),
            GameState::BetweenWaves => self.update_between_waves(dt),
            GameState::Shop => self.update_shop(dt),
            GameState::GameOver => self.update_game_over(dt),
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        match self.state {
            GameState::MainMenu => self.draw_menu(),
            GameState::InWave => self.draw_wave(),
            GameState::BetweenWaves => self.draw_between_waves(),
            GameState::Shop => self.draw_shop(),
            GameState::GameOver => self.draw_game_over(),
        }
    }

    // --- Update par état ---

    fn update_menu(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.start_wave();
        }
    }

    fn update_wave(&mut self, dt: f32) {
        self.player.update(dt);

        // Capture les touches pressées et les envoie à la vague
        if let Some(c) = get_char_pressed() {
            if let Some(wave) = &mut self.wave {
                wave.type_char(c);
            }
        }

        let player_pos = self.player.position;

        if let Some(wave) = &mut self.wave {
            wave.update(dt, player_pos);

            // Vérifie si un virus a atteint le joueur
            let mut damage_taken = 0u32;
            let mut kills = 0usize;
            wave.entries.retain(|e| {
                if e.virus.distance_to(player_pos) < 25.0 {
                    damage_taken += 20;
                    kills += 1;
                    false
                } else {
                    true
                }
            });
            for _ in 0..kills {
                wave.register_kill();
            }

            if damage_taken > 0 {
                self.player.take_damage(damage_taken);
            }

            // La mort prend la priorité sur la fin de vague
            if !self.player.is_alive() {
                self.state = GameState::GameOver;
            } else if wave.is_complete() {
                self.state = GameState::BetweenWaves;
            }
        }
    }

    fn update_between_waves(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.start_wave();
        }
    }

    fn update_shop(&mut self, _dt: f32) {}

    fn update_game_over(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            // Redémarre le jeu
            self.wave_number = 0;
            self.player = Player::new();
            self.state = GameState::MainMenu;
        }
    }

    // --- Draw par état ---

    fn draw_menu(&self) {
        let text = "HACKDLE — Appuie sur Entrée pour commencer";
        draw_text(text, 20.0, screen_height() / 2.0, 24.0, WHITE);
    }

    fn draw_wave(&self) {
        if let Some(wave) = &self.wave {
            wave.draw();
        }
        self.player.draw();
        renderer::draw_hud(&self.player, self.wave_number);
    }

    fn draw_between_waves(&self) {
        let text = format!("Vague {} terminée ! Appuie sur Entrée pour continuer", self.wave_number);
        draw_text(&text, 20.0, screen_height() / 2.0, 24.0, GREEN);
    }

    fn draw_shop(&self) {}

    fn draw_game_over(&self) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        draw_text("GAME OVER", cx - 80.0, cy - 20.0, 48.0, RED);
        let sub = format!("Vague {} atteinte — Entrée pour recommencer", self.wave_number);
        draw_text(&sub, cx - 160.0, cy + 30.0, 20.0, WHITE);
    }

    // --- Helpers ---

    fn start_wave(&mut self) {
        self.wave_number += 1;
        self.wave = Some(Wave::new(self.wave_number));
        self.state = GameState::InWave;
    }
}