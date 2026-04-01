use macroquad::prelude::*;

use crate::core::virus::{Virus, VirusKind};
use crate::core::input::TypingState;
use crate::data::words::{Difficulty, WordList};

pub const BOSS_EVERY_N_WAVES: u32 = 5;
const CLASSIC_BOSS_WORDS: usize = 4;
const REVERSE_BOSS_BASE_WORDS: usize = 3;
const SUMMONER_BOSS_FINAL_WORDS: usize = 1;
const SUMMONER_BOSS_BASE_CYCLES: usize = 2;
const SUMMONER_BOSS_SEED_OFFSET: usize = 17;
const REVERSE_BOSS_SEED_OFFSET: usize = 31;
const SUMMONER_BOSS_MINIONS_PER_CYCLE: usize = 4;
const SUMMONER_RADIUS: f32 = 140.0;

pub fn is_boss_wave(wave_number: u32) -> bool {
    wave_number > 0 && wave_number % BOSS_EVERY_N_WAVES == 0
}

pub fn boss_kind_for_wave(wave_number: u32) -> VirusKind {
    if !is_boss_wave(wave_number) {
        return VirusKind::Boss;
    }

    let boss_index = wave_number / BOSS_EVERY_N_WAVES;
    if boss_index == 1 {
        VirusKind::Boss
    } else if boss_index % 2 == 0 {
        VirusKind::SummonerBoss
    } else {
        VirusKind::ReverseBoss
    }
}

pub fn first_boss_word(wave_number: u32, kind: VirusKind) -> String {
    get_boss_word(kind, wave_number, 0)
}

pub fn initial_boss_words_remaining(kind: VirusKind, wave_number: u32) -> usize {
    match kind {
        VirusKind::Boss => CLASSIC_BOSS_WORDS,
        VirusKind::ReverseBoss => reverse_boss_words_for_wave(wave_number),
        VirusKind::SummonerBoss => SUMMONER_BOSS_FINAL_WORDS,
        _ => 0,
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
    wave_number: u32,
    cycles_before: usize,
    cycles_after: usize,
) -> Option<usize> {
    if matches!(kind, VirusKind::SummonerBoss)
        && cycles_after > cycles_before
        && cycles_after <= summoner_boss_cycles_for_wave(wave_number)
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
    boss_words_remaining: &mut usize,
    boss_spawn_cycles_done: &mut usize,
    has_alive_summoned: bool,
) {
    match virus.kind {
        VirusKind::Boss | VirusKind::ReverseBoss => {
            progress_or_kill(
                virus.kind,
                wave_number,
                virus,
                typing,
                active,
                boss_phase,
                boss_words_remaining,
            );
        }
        VirusKind::SummonerBoss => {
            if has_alive_summoned {
                // Invincible tant qu'au moins un sbire invoque est vivant.
                set_next_boss_word(virus.kind, wave_number, virus, typing, active, boss_phase);
            } else if *boss_spawn_cycles_done < summoner_boss_cycles_for_wave(wave_number) {
                // Le boss invocateur devient plus difficile avec +1 cycle tous les 10 niveaux.
                *boss_spawn_cycles_done += 1;
                set_next_boss_word(virus.kind, wave_number, virus, typing, active, boss_phase);
            } else {
                progress_or_kill(
                    virus.kind,
                    wave_number,
                    virus,
                    typing,
                    active,
                    boss_phase,
                    boss_words_remaining,
                );
            }
        }
        _ => {
            virus.kill();
            *active = false;
        }
    }
}

fn progress_or_kill(
    kind: VirusKind,
    wave_number: u32,
    virus: &mut Virus,
    typing: &mut TypingState,
    active: &mut bool,
    boss_phase: &mut usize,
    boss_words_remaining: &mut usize,
) {
    if *boss_words_remaining > 1 {
        *boss_words_remaining -= 1;
        set_next_boss_word(kind, wave_number, virus, typing, active, boss_phase);
    } else {
        *boss_words_remaining = 0;
        virus.kill();
        *active = false;
    }
}

fn set_next_boss_word(
    kind: VirusKind,
    wave_number: u32,
    virus: &mut Virus,
    typing: &mut TypingState,
    active: &mut bool,
    boss_phase: &mut usize,
) {
    *boss_phase += 1;
    let next_word = get_boss_word(kind, wave_number, *boss_phase);
    virus.word = next_word.clone();
    *typing = TypingState::new(next_word);
    *active = true;
}

fn get_boss_word(kind: VirusKind, wave_number: u32, phase: usize) -> String {
    match kind {
        VirusKind::SummonerBoss => summoner_boss_word_for_phase(wave_number, phase).to_string(),
        VirusKind::ReverseBoss => reverse_boss_word_for_phase(wave_number, phase).to_string(),
        _ => boss_word_for_phase(wave_number, phase).to_string(),
    }
}

fn boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // Hard words pour renforcer l'identite boss
    WordList::pick(Difficulty::Hard, wave_number as usize + phase)
}

fn summoner_boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // On garde des mots difficiles pour le boss invocateur aussi.
    WordList::pick(Difficulty::Hard, wave_number as usize + SUMMONER_BOSS_SEED_OFFSET + phase)
}

fn reverse_boss_word_for_phase(wave_number: u32, phase: usize) -> &'static str {
    // Variante de seed pour limiter les répétitions avec les autres boss.
    WordList::pick(Difficulty::Hard, wave_number as usize + REVERSE_BOSS_SEED_OFFSET + phase)
}

fn reverse_boss_words_for_wave(wave_number: u32) -> usize {
    // 15 => 3 mots, 25 => 4, 35 => 5...
    REVERSE_BOSS_BASE_WORDS + ((wave_number.saturating_sub(15) / 10) as usize)
}

fn summoner_boss_cycles_for_wave(wave_number: u32) -> usize {
    // 10 => 2 cycles, 20 => 3, 30 => 4...
    SUMMONER_BOSS_BASE_CYCLES + ((wave_number.saturating_sub(10) / 10) as usize)
}

fn summoned_minion_word(wave_number: u32, cycle: usize, idx: usize) -> &'static str {
    let seed = wave_number as usize + cycle * SUMMONER_BOSS_MINIONS_PER_CYCLE + idx;
    WordList::pick(Difficulty::Easy, seed)
}
