use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use ::rand::Rng;
use ::rand::thread_rng;

use crate::core::boss;
use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::core::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState,
    // true si ce virus est en cours de frappe
    pub active: bool,
    // Phase de bouclier du boss (0, 1, 2, ...)
    pub boss_phase: usize,
    // Nombre de mots restants avant de vaincre le boss.
    pub boss_words_remaining: usize,
    // Nombre de vagues de sbires déjà invoquées (boss invocateur uniquement)
    pub boss_spawn_cycles_done: usize,
    // Marque les sbires invoqués par le boss invocateur
    pub summoned_by_boss: bool,
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

impl VirusEntry {
    fn handle_completion(&mut self, wave_number: u32, has_alive_summoned: bool) -> Option<(usize, Vec2)> {
        let cycles_before = self.boss_spawn_cycles_done;

        boss::on_word_complete(
            wave_number,
            &mut self.virus,
            &mut self.typing,
            &mut self.active,
            &mut self.boss_phase,
            &mut self.boss_words_remaining,
            &mut self.boss_spawn_cycles_done,
            has_alive_summoned,
        );

        let cycles_after = self.boss_spawn_cycles_done;
        boss::should_spawn_summoned_minions(
            self.virus.kind,
            wave_number,
            cycles_before,
            cycles_after,
        )
        .map(|cycle| (cycle, self.virus.position))
    }
}

impl Wave {
    pub fn new(number: u32) -> Self {
        let to_kill = if boss::is_boss_wave(number) { 1 } else { Self::kills_required(number) };
        Self {
            number,
            entries: Vec::new(),
            to_kill,
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
            VirusKind::SummonerBoss => Difficulty::Hard,
            VirusKind::ReverseBoss => Difficulty::Hard,
        }
    }

    fn spawn_boss_entry(&mut self) {
        let kind = boss::boss_kind_for_wave(self.number);
        let center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let radius = screen_width().min(screen_height()) * 0.4;
        let position = center + Vec2::new(0.0, -radius);
        let word = boss::first_boss_word(self.number, kind);
        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, kind, word);

        self.entries.push(VirusEntry {
            virus,
            typing,
            active: false,
            boss_phase: 0,
            boss_words_remaining: boss::initial_boss_words_remaining(kind, self.number),
            boss_spawn_cycles_done: 0,
            summoned_by_boss: false,
            glitch_timer: 0.0,
        });

        self.spawned += 1;
    }

    fn spawn_summoned_minions(&mut self, wave_number: u32, cycle: usize, center: Vec2) {
        for (position, word) in boss::build_summoned_minions(wave_number, cycle, center) {
            let typing = TypingState::new(word.clone());
            let virus = Virus::new(position, VirusKind::Classic, word);

            self.entries.push(VirusEntry {
                virus,
                typing,
                active: false,
                boss_phase: 0,
                boss_words_remaining: 0,
                boss_spawn_cycles_done: 0,
                summoned_by_boss: true,
                glitch_timer: 0.0,
            });
        }
    }

    // fait apparaitre un virus
    fn spawn_one(&mut self) {
        if boss::is_boss_wave(self.number) {
            if self.spawned == 0 {
                self.spawn_boss_entry();
            }
            return;
        }

        // on calcule la position où le virus apparait
        let angle: f32 = thread_rng().gen_range(0.0..std::f32::consts::TAU); // TAU = 2pi, on choisit un angle aléatoire tout autour
        let radius = screen_width().min(screen_height()) * 0.45;
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
        let has_alive_summoned = self
            .entries
            .iter()
            .any(|e| e.summoned_by_boss && e.virus.is_alive());

        // Priorité aux sbires invoqués: tant qu'ils sont vivants,
        // on évite que le boss invocateur capte la saisie.
        if has_alive_summoned {
            for entry in self.entries.iter_mut() {
                if matches!(entry.virus.kind, VirusKind::SummonerBoss) {
                    entry.active = false;
                    entry.typing.reset();
                }
            }
        }

        let any_active = self.entries.iter().any(|e| e.active);
        let mut summon_requests: Vec<(usize, Vec2)> = Vec::new();

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
                        if let Some(req) = entry.handle_completion(self.number, has_alive_summoned) {
                            summon_requests.push(req);
                        }
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        if !entry.virus.is_alive() {
                            vfx.spawn_explosion(entry.virus.position, 20, entry.virus.color());
                        }
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

            for (cycle, center) in summon_requests {
                self.spawn_summoned_minions(self.number, cycle, center);
            }
        } else {
            // Cas où encore aucun virus n'est actif (ciblé par le joueur)
            let mut found = false;
            for entry in self.entries.iter_mut() {
                if has_alive_summoned && matches!(entry.virus.kind, VirusKind::SummonerBoss) {
                    continue;
                }

                match entry.typing.type_char(c) {
                    TypingResult::Correct => {
                        entry.active = true;
                        found = true;
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Complete => {
                        if let Some(req) = entry.handle_completion(self.number, has_alive_summoned) {
                            summon_requests.push(req);
                        }
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        if !entry.virus.is_alive() {
                            vfx.spawn_explosion(entry.virus.position, 20, entry.virus.color());
                        }
                        found = true;
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Wrong => {
                        // On ne fait rien ici pour l'instant, on attend de voir si un autre virus correspond
                    }
                }
            }

            for (cycle, center) in summon_requests {
                self.spawn_summoned_minions(self.number, cycle, center);
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

            if matches!(entry.virus.kind, VirusKind::ReverseBoss) {
                // Boss inverse: inversion uniquement visuelle, la saisie reste normale.
                let visible = boss::visual_word(entry.virus.kind, &entry.virus.word);
                renderer::draw_virus_word("", &visible, x, y, Some(&assets.font));
            } else if entry.active {
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