use macroquad::prelude::*;

use crate::core::virus::{Virus, VirusKind};
use crate::data::words::{Difficulty, WordList};
use crate::ui::input::TypingState;

pub const BOSS_EVERY_N_WAVES: u32 = 5;
pub const SUMMONER_BOSS_CYCLES: usize = 2;
const SUMMONER_BOSS_MINIONS_PER_CYCLE: usize = 4;
const SUMMONER_RADIUS: f32 = 80.0;

pub fn is_boss_wave(wave_number: u32) -> bool {
    wave_number > 0 && wave_number % BOSS_EVERY_N_WAVES == 0
}

pub fn boss_kind_for_wave(wave_number: u32) -> VirusKind {
    if wave_number == 5 {
        VirusKind::Boss
    } else if wave_number % 10 == 0 {
        VirusKind::SummonerBoss
    } else if wave_number > 5 && wave_number % 10 == 5 {
        VirusKind::ReverseBoss
    } else {
        VirusKind::Boss
    }
}

pub fn first_boss_word(wave_number: u32, kind: VirusKind) -> String {
    match kind {
        VirusKind::SummonerBoss => summoner_boss_word_for_phase(wave_number, 0).to_string(),
        VirusKind::ReverseBoss => reverse_boss_word_for_phase(wave_number, 0).to_string(),
        _ => boss_word_for_phase(wave_number, 0).to_string(),
    }
}

pub fn visual_word(kind: VirusKind, word: &str) -> String {
    if matches!(kind, VirusKind::ReverseBoss) {
        word.chars().rev().collect()
    } else {
        word.to_string()
    }
}

pub fn build_summoned_minions(wave_number: u32, cycle: usize, center: Vec2) -> Vec<(Vec2, String)> {
    let mut minions = Vec::with_capacity(SUMMONER_BOSS_MINIONS_PER_CYCLE);

    for i in 0..SUMMONER_BOSS_MINIONS_PER_CYCLE {
        let angle = (i as f32 / SUMMONER_BOSS_MINIONS_PER_CYCLE as f32) * std::f32::consts::TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let position = center + dir * SUMMONER_RADIUS;
        let word = summoned_minion_word(wave_number, cycle, i).to_string();
        minions.push((position, word));
    }

    minions
}

pub fn should_spawn_summoned_minions(
    kind: VirusKind,
    hp_before: u32,
    hp_after: u32,
    cycles_before: usize,
    cycles_after: usize,
) -> Option<usize> {
    if matches!(kind, VirusKind::SummonerBoss)
        && hp_after == hp_before
        && hp_after > 0
        && cycles_after > cycles_before
        && cycles_after <= SUMMONER_BOSS_CYCLES
    {
        Some(cycles_after)
    } else {
        None
    }
}

pub fn on_word_complete(
    wave_number: u32,
    virus: &mut Virus,
    typing: &mut TypingState,
    active: &mut bool,
    boss_phase: &mut usize,
    boss_spawn_cycles_done: &mut usize,
    has_alive_summoned: bool,
) {
    if matches!(virus.kind, VirusKind::Boss) {
        // Un mot valide retire 1 couche de bouclier au boss.
        virus.take_damage(1);

        if virus.is_alive() {
            *boss_phase += 1;
            let next_word = boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
        } else {
            *active = false;
        }
        return;
    }

    if matches!(virus.kind, VirusKind::ReverseBoss) {
        virus.take_damage(1);

        if virus.is_alive() {
            *boss_phase += 1;
            let next_word = reverse_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
        } else {
            *active = false;
        }
        return;
    }

    if matches!(virus.kind, VirusKind::SummonerBoss) {
        if has_alive_summoned {
            // Invincible tant qu'au moins un sbire invoque est vivant.
            *boss_phase += 1;
            let next_word = summoner_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
            return;
        }

        // Le boss invocateur n'est vulnerable qu'apres 2 cycles d'invocation.
        if *boss_spawn_cycles_done < SUMMONER_BOSS_CYCLES {
            *boss_spawn_cycles_done += 1;
            *boss_phase += 1;
            let next_word = summoner_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
            return;
        }

        virus.take_damage(1);
        *active = false;
        return;
    }

    let hp = virus.health;
    virus.take_damage(hp);
    *active = false;
}

fn boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // Hard words pour renforcer l'identite boss
    WordList::pick(Difficulty::Hard, wave_number as usize + phase)
}

fn summoner_boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // On garde des mots difficiles pour le boss invocateur aussi.
    WordList::pick(Difficulty::Hard, wave_number as usize + 17 + phase)
}

fn reverse_boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // Variante de seed pour limiter les répétitions avec les autres boss.
    WordList::pick(Difficulty::Hard, wave_number as usize + 31 + phase)
}

fn summoned_minion_word(wave_number: u32, cycle: usize, idx: usize) -> &'static str {
    let seed = wave_number as usize + cycle * SUMMONER_BOSS_MINIONS_PER_CYCLE + idx;
    WordList::pick(Difficulty::Easy, seed)
}
