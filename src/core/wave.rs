use macroquad::prelude::*;

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::{TypingState, TypingResult};
use crate::ui::renderer;

const BOSS_EVERY_N_WAVES: u32 = 5;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState,
    // true si ce virus est en cours de frappe
    pub active: bool,
    // Phase de bouclier du boss (0, 1, 2, ...)
    pub boss_phase: usize,
}

pub struct Wave {
    pub number: u32,
    pub entries: Vec<VirusEntry>,
}

impl Wave {
    pub fn new(number: u32) -> Self {
        let entries = Self::spawn_viruses(number);
        Self { number, entries }
    }

    fn spawn_viruses(wave_number: u32) -> Vec<VirusEntry> {
        if Self::is_boss_wave(wave_number) {
            return vec![Self::spawn_boss(wave_number)];
        }

        let count = 3 + wave_number as usize;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let position = Self::spawn_position(i, count);
            let word = WordList::pick(Difficulty::Easy, i).to_string();
            let typing = TypingState::new(word.clone());
            let virus = Virus::new(position, VirusKind::Classic, word);
            entries.push(VirusEntry {
                virus,
                typing,
                active: false,
                boss_phase: 0,
            });
        }

        entries
    }

    fn is_boss_wave(wave_number: u32) -> bool {
        wave_number > 0 && wave_number % BOSS_EVERY_N_WAVES == 0
    }

    fn spawn_boss(wave_number: u32) -> VirusEntry {
        let position = Vec2::new(screen_width() / 2.0, 90.0);
        let word = Self::boss_word_for_phase(wave_number, 0).to_string();
        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, VirusKind::Boss, word);

        VirusEntry {
            virus,
            typing,
            active: false,
            boss_phase: 0,
        }
    }

    fn boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
        // Hard words pour renforcer l'identité boss
        WordList::pick(Difficulty::Hard, wave_number as usize + phase)
    }

    fn spawn_position(index: usize, total: usize) -> Vec2 {
        let angle = (index as f32 / total as f32) * std::f32::consts::TAU;
        let radius = screen_width().min(screen_height()) * 0.5;
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        Vec2::new(cx + angle.cos() * radius, cy + angle.sin() * radius)
    }

    pub fn type_char(&mut self, c: char) {
        let any_active = self.entries.iter().any(|e| e.active);

        if any_active {
            // On tape sur tous les virus actifs en parallèle
            let mut any_correct = false;

            for entry in self.entries.iter_mut().filter(|e| e.active) {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        any_correct = true;
                    }
                    TypingResult::Complete => {
                        any_correct = true;
                        Self::on_word_complete(self.number, entry);
                    }
                    TypingResult::Wrong => {
                        // Ce virus diverge — on le désactive et reset
                        entry.active = false;
                        entry.typing.reset();
                    }
                }
            }

            // Si aucun virus actif n'a accepté le caractère — reset global
            if !any_correct {
                for entry in self.entries.iter_mut() {
                    entry.typing.reset();
                    entry.active = false;
                }
            }
        } else {
            // Aucun virus actif — on cherche tous ceux qui commencent par ce caractère
            let mut found = false;
            for entry in self.entries.iter_mut() {
                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        entry.active = true;
                        found = true;
                    }
                    TypingResult::Complete => {
                        // Mot d'une seule lettre
                        Self::on_word_complete(self.number, entry);
                        found = true;
                    }
                    TypingResult::Wrong => {
                        // Ce virus ne commence pas par ce caractère — on ignore
                    }
                }
            }

            // Mauvaise lettre — rien ne correspond, on ne fait rien
            let _ = found;
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec2) {
        for entry in &mut self.entries {
            entry.virus.update(dt, player_pos);
        }
        self.entries.retain(|e| e.virus.is_alive());
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
    }

    pub fn is_complete(&self) -> bool {
        self.entries.is_empty()
    }

    fn on_word_complete(wave_number: u32, entry: &mut VirusEntry) {
        if matches!(entry.virus.kind, VirusKind::Boss) {
            // Un mot validé retire 1 couche de bouclier au boss.
            entry.virus.take_damage(1);

            if entry.virus.is_alive() {
                entry.boss_phase += 1;
                let next_word = Self::boss_word_for_phase(wave_number, entry.boss_phase).to_string();
                entry.virus.word = next_word.clone();
                entry.typing = TypingState::new(next_word);
                entry.active = true;
            } else {
                entry.active = false;
            }
            return;
        }

        let hp = entry.virus.health;
        entry.virus.take_damage(hp);
        entry.active = false;
    }
}