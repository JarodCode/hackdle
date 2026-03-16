use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::{TypingState, TypingResult};
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
    pub entries: Vec<VirusEntry>,
}

impl Wave {
    pub fn new(number: u32) -> Self {
        let entries = Self::spawn_viruses(number);
        Self { number, entries }
    }

    fn spawn_viruses(wave_number: u32) -> Vec<VirusEntry> {
        let count = 3 + wave_number as usize;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let position = Self::spawn_position(i, count);
            let word = WordList::pick(Difficulty::Easy, i).to_string();
            let typing = TypingState::new(word.clone());
            let virus = Virus::new(position, VirusKind::Classic, word);
            entries.push(VirusEntry { virus, typing, active: false, glitch_timer: 0.0 });
        }

        entries
    }

    fn spawn_position(index: usize, total: usize) -> Vec2 {
        let angle = (index as f32 / total as f32) * std::f32::consts::TAU;
        let radius = screen_width().min(screen_height()) * 0.5;
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        Vec2::new(cx + angle.cos() * radius, cy + angle.sin() * radius)
    }

    pub fn type_char(&mut self, c: char, vfx: &mut crate::core::vfx::VfxManager, player_pos: Vec2, assets: &crate::core::assets::GameAssets) {
        let any_active = self.entries.iter().any(|e| e.active);

        if any_active {
            // On tape sur tous les virus actifs en parallèle
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

            // Si aucun virus actif n'a accepté le caractère — reset global
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
            // Aucun virus actif — on cherche tous ceux qui commencent par ce caractère
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
                        // Mot d'une seule lettre
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
        for entry in &mut self.entries {
            entry.virus.update(dt, player_pos);
            if entry.glitch_timer > 0.0 {
                entry.glitch_timer -= dt;
            }
        }
        self.entries.retain(|e| e.virus.is_alive());
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
    }

    pub fn is_complete(&self) -> bool {
        self.entries.is_empty()
    }
}