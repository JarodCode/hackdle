use macroquad::prelude::*;

use crate::core::boss;
use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState,
    // true si ce virus est en cours de frappe
    pub active: bool,
    // Phase de bouclier du boss (0, 1, 2, ...)
    pub boss_phase: usize,
    // Nombre de vagues de sbires déjà invoquées (boss invocateur uniquement)
    pub boss_spawn_cycles_done: usize,
    // Marque les sbires invoqués par le boss invocateur
    pub summoned_by_boss: bool,
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
        if boss::is_boss_wave(wave_number) {
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
                boss_spawn_cycles_done: 0,
                summoned_by_boss: false,
            });
        }

        entries
    }

    fn spawn_boss(wave_number: u32) -> VirusEntry {
        let kind = boss::boss_kind_for_wave(wave_number);
        let position = Vec2::new(screen_width() / 2.0, 90.0);
        let word = boss::first_boss_word(wave_number, kind);
        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, kind, word);

        VirusEntry {
            virus,
            typing,
            active: false,
            boss_phase: 0,
            boss_spawn_cycles_done: 0,
            summoned_by_boss: false,
        }
    }

    fn spawn_summoned_minions(&mut self, wave_number: u32, cycle: usize, center: Vec2) {
        for (position, word) in boss::build_summoned_minions(wave_number, cycle, center) {
            let typing = TypingState::new(word.clone());
            let virus = Virus::new(position, VirusKind::Fast, word);

            self.entries.push(VirusEntry {
                virus,
                typing,
                active: false,
                boss_phase: 0,
                boss_spawn_cycles_done: 0,
                summoned_by_boss: true,
            });
        }
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
        let has_alive_summoned = self
            .entries
            .iter()
            .any(|e| e.summoned_by_boss && e.virus.is_alive());
        let mut summon_requests: Vec<(usize, Vec2)> = Vec::new();

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
                        let hp_before = entry.virus.health;
                        let cycles_before = entry.boss_spawn_cycles_done;
                        boss::on_word_complete(
                            self.number,
                            &mut entry.virus,
                            &mut entry.typing,
                            &mut entry.active,
                            &mut entry.boss_phase,
                            &mut entry.boss_spawn_cycles_done,
                            has_alive_summoned,
                        );
                        let hp_after = entry.virus.health;
                        let cycles_after = entry.boss_spawn_cycles_done;

                        if let Some(cycle) = boss::should_spawn_summoned_minions(
                            entry.virus.kind,
                            hp_before,
                            hp_after,
                            cycles_before,
                            cycles_after,
                        ) {
                            summon_requests.push((cycle, entry.virus.position));
                        }
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

            for (cycle, center) in summon_requests {
                self.spawn_summoned_minions(self.number, cycle, center);
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
                        let hp_before = entry.virus.health;
                        let cycles_before = entry.boss_spawn_cycles_done;
                        boss::on_word_complete(
                            self.number,
                            &mut entry.virus,
                            &mut entry.typing,
                            &mut entry.active,
                            &mut entry.boss_phase,
                            &mut entry.boss_spawn_cycles_done,
                            has_alive_summoned,
                        );
                        let hp_after = entry.virus.health;
                        let cycles_after = entry.boss_spawn_cycles_done;

                        if let Some(cycle) = boss::should_spawn_summoned_minions(
                            entry.virus.kind,
                            hp_before,
                            hp_after,
                            cycles_before,
                            cycles_after,
                        ) {
                            summon_requests.push((cycle, entry.virus.position));
                        }
                        found = true;
                    }
                    TypingResult::Wrong => {
                        // Ce virus ne commence pas par ce caractère — on ignore
                    }
                }
            }

            // Mauvaise lettre — rien ne correspond, on ne fait rien
            let _ = found;

            for (cycle, center) in summon_requests {
                self.spawn_summoned_minions(self.number, cycle, center);
            }
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
}