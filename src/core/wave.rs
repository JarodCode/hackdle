use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use ::rand::Rng;
use ::rand::thread_rng;

use crate::core::boss::{
    self,
    VirusBehavior,
    SummonerResult,
};
use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::{TypingState, TypingResult};
use crate::ui::renderer;

pub struct VirusEntry {
    pub virus: Virus,
    pub typing: TypingState, // Avancée de l'écriture du mot
    pub active: bool, // Est-ce que le joueur cible ce virus
    pub behavior: VirusBehavior, // Normal, Boss, SummoningBoss ou ReverseBoss (Boos.rs)
    pub summoned_by_boss: bool, // Est-ce que le virus a été invoqué par le summoning Boss
    pub glitch_timer: f32,
}

pub struct Wave {
    pub number: u32, // Numéro de vague (redondant)
    pub entries: Vec<VirusEntry>, // Les virus à l'écran
    pub to_kill: usize, // Le nombre de virus qu'il reste à tuer
    pub killed: usize, // Le nombre de virus tuer
    spawned: usize, // Le nombre de virus qui sont apparu
    elapsed: f32, // Le temps passé
    next_spawn_in: f32, // Délais entre deux spawn
}

impl VirusEntry {
    // SI le virus à faire apparaitre est un boss alors on complète son comportement selon le type de boss que c'est
    fn handle_completion(&mut self, wave_number: u32, has_alive_summoned: bool) -> Option<(usize, Vec2)> {
        match &mut self.behavior {
            VirusBehavior::Boss(state) => {
                let dead = boss::on_boss_word_complete(
                    state,
                    wave_number,
                    self.virus.kind,
                    &mut self.virus.word,
                    &mut self.typing,
                );
                if dead { self.virus.kill(); }
                None
            }
            VirusBehavior::ReverseBoss(state) => {
                let dead = boss::on_reverse_boss_word_complete(
                    state,
                    wave_number,
                    &mut self.virus.word,
                    &mut self.typing,
                );
                if dead { self.virus.kill(); }
                None
            }
            VirusBehavior::SummonerBoss(state) => {
                match boss::on_summoner_word_complete(
                    state,
                    wave_number,
                    has_alive_summoned,
                    &mut self.virus.word,
                    &mut self.typing,
                ) {
                    SummonerResult::SpawnMinions(cycle) => {
                        Some((cycle, self.virus.position))
                    }
                    SummonerResult::Killed => {
                        self.virus.kill();
                        None
                    }
                    SummonerResult::NextWord => None,
                }
            }
            VirusBehavior::Normal => {
                self.virus.kill();
                None
            }
        }
    }
}

impl Wave {
    // Initialise une vague avec son objectif de kills
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

    // Détermine la quantité d'ennemis à éliminer selon la vague
    fn kills_required(wave_number: u32) -> usize {
        match wave_number {
            1 => 5,
            2 => 8,
            3 => 12,
            _ => 12 + (wave_number as usize - 3) * 4,
        }
    }

    // Détermine le délai avant le prochain spawn
    fn spawn_delay(&self) -> f32 {
        let remaining = self.to_kill.saturating_sub(self.spawned);
        if remaining == 0 {
            return f32::MAX;
        }
        // Plus la vague avance, plus les spawns se rapprochent (logarithmique), sans tomber à 0
        (-(self.elapsed.ln() / 6.0 - 1.0)).clamp(0.0, 1.0)
    }

    // Selon la manche les chances d'appartition du type d'ennemie changent
    fn pick_kind(&self) -> VirusKind {
        let roll: f32 = thread_rng().gen_range(0.0..1.0);

        match self.number {
            // manche 1 : que des virus classiques
            1 => VirusKind::Classic,
            // manche 2 : quelques ennemies rapides
            2 => {
                if roll < 0.2 { VirusKind::Fast }
                else { VirusKind::Classic }
            }
            // manche 3 : ennemies rapides et quelques ennemies lourds
            3 => {
                if roll < 0.25 { VirusKind::Fast }
                else if roll < 0.35 { VirusKind::Heavy }
                else { VirusKind::Classic }
            }
            // manche 4 et plus : tous les types de virus
            _ => {
                if roll < 0.3 { VirusKind::Fast }
                else if roll < 0.5 { VirusKind::Heavy }
                else { VirusKind::Classic }
            }
        }
    }

    // Associe chaque type d'ennemi à sa difficulté de mots.
    fn pick_difficulty(kind: &VirusKind) -> Difficulty {
        match kind {
            VirusKind::Fast         => Difficulty::Easy,
            VirusKind::Classic      => Difficulty::Medium,
            VirusKind::Heavy        => Difficulty::Hard,
            VirusKind::Boss         => Difficulty::Hard,
            VirusKind::SummonerBoss => Difficulty::Hard,
            VirusKind::ReverseBoss  => Difficulty::Hard,
        }
    }

    // Spawn le boss attendu sur une vague boss (multiple de 5)
    fn spawn_boss_entry(&mut self) {
        // position du spawn
        let angle: f32 = std::f32::consts::FRAC_PI_2; // spawn toujours en bas 
        let radius = screen_width().hypot(screen_height()) / 3.6;
        // On prend le centre de l'écran
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let position = Vec2::new(cx + angle.cos() * radius, cy + angle.sin() * radius);

        let kind = boss::boss_kind_for_wave(self.number);
        let word = boss::first_boss_word(self.number, kind);
        let typing = TypingState::new(word.clone());
        let virus = Virus::new(position, kind, word);

        self.entries.push(VirusEntry {
            virus,
            typing,
            active: false,
            behavior: VirusBehavior::for_kind(kind, self.number),
            summoned_by_boss: false,
            glitch_timer: 0.0,
        });

        self.spawned += 1;
    }

    // Fait apparaitre les sbires invoqués par le SummonerBoss
    fn spawn_summoned_minions(&mut self, wave_number: u32, cycle: usize, center: Vec2) {
        for (position, word) in boss::build_summoned_minions(wave_number, cycle, center) {
            let typing = TypingState::new(word.clone());
            let virus = Virus::new(position, VirusKind::Classic, word);

            self.entries.push(VirusEntry {
                virus,
                typing,
                active: false,
                behavior: VirusBehavior::Normal,
                summoned_by_boss: true,
                glitch_timer: 0.0,
            });
        }
    }

    // Fais apparaitre un virus
    fn spawn_one(&mut self) {
        if boss::is_boss_wave(self.number) {
            if self.spawned == 0 {
                self.spawn_boss_entry();
            }
            return;
        }

        // Position d'apparition du Virus
        let angle: f32 = thread_rng().gen_range(0.0..std::f32::consts::TAU); // TAU = 2pi, on choisit un angle aléatoire sur l'ensemble du cercle
        let radius = screen_width().hypot(screen_height()) / 2.0;
        // On prend le centre de l'écran
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
        self.entries.push(VirusEntry {
            virus,
            typing,
            active: false,
            behavior: VirusBehavior::Normal,
            summoned_by_boss: false,
            glitch_timer: 0.0,
        });
    }

    // Traite une frappe clavier et la route vers la bonne cible active 
    pub fn type_char(
        &mut self,
        c: char,
        vfx: &mut crate::ui::vfx::VfxManager,
        player_pos: Vec2,
        assets: &crate::ui::assets::GameAssets,
    ) {
        // Le SummonerBoss reste verrouillé tant qu'au moins un sbire est en vie
        let has_alive_summoned = self
            .entries
            .iter()
            .any(|e| e.summoned_by_boss && e.virus.is_alive());

        // On regarde si il y a encore des summoned en vie, si oui on empêche le ciblage
        if has_alive_summoned {
            for entry in self.entries.iter_mut() {
                if matches!(entry.virus.kind, VirusKind::SummonerBoss) {
                    entry.active = false;
                    entry.typing.reset();
                }
            }
        }

        let any_active = self.entries.iter().any(|e| e.active);
        // Les summons sont différés pour éviter d'emprunter self.entries
        let mut summon_requests: Vec<(usize, Vec2)> = Vec::new(); // mut car on push dessus ligne 285

        // Si un Virus est déjà ciblé
        if any_active {
            let mut any_correct = false;

            // Une fois une cible verrouillée (active), seule cette cible progresse
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
                        // Erreur sur une cible active: on casse le lock et on redemande un ciblage
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

            // Fais apparaitre les 
            for (cycle, center) in summon_requests {
                self.spawn_summoned_minions(self.number, cycle, center);
            }
            
        // Aucun Virus n'est déjà ciblé
        } else {
            let mut found = false;
            let mut summon_requests_no_active: Vec<(usize, Vec2)> = Vec::new();

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
                            summon_requests_no_active.push(req);
                        }
                        vfx.spawn_laser(player_pos, entry.virus.position, GREEN);
                        if !entry.virus.is_alive() {
                            vfx.spawn_explosion(entry.virus.position, 20, entry.virus.color());
                        }
                        found = true;
                        play_sound(&assets.sound_laser, PlaySoundParams { looped: false, volume: 0.5 });
                    }
                    TypingResult::Wrong => {}
                }
            }

            for (cycle, center) in summon_requests_no_active {
                self.spawn_summoned_minions(self.number, cycle, center);
            }

            if !found {
                play_sound(&assets.sound_error, PlaySoundParams { looped: false, volume: 1.0 });
                for entry in self.entries.iter_mut() {
                    entry.glitch_timer = 0.2;
                }
            }
        }
    }

    // Fait avancer la simulation: spawn, mouvement et nettoyage des morts
    pub fn update(&mut self, dt: f32, player_pos: Vec2) {
        self.elapsed += dt;

        // Combien il reste de Virus a spawn
        let remaining = self.to_kill.saturating_sub(self.spawned); // sat_sub : soustraction qui s'arrête à 0

        self.next_spawn_in -= dt / 2.0; // 2.0 permet de règler la difficulté (future amélioration)
        // On diminue .next_spawn_in à chaque itération en dessous de 0 le Virus spawn
        if self.next_spawn_in <= 0.0 {
            if remaining > 0 {
                self.spawn_one();
                // Plus le temps passe plus spawn_delay est petit
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

        // Les kills sont comptés uniquement au moment où l'entrée disparaît
        self.killed += before - after;
    }

    // Délègue le rendu complet de la vague au renderer UI
    pub fn draw(&self, assets: &crate::ui::assets::GameAssets, global_offset: Vec2) {
        renderer::draw_wave(&self.entries, self.killed, self.to_kill, assets, global_offset);
    }

    // Une vague est finie quand l'objectif est atteint et qu'il ne reste plus d'ennemis (pas nécessaire mais garde fou)
    pub fn is_complete(&self) -> bool {
        self.killed >= self.to_kill && self.entries.is_empty()
    }
}