use std::cmp::Ordering;

use macroquad::prelude::*;

use crate::data::UserProfile;
use crate::ui::assets::GameAssets;
use crate::core::player::Player;
use crate::ui::vfx::VfxManager;
use crate::core::wave::Wave;
use crate::data::{SaveData, Storage};
use crate::ui::renderer;
use std::rc::Rc;

const DAMAGE_PER_HIT: u32 = 10;
const MAX_USERNAME_LEN: usize = 16;

// Tous les états possibles du jeu, selon le state actuelle on fait des actions différentes
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
    // Garde-fou pour n'enregistrer le score d'une run qu'une seule fois.
    run_recorded: bool,
    assets: Rc<GameAssets>,
    vfx: VfxManager,
    matrix_bg: crate::ui::matrix_bg::MatrixBackground,
}

impl Game {
    // Charge les ressources et restaure les profils au lancement.
    pub async fn new() -> Self {
        let assets = Rc::new(GameAssets::load().await);
        let save_data = Storage::load();
        let leaderboard = Self::build_leaderboard(&save_data.profiles);

        macroquad::audio::play_sound(
            &assets.bg_music,
            macroquad::audio::PlaySoundParams {
                looped: true, // Pour que ca boucle
                volume: 0.15,  // Volume bas pour qu'on entende bien les lasers et les erreurs
            },
        );

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
            vfx: VfxManager::new(),
            matrix_bg: crate::ui::matrix_bg::MatrixBackground::new(),
        }
    }

    // Tick principal: met à jour l'état actif puis les effets visuels.
    pub fn update(&mut self, dt: f32) {
        self.matrix_bg.update(dt);
        match self.state {
            GameState::Login => self.update_login(dt),
            GameState::MainMenu => self.update_menu(dt),
            GameState::InWave => self.update_wave(dt),
            GameState::BetweenWaves => self.update_between_waves(dt),
            GameState::Shop => self.update_shop(dt),
            GameState::GameOver => self.update_game_over(dt),
        }

        self.vfx.update(dt);
    }

    // Passe de rendu principal avec background, scène et VFX.
    pub fn draw(&self) {
        clear_background(BLACK);
        
        // Offset global appliqué à la scène pour matérialiser le screen shake.
        let shake = self.vfx.get_shake_offset();


        self.matrix_bg.draw(&self.assets);

        match self.state {
            GameState::Login => self.draw_login(),
            GameState::MainMenu => self.draw_menu(),
            GameState::InWave => self.draw_wave(shake),
            GameState::BetweenWaves => self.draw_between_waves(),
            GameState::Shop => self.draw_shop(),
            GameState::GameOver => self.draw_game_over(),
        }
        
        self.vfx.draw(shake);
    }

    // --- Update par état ---

    // Gère la saisie du pseudo utilisateur.
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
            std::process::exit(0);
        }

        if is_key_pressed(KeyCode::Enter) {
            self.handle_login_submit();
        }
    }

    // Gère les actions du menu principal (start, logout, quit).
    fn update_menu(&mut self, _dt: f32) {
        if self.current_user.is_some() && is_key_pressed(KeyCode::Enter) {
            self.reset_run_state();
            self.start_wave();
        }

        if is_key_pressed(KeyCode::L) {
            self.logout();
        }

        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }
    }

    // Boucle gameplay pendant une vague active.
    fn update_wave(&mut self, dt: f32) {
        self.player.update(dt);

        if is_key_pressed(KeyCode::Escape) {
            self.handle_player_defeat();
            return;
        }

        let player_pos = self.player.position;

        if let Some(c) = get_char_pressed() {
            if let Some(wave) = &mut self.wave {
                wave.type_char(c, &mut self.vfx, player_pos, &self.assets);
            }
        }
        let hits = 0u32;

        if let Some(wave) = &mut self.wave {
            wave.update(dt, player_pos);

            // Vérifie si un virus a atteint le joueur
            let mut damage_taken = 0u32;
            for entry in wave.entries.iter_mut() {
                let contact_radius = entry.virus.radius() + 75.0;
                if entry.virus.distance_to(player_pos) < contact_radius {
                    // Le rebond sert d'anti-collision continue pour éviter de vider
                    // toute la vie en restant collé au joueur sur plusieurs frames.
                    damage_taken += 20;
                    entry.virus.bounce_away(player_pos);
                }
            }

            if damage_taken > 0 {
                self.player.take_damage(damage_taken);
                self.vfx.trigger_shake(10.0, 0.3);

                macroquad::audio::play_sound(
                    &self.assets.sound_hit,
                    macroquad::audio::PlaySoundParams {
                        looped: false,
                        volume: 0.7,
                    },
                );
            }

            // La mort prend la priorité sur la fin de vague
            if !self.player.is_alive() {
                self.handle_player_defeat();
            } else if wave.is_complete() {
                self.state = GameState::BetweenWaves;
            }
        }

        if hits > 0 {
            let damage = DAMAGE_PER_HIT.saturating_mul(hits);
            self.player.take_damage(damage);
            self.vfx.trigger_shake(hits as f32 * 5.0 + 5.0, 0.3);
            if !self.player.is_alive() {
                self.handle_player_defeat();
            }   
        }
    }

    // Écran de transition entre deux vagues.
    fn update_between_waves(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.start_wave();
        }

        if is_key_pressed(KeyCode::Escape) {
            self.handle_player_defeat();
        }
    }

    // Placeholder boutique (non implémentée pour l'instant).
    fn update_shop(&mut self, _dt: f32) {}

    // Gère les entrées disponibles après défaite.
    fn update_game_over(&mut self, _dt: f32) {
        if is_key_pressed(KeyCode::Enter) {
            self.reset_run_state();
            self.state = GameState::MainMenu;
        }

        if is_key_pressed(KeyCode::L) {
            self.logout();
        }

        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }
    }

    // --- Draw par état ---

    // Écran de login + top leaderboard.
    fn draw_login(&self) {
        draw_text_ex(
            "HACKDLE / LOGIN",
            20.0,
            80.0,
            TextParams { font_size: 36, font: Some(&self.assets.font), color: WHITE, ..Default::default() },
        );
        draw_text_ex(
            "ENTER USERNAME [A-Z, 0-9, _, -]",
            20.0,
            120.0,
            TextParams { font_size: 20, font: Some(&self.assets.font), color: GRAY, ..Default::default() },
        );
        draw_text_ex(
            "<BACKSPACE> CLEAR_LAST | <ESC> CLEAR_ALL",
            20.0,
            150.0,
            TextParams { font_size: 18, font: Some(&self.assets.font), color: DARKGRAY, ..Default::default() },
        );

        draw_rectangle_lines(16.0, 170.0, 360.0, 52.0, 2.0, WHITE);
        let display = format!("> {}", self.login_input);
        draw_text_ex(&display, 28.0, 206.0, TextParams { font_size: 32, font: Some(&self.assets.font), color: YELLOW, ..Default::default() });

        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6, Some(&self.assets.font));
    }

    // Écran d'accueil avant lancement de run.
    fn draw_menu(&self) {
        let center_y = screen_height() / 2.0;
        let message = self
            .current_username()
            .map(|name| format!("AGENT [{}]: PRESS <ENTER> TO INITIATE DEFENSE", name))
            .unwrap_or_else(|| "UNAUTHORIZED: LOGIN REQUIRED".to_string());

        draw_text_ex(&message, 20.0, center_y, TextParams { font_size: 26, font: Some(&self.assets.font), color: WHITE, ..Default::default() });
        draw_text_ex(
            "PRESS <L> TO SWITCH AGENT",
            20.0,
            center_y + 32.0,
            TextParams { font_size: 20, font: Some(&self.assets.font), color: GRAY, ..Default::default() },
        );
        draw_text_ex(
            "PRESS <ESC> TO TERMINATE",
            20.0,
            center_y + 64.0,
            TextParams { font_size: 20, font: Some(&self.assets.font), color: DARKGRAY, ..Default::default() },
        );

        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6, Some(&self.assets.font));
    }

    // Rendu de la scène de combat et HUD.
    fn draw_wave(&self, offset: Vec2) {
        if let Some(wave) = &self.wave {
            wave.draw(&self.assets, offset);
        }
        self.player.draw(&self.assets, offset);
        renderer::draw_hud(&self.player, self.wave_number, Some(&self.assets.font));
    }

    // Écran de validation de fin de vague.
    fn draw_between_waves(&self) {
        let text = format!(
            "WAVE_{:03} CLEARED. PRESS <ENTER> TO PROCEED. <ESC> TO ABORT.",
            self.wave_number
        );
        draw_text_ex(&text, 20.0, screen_height() / 2.0, TextParams { font_size: 24, font: Some(&self.assets.font), color: GREEN, ..Default::default() });
        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6, Some(&self.assets.font));
    }

    // Placeholder boutique (non implémentée pour l'instant).
    fn draw_shop(&self) {}

    // Overlay de fin de partie avec actions rapides.
    fn draw_game_over(&self) {
        // 1. Assombrir tout l'écran avec un filtre semi-transparent
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.8));

        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        // 2. Gros titre rouge bien centré
        let title = "SYSTEM FAILURE";
        let title_dim = measure_text(title, Some(&self.assets.font), 64, 1.0);
        draw_text_ex(
            title,
            center_x - title_dim.width / 2.0,
            center_y - 100.0,
            TextParams { font_size: 64, font: Some(&self.assets.font), color: RED, ..Default::default() }
        );

        // 3. Stats de la partie
        let stats = format!("WAVES SURVIVED: {:03}", self.wave_number);
        let stats_dim = measure_text(&stats, Some(&self.assets.font), 32, 1.0);
        draw_text_ex(
            &stats,
            center_x - stats_dim.width / 2.0,
            center_y - 20.0,
            TextParams { font_size: 32, font: Some(&self.assets.font), color: WHITE, ..Default::default() }
        );

        // 4. Options d'action ("Boutons")
        let option1 = "> PRESS [ENTER] TO INITIALIZE NEW RUN <";
        let option2 = "PRESS [L] TO DISCONNECT AGENT";
        let option3 = "PRESS [ESC] TO QUIT SYSTEM";

        let op1_dim = measure_text(option1, Some(&self.assets.font), 24, 1.0);
        let op2_dim = measure_text(option2, Some(&self.assets.font), 20, 1.0);
        let op3_dim = measure_text(option3, Some(&self.assets.font), 20, 1.0);

        // Effet de clignotement fluide pour "Play Again"
        let alpha = (get_time() * 4.0).sin().abs() as f32; // Oscille entre 0.0 et 1.0
        let mut yellow_blink = YELLOW;
        yellow_blink.a = 0.4 + (alpha * 0.6); // L'opacité varie entre 0.4 et 1.0

        draw_text_ex(
            option1,
            center_x - op1_dim.width / 2.0,
            center_y + 80.0,
            TextParams { font_size: 24, font: Some(&self.assets.font), color: yellow_blink, ..Default::default() }
        );

        draw_text_ex(
            option2,
            center_x - op2_dim.width / 2.0,
            center_y + 130.0,
            TextParams { font_size: 20, font: Some(&self.assets.font), color: GRAY, ..Default::default() }
        );

        draw_text_ex(
            option3,
            center_x - op3_dim.width / 2.0,
            center_y + 160.0,
            TextParams { font_size: 20, font: Some(&self.assets.font), color: RED, ..Default::default() }
        );

        renderer::draw_scoreboard(&self.leaderboard, "TOP AGENTS", 6, Some(&self.assets.font));
    }

    // --- Helpers ---

    // Démarre une nouvelle vague si un profil est actif.
    fn start_wave(&mut self) {
        if self.current_user.is_none() {
            return;
        }

        self.wave_number += 1;
        self.wave = Some(Wave::new(self.wave_number));
        self.run_recorded = false;
        self.state = GameState::InWave;
    }

    // Valide/crée le profil puis bascule vers le menu principal.
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

    // Recalcule le classement trié depuis les profils sauvegardés.
    fn refresh_leaderboard(&mut self) {
        self.leaderboard = Self::build_leaderboard(&self.save_data.profiles);
    }

    // Persiste l'état global des profils sur disque.
    fn persist_save(&self) {
        if let Err(err) = Storage::save(&self.save_data) {
            eprintln!("Impossible d'enregistrer les profils: {:?}", err);
        }
    }

    // Enregistre la performance de la run courante une seule fois.
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

    // Renvoie le pseudo du profil actif.
    fn current_username(&self) -> Option<&str> {
        self.current_user
            .and_then(|idx| self.save_data.profiles.get(idx))
            .map(|profile| profile.username.as_str())
    }

    // Réinitialise tous les éléments liés à une run.
    fn reset_run_state(&mut self) {
        self.wave_number = 0;
        self.wave = None;
        self.player = Player::new();
        self.run_recorded = true;
    }

    // Centralise les effets de défaite et la transition d'état.
    fn handle_player_defeat(&mut self) {
        macroquad::audio::play_sound(
            &self.assets.sound_game_over,
            macroquad::audio::PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );

        self.wave = None;
        self.record_current_run();
        self.state = GameState::GameOver;
    }

    // Déconnecte le profil courant et revient à l'écran de login.
    fn logout(&mut self) {
        self.current_user = None;
        self.login_input.clear();
        self.reset_run_state();
        while let Some(_) = get_char_pressed() {}
        self.state = GameState::Login;
    }

    // Construit le classement affiché à partir des profils.
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

    // Contrôle les caractères autorisés dans le pseudo.
    fn is_allowed_username_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }
}
