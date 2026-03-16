use macroquad::prelude::*;
use ::rand::Rng;
use ::rand::thread_rng;

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState,
    pub active: bool,
}

pub struct Wave {
    pub number: u32,
    pub entries: Vec<VirusEntry>,
    to_kill: usize,      // nombre de virus à tuer pour terminer la vague
    killed: usize,       // nombre de virus tués jusqu'ici
    spawned: usize,      // nombre de virus spawnés jusqu'ici
    spawn_timer: f32,
    spawn_tick: f32,
    elapsed: f32,        // temps écoulé — sert à faire monter la probabilité
}

impl Wave {
    pub fn new(number: u32) -> Self {
        let mut wave = Self {
            number,
            entries: Vec::new(),
            to_kill: Self::kills_required(number),
            killed: 0,
            spawned: 0,
            spawn_timer: 0.5,
            spawn_tick: 0.5,
            elapsed: 0.0,
        };
        // Spawn immédiat du premier virus sans attendre le premier tick
        wave.spawn_one();
        wave
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

    // Probabilité de spawn — monte avec le temps mais se calme
    // si trop de virus sont déjà à l'écran
    fn spawn_probability(&self) -> f32 {
        // Pas besoin de spawner plus que ce qu'il reste à tuer
        let remaining = self.to_kill.saturating_sub(self.spawned);
        if remaining == 0 || self.entries.len() >= 5 {
            return 0.0;
        }

        // La pression monte avec le temps
        let pressure = (self.elapsed / 20.0).clamp(0.0, 1.0);

        let base = match self.number {
            1 => 0.05,
            2 => 0.08,
            3 => 0.12,
            _ => 0.15,
        };

        let max = match self.number {
            1 => 0.30,
            2 => 0.45,
            3 => 0.60,
            _ => (0.60 + (self.number as f32 - 3.0) * 0.10).min(0.90),
        };

        base + (max - base) * pressure
    }

    fn try_spawn(&mut self) {
        let prob = self.spawn_probability();
        if prob == 0.0 { return; }

        let roll: f32 = thread_rng().gen_range(0.0..1.0);
        if roll < prob {
            self.spawn_one();
        }
    }

    // Choisit un type de virus selon la vague — nouveaux types débloqués progressivement
    fn pick_kind(&self) -> VirusKind {
        let roll: f32 = thread_rng().gen_range(0.0..1.0);

        match self.number {
            // Vague 1-2 : uniquement Classic
            1..=2 => VirusKind::Classic,

            // Vague 3-4 : Classic majoritaire, quelques Fast
            3..=4 => {
                if roll < 0.25 { VirusKind::Fast }
                else { VirusKind::Classic }
            }

            // Vague 5-6 : Fast et Classic, premiers Heavy
            5..=6 => {
                if roll < 0.20 { VirusKind::Fast }
                else if roll < 0.35 { VirusKind::Heavy }
                else { VirusKind::Classic }
            }

            // Vague 7+ : tous les types, Boss possible
            _ => {
                if roll < 0.20 { VirusKind::Fast }
                else if roll < 0.40 { VirusKind::Heavy }
                else if roll < 0.50 { VirusKind::Boss }
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

    fn spawn_one(&mut self) {
        let angle: f32 = thread_rng().gen_range(0.0..std::f32::consts::TAU);
        let margin = 60.0;
        let radius = (screen_width().hypot(screen_height()) / 2.0) + margin;

        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let position = Vec2::new(cx + angle.cos() * radius, cy + angle.sin() * radius);

        let kind = self.pick_kind();
        let difficulty = Self::pick_difficulty(&kind);
        let list = WordList::get(difficulty);
        let idx = thread_rng().gen_range(0..list.len());
        let word = list[idx].to_string();
        self.spawned += 1;

        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, kind, word);
        self.entries.push(VirusEntry { virus, typing, active: false });
    }

    pub fn type_char(&mut self, c: char) {
        let any_active = self.entries.iter().any(|e| e.active);

        if any_active {
            let mut any_correct = false;

            for entry in self.entries.iter_mut().filter(|e| e.active) {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        any_correct = true;
                    }
                    TypingResult::Complete => {
                        any_correct = true;
                        let hp = entry.virus.health;
                        entry.virus.take_damage(hp);
                        entry.active = false;
                    }
                    TypingResult::Wrong => {
                        entry.active = false;
                        entry.typing.reset();
                    }
                }
            }

            if !any_correct {
                for entry in self.entries.iter_mut() {
                    entry.typing.reset();
                    entry.active = false;
                }
            }
        } else {
            for entry in self.entries.iter_mut() {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        entry.active = true;
                    }
                    TypingResult::Complete => {
                        let hp = entry.virus.health;
                        entry.virus.take_damage(hp);
                    }
                    TypingResult::Wrong => {}
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec2) {
        self.elapsed += dt;

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.try_spawn();
            self.spawn_timer = self.spawn_tick;
        }

        let before = self.entries.len();
        for entry in &mut self.entries {
            entry.virus.update(dt, player_pos);
        }
        self.entries.retain(|e| e.virus.is_alive());
        let after = self.entries.len();

        // Chaque virus supprimé par frappe compte comme un kill
        self.killed += before - after;
    }

    pub fn draw(&self) {
        for entry in self.entries.iter() {
            entry.virus.draw();

            let x = entry.virus.position.x - 20.0;
            let y = entry.virus.position.y - entry.virus.radius() - 8.0;

            if entry.active {
                renderer::draw_virus_word(
                    entry.typing.typed_part(),
                    entry.typing.remaining_part(),
                    x, y,
                );
            } else {
                renderer::draw_virus_word("", &entry.virus.word, x, y);
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

    // Appelé depuis game.rs quand un virus atteint le joueur
    pub fn register_kill(&mut self) {
        self.killed += 1;
    }

    pub fn is_complete(&self) -> bool {
        self.killed >= self.to_kill && self.entries.is_empty()
    }
}