use macroquad::prelude::*;

use crate::core::virus::{Virus, VirusKind};
use crate::core::input::TypingState;
use crate::data::words::{Difficulty, WordList};

pub const BOSS_EVERY_N_WAVES: u32 = 5;
const CLASSIC_BOSS_WORDS: usize = 4;
const REVERSE_BOSS_BASE_WORDS: usize = 3;
const SUMMONER_BOSS_FINAL_WORDS: usize = 1;
const SUMMONER_BOSS_BASE_CYCLES: usize = 2;
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
    if matches!(virus.kind, VirusKind::Boss) {
        if *boss_words_remaining > 1 {
            *boss_words_remaining -= 1;
            *boss_phase += 1;
            let next_word = boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
        } else {
            *boss_words_remaining = 0;
            virus.kill();
            *active = false;
        }
        return;
    }

    if matches!(virus.kind, VirusKind::ReverseBoss) {
        if *boss_words_remaining > 1 {
            *boss_words_remaining -= 1;
            *boss_phase += 1;
            let next_word = reverse_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
        } else {
            *boss_words_remaining = 0;
            virus.kill();
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

        // Le boss invocateur devient plus difficile avec +1 cycle tous les 10 niveaux.
        if *boss_spawn_cycles_done < summoner_boss_cycles_for_wave(wave_number) {
            *boss_spawn_cycles_done += 1;
            *boss_phase += 1;
            let next_word = summoner_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
            return;
        }

        if *boss_words_remaining > 1 {
            *boss_words_remaining -= 1;
            *boss_phase += 1;
            let next_word = summoner_boss_word_for_phase(wave_number, *boss_phase).to_string();
            virus.word = next_word.clone();
            *typing = TypingState::new(next_word);
            *active = true;
        } else {
            *boss_words_remaining = 0;
            virus.kill();
            *active = false;
        }
        return;
    }

    virus.kill();
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
