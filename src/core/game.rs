use std::cmp::Ordering;

use macroquad::prelude::*;

use crate::accounts::UserProfile;
use crate::core::assets::GameAssets;
use crate::core::player::Player;
use crate::core::wave::Wave;
use crate::data::{SaveData, Storage};
use crate::ui::renderer;
use std::rc::Rc;

const DAMAGE_PER_HIT: u32 = 10;
const MAX_USERNAME_LEN: usize = 16;

// Tous les états possibles du jeu
pub enum GameState {
    Login,
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
    save_data: SaveData,
    leaderboard: Vec<UserProfile>,
    current_user: Option<usize>,
    login_input: String,
    run_recorded: bool,
    assets: Rc<GameAssets>,
}

impl Game {
    pub async fn new() -> Self {
        let assets = Rc::new(GameAssets::load().await);
        let save_data = Storage::load();
        let leaderboard = Self::build_leaderboard(&save_data.profiles);

        Self {
            state: GameState::Login,
            player: Player::new(),
            wave: None,
            wave_number: 0,
            save_data,
            leaderboard,
            current_user: None,
            login_input: String::new(),
            run_recorded: true,
            assets,
        }
    }

    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::Login => self.update_login(dt),
            GameState::MainMenu => self.update_menu(dt),
            GameState::InWave => self.update_wave(dt),
            GameState::BetweenWaves => self.update_between_waves(dt),
            GameState::Shop => self.update_shop(dt),
            GameState::GameOver => self.update_game_over(dt),
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);
        
        // Draw the background texture scaled to the screen size
        draw_texture_ex(
            &self.assets.background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        match self.state {
            GameState::Login => self.draw_login(),
            GameState::MainMenu => self.draw_menu(),
            GameState::InWave => self.draw_wave(),
            GameState::BetweenWaves => self.draw_between_waves(),
            GameState::Shop => self.draw_shop(),
            GameState::GameOver => self.draw_game_over(),
        }
    }

    // --- Update par état ---

    fn update_login(&mut self, _dt: f32) {
        while let Some(c) = get_char_pressed() {
            if Self::is_allowed_username_char(c) && self.login_input.len() < MAX_USERNAME_LEN {
                self.login_input.push(c);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.login_input.pop();
        }

        if is_key_pressed(KeyCode::Escape) {
            self.login_input.clear();
        }

        if is_key_pressed(KeyCode::Enter) {
            self.handle_login_submit();
        }
    }

    fn update_menu(&mut self, _dt: f32) {
        if self.current_user.is_some() && is_key_pressed(KeyCode::Enter) {
            self.reset_run_state();
            self.start_wave();
        }

        if is_key_pressed(KeyCode::L) {
            self.logout();
        }
    }

    fn update_wave(&mut self, dt: f32) {
        self.player.update(dt);

        if is_key_pressed(KeyCode::Escape) {
            self.handle_player_defeat();
            return;
        }

        if let Some(c) = get_char_pressed() {
            if let Some(wave) = &mut self.wave {
                wave.type_char(c);
            }
        }

        let player_pos = self.player.position;
        let mut hits = 0u32;

        if let Some(wave) = &mut self.wave {
            wave.update(dt, player_pos);

            wave.entries.retain(|entry| {
                let hit = entry.virus.distance_to(player_pos) < 25.0;
                if hit {
                    hits += 1;
                }
                !hit
            });

            if wave.is_complete() {
                self.state = GameState::BetweenWaves;
            }
        }

        if hits > 0 {
            let damage = DAMAGE_PER_HIT.saturating_mul(hits);
            self.player.take_damage(damage);
            if !self.player.is_alive() {
                self.handle_player_defeat();
            }
        }
    }

    fn update_between_waves(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.start_wave();
        }

        if is_key_pressed(KeyCode::Escape) {
            self.handle_player_defeat();
        }
    }

    fn update_shop(&mut self, _dt: f32) {}

    fn update_game_over(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.reset_run_state();
            self.state = GameState::MainMenu;
        }

        if is_key_pressed(KeyCode::L) {
            self.logout();
        }
    }

    // --- Draw par état ---

    fn draw_login(&self) {
        draw_text(
            "HACKDLE — Connecte-toi",
            20.0,
            80.0,
            36.0,
            WHITE,
        );
        draw_text(
            "Tape un identifiant (A-Z, 0-9, _ ou -) puis Entrée",
            20.0,
            120.0,
            20.0,
            GRAY,
        );
        draw_text(
            "Backspace pour effacer, Échap pour vider",
            20.0,
            150.0,
            18.0,
            DARKGRAY,
        );

        draw_rectangle_lines(16.0, 170.0, 360.0, 52.0, 2.0, WHITE);
        let display = format!("> {}", self.login_input);
        draw_text(&display, 28.0, 206.0, 32.0, YELLOW);

        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6);
    }

    fn draw_menu(&self) {
        let center_y = screen_height() / 2.0;
        let message = self
            .current_username()
            .map(|name| format!("Agent {name}, appuie sur Entrée pour lancer la défense"))
            .unwrap_or_else(|| "Connecte-toi pour jouer".to_string());

        draw_text(&message, 20.0, center_y, 26.0, WHITE);
        draw_text(
            "L pour changer d'agent",
            20.0,
            center_y + 32.0,
            20.0,
            GRAY,
        );

        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6);
    }

    fn draw_wave(&self) {
        if let Some(wave) = &self.wave {
            wave.draw(&self.assets);
        }
        self.player.draw(&self.assets);
        renderer::draw_hud(&self.player, self.wave_number);
    }

    fn draw_between_waves(&self) {
        let text = format!(
            "Vague {} terminée ! Entrée pour la suivante (Échap pour abandonner)",
            self.wave_number
        );
        draw_text(&text, 20.0, screen_height() / 2.0, 24.0, GREEN);
        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6);
    }

    fn draw_shop(&self) {}

    fn draw_game_over(&self) {
        let message = format!("GAME OVER — vague atteinte {}", self.wave_number);
        draw_text(&message, 20.0, screen_height() / 2.0, 28.0, RED);
        draw_text(
            "Entrée pour retenter, L pour changer d'agent",
            20.0,
            screen_height() / 2.0 + 36.0,
            20.0,
            GRAY,
        );
        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6);
    }

    // --- Helpers ---

    fn start_wave(&mut self) {
        if self.current_user.is_none() {
            return;
        }

        self.wave_number += 1;
        self.wave = Some(Wave::new(self.wave_number));
        self.run_recorded = false;
        self.state = GameState::InWave;
    }

    fn handle_login_submit(&mut self) {
        let trimmed = self.login_input.trim();
        if trimmed.is_empty() {
            return;
        }

        let mut username = trimmed.to_string();
        if username.len() > MAX_USERNAME_LEN {
            username.truncate(MAX_USERNAME_LEN);
        }

        let existing_index = self
            .save_data
            .profiles
            .iter()
            .position(|profile| profile.username.eq_ignore_ascii_case(&username));

        let index = match existing_index {
            Some(idx) => idx,
            None => {
                self.save_data.profiles.push(UserProfile::new(username.clone()));
                self.persist_save();
                self.save_data.profiles.len() - 1
            }
        };

        self.current_user = Some(index);
        self.login_input.clear();
        self.refresh_leaderboard();
        self.reset_run_state();
        self.state = GameState::MainMenu;
    }

    fn refresh_leaderboard(&mut self) {
        self.leaderboard = Self::build_leaderboard(&self.save_data.profiles);
    }

    fn persist_save(&self) {
        if let Err(err) = Storage::save(&self.save_data) {
            eprintln!("Impossible d'enregistrer les profils: {:?}", err);
        }
    }

    fn record_current_run(&mut self) {
        if self.run_recorded {
            return;
        }

        if let Some(idx) = self.current_user {
            if let Some(profile) = self.save_data.profiles.get_mut(idx) {
                profile.register_run(self.wave_number);
                self.persist_save();
                self.refresh_leaderboard();
            }
        }

        self.run_recorded = true;
    }

    fn current_username(&self) -> Option<&str> {
        self.current_user
            .and_then(|idx| self.save_data.profiles.get(idx))
            .map(|profile| profile.username.as_str())
    }

    fn reset_run_state(&mut self) {
        self.wave_number = 0;
        self.wave = None;
        self.player = Player::new();
        self.run_recorded = true;
    }

    fn handle_player_defeat(&mut self) {
        self.wave = None;
        self.record_current_run();
        self.state = GameState::GameOver;
    }

    fn logout(&mut self) {
        self.current_user = None;
        self.login_input.clear();
        self.reset_run_state();
        self.state = GameState::Login;
    }

    fn build_leaderboard(source: &[UserProfile]) -> Vec<UserProfile> {
        let mut entries = source.to_vec();
        entries.sort_by(|a, b| {
            b.best_wave
                .cmp(&a.best_wave)
                .then_with(|| {
                    b.average_wave()
                        .partial_cmp(&a.average_wave())
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| a.username.cmp(&b.username))
        });
        entries
    }

    fn is_allowed_username_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }
}
