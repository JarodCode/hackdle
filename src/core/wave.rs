use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use ::rand::Rng;
use ::rand::thread_rng;

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::core::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState,
    // true si ce virus est en cours de frappe
    pub active: bool,
    pub glitch_timer: f32,
}

pub struct Wave {
    pub number: u32,
    pub entries: Vec<VirusEntry>, // liste de tous les virus à l'écran
    to_kill: usize, // nombre de virus à tuer pour terminer la vague
    killed: usize, // nombre de virus tués jusqu'ici
    spawned: usize, // nombre de virus spawnés jusqu'ici
    elapsed: f32, // temps écoulé
    next_spawn_in: f32,
}

impl Wave {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            entries: Vec::new(),
            to_kill: Self::kills_required(number),
            killed: 0,
            spawned: 0,
            elapsed: 0.0,
            next_spawn_in: 0.5,
        }
    }

    // Nombre de virus à tuer par vague
    fn kills_required(wave_number: u32) -> usize {
        match wave_number {
            1 => 5,
            2 => 8,
            3 => 12,
            _ => 12 + (wave_number as usize - 3) * 4,
        }
    }

    // délais entre l'apparition de deux Virus
    fn spawn_delay(&self) -> f32 {
        let remaining = self.to_kill.saturating_sub(self.spawned); // .sat_sub : soustraction qui s'arrête à 0
        if remaining == 0 {
            return f32::MAX;
        }

        return (-(self.elapsed.ln() / 6.0 - 1.0)).clamp(0.0, 1.0);
    }

    // Choisit un type de virus selon la vague
    fn pick_kind(&self) -> VirusKind {
        let roll: f32 = thread_rng().gen_range(0.0..1.0);

        match self.number {
            // Vague 1: uniquement Classic
            1 => VirusKind::Classic,

            // Vague 2: petite chance de rapide
            2 => {
                if roll < 0.2 { VirusKind::Fast }
                else { VirusKind::Classic } 
            }

            // Vague 3: Classic majoritaire, quelques Fast, petite chance lourd
            3 => {
                if roll < 0.25 { VirusKind::Fast }
                else if roll < 0.35 { VirusKind::Heavy }
                else { VirusKind::Classic }
            }

            // Vague 4+ : tous les types, Boss possible
            _ => {
                if roll < 0.3 { VirusKind::Fast }
                else if roll < 0.5 { VirusKind::Heavy }
                else { VirusKind::Classic }
            }
        }
    }

    // Difficulté des mots selon le type de virus
    fn pick_difficulty(kind: &VirusKind) -> Difficulty {
        match kind {
            VirusKind::Fast    => Difficulty::Easy,
            VirusKind::Classic => Difficulty::Medium,
            VirusKind::Heavy   => Difficulty::Hard,
            VirusKind::Boss    => Difficulty::Hard,
        }
    }

    // fait apparaitre un virus
    fn spawn_one(&mut self) {
        // on calcule la position où le virus apparait
        let angle: f32 = thread_rng().gen_range(0.0..std::f32::consts::TAU); // TAU = 2pi, on choisit un angle aléatoire tout autour
        let radius = screen_width().hypot(screen_height()) / 2.0;
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let position = Vec2::new(cx + angle.cos() * radius, cy + angle.sin() * radius);

        // on choisit le type et le mot en fonction
        let kind = self.pick_kind();
        let difficulty = Self::pick_difficulty(&kind);
        let list = WordList::get(difficulty);
        let idx = thread_rng().gen_range(0..list.len()); // index aléatoire parmis la liste de mot
        let word = list[idx].to_string();

        // On crée le virus en lui-même
        self.spawned += 1;
        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, kind, word);
        self.entries.push(VirusEntry { virus, typing, active: false, glitch_timer: 0.0});
    }

    pub fn type_char(&mut self, c: char, vfx: &mut crate::core::vfx::VfxManager, player_pos: Vec2, assets: &crate::core::assets::GameAssets) {
        let any_active = self.entries.iter().any(|e| e.active);

        if any_active {
            // Cas où un Virus est déjà actif (cbilé par le joueur)
            let mut any_correct = false;

            for entry in self.entries.iter_mut().filter(|e| e.active) {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        any_correct = true;
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Complete => {
                        any_correct = true;
                        let hp = entry.virus.health;
                        entry.virus.take_damage(hp);
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        vfx.spawn_explosion(entry.virus.position, 20, entry.virus.color());
                        entry.active = false;
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Wrong => {
                        // Ce virus diverge — on le désactive et reset, et on ajoute un glitch visuel
                        entry.glitch_timer = 0.2;
                        entry.active = false;
                        entry.typing.reset();
                    }
                }
            }

            if !any_correct {
                vfx.trigger_shake(3.0, 0.1);
                play_sound(&assets.sound_error, PlaySoundParams { looped: false, volume: 1.0 });
                for entry in self.entries.iter_mut() {
                    if entry.active {
                        entry.glitch_timer = 0.2;
                    }
                    entry.typing.reset();
                    entry.active = false;
                }
            }
        } else {
            // Cas où encore aucun virus n'est actif (ciblé par le joueur)
            let mut found = false;
            for entry in self.entries.iter_mut() {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        entry.active = true;
                        found = true;
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Complete => {
                        let hp = entry.virus.health;
                        entry.virus.take_damage(hp);
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        vfx.spawn_explosion(entry.virus.position, 20, entry.virus.color());
                        found = true;
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Wrong => {
                        // On ne fait rien ici pour l'instant, on attend de voir si un autre virus correspond
                    }
                    TypingResult::Wrong => {}
                }
            }

            // Mauvaise lettre — rien ne correspond, on glitch tous les virus
            if !found {
                play_sound(&assets.sound_error, PlaySoundParams { looped: false, volume: 1.0 });
                for entry in self.entries.iter_mut() {
                    entry.glitch_timer = 0.2;
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec2) {
        self.elapsed += dt;
        
        let remaining = self.to_kill.saturating_sub(self.spawned);

        self.next_spawn_in -= dt / 2.0;
        if self.next_spawn_in <= 0.0 {
            if remaining > 0 {
                self.spawn_one();
                self.next_spawn_in = self.spawn_delay();
            }
        }

        let before = self.entries.len();
        for entry in &mut self.entries {
            entry.virus.update(dt, player_pos);
            if entry.glitch_timer > 0.0 {
                entry.glitch_timer -= dt;
            }
        }
        self.entries.retain(|e| e.virus.is_alive());
        let after = self.entries.len();

        self.killed += before - after;
    }

    pub fn draw(&self, assets: &crate::core::assets::GameAssets, global_offset: Vec2) {
        for entry in self.entries.iter() {
            let mut offset_x = global_offset.x;
            let mut offset_y = global_offset.y;
            let mut color_override = WHITE;
            
            if entry.glitch_timer > 0.0 {
                offset_x += rand::gen_range(-5.0, 5.0);
                offset_y += rand::gen_range(-5.0, 5.0);
                color_override = if rand::gen_range(0, 2) == 0 { RED } else { BLUE };
            }
            
            entry.virus.draw_with_offset(assets, offset_x, offset_y, color_override);

            let x = entry.virus.position.x - 20.0 + offset_x;
            let y = entry.virus.position.y - entry.virus.radius() - 8.0 + offset_y;

            if entry.active {
                renderer::draw_virus_word(
                    entry.typing.typed_part(),
                    entry.typing.remaining_part(),
                    x, y,
                    Some(&assets.font),
                );
            } else {
                renderer::draw_virus_word("", &entry.virus.word, x, y, Some(&assets.font));
            }
        }

        // Compteur de kills en bas de l'écran
        self.draw_kill_counter();
    }

    fn draw_kill_counter(&self) {
        let text = format!("{} / {}", self.killed, self.to_kill);
        let x = screen_width() / 2.0 - 30.0;
        let y = screen_height() - 16.0;
        draw_text(&text, x, y, 20.0, YELLOW);
    }

    pub fn is_complete(&self) -> bool {
        self.killed >= self.to_kill && self.entries.is_empty()
    }
}