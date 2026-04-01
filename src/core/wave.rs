use macroquad::prelude::*;
use ::rand::Rng;
use ::rand::thread_rng;

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::core::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus, // le virus
    pub typing: TypingState, // etat de frappe
    pub active: bool, // est-il ciblè par le joueur
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
        self.entries.push(VirusEntry { virus, typing, active: false });
    }

    // Gère l'état de complétion des mots pour tous les virus à l'écran
    pub fn type_char(&mut self, c: char) {
        let any_active = self.entries.iter().any(|e| e.active);

        if any_active {
            // Cas où un Virus est déjà actif (cbilé par le joueur)
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
                // Cas où le joueur se trompe
                for entry in self.entries.iter_mut() {
                    entry.typing.reset();
                    entry.active = false;
                }
            }
        } else {
            // Cas où encore aucun virus n'est actif (ciblé par le joueur)
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
        }
        self.entries.retain(|e| e.virus.is_alive());
        let after = self.entries.len();

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

    pub fn is_complete(&self) -> bool {
        self.killed >= self.to_kill && self.entries.is_empty()
    }
}