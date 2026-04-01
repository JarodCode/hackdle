use crate::core::virus::VirusKind;
use crate::ui::input::TypingState;
use crate::data::words::{Difficulty, WordList};

// --- Constantes ---

pub const BOSS_EVERY_N_WAVES: u32 = 5;
const CLASSIC_BOSS_WORDS: usize = 4;
const REVERSE_BOSS_BASE_WORDS: usize = 3;
const SUMMONER_BOSS_BASE_CYCLES: usize = 2;
const SUMMONER_BOSS_SEED_OFFSET: usize = 17;
const REVERSE_BOSS_SEED_OFFSET: usize = 31;
const SUMMONER_BOSS_MINIONS_PER_CYCLE: usize = 4;
pub const SUMMONER_RADIUS: f32 = 140.0;

// etats des boss chaque boss à ses propres états
pub struct BossState {
    pub phase: usize,
    pub words_remaining: usize,
}

pub struct SummonerState {
    pub phase: usize,
    pub spawn_cycles_done: usize,
}

pub struct ReverseBossState {
    pub phase: usize,
    pub words_remaining: usize,
}

pub enum VirusBehavior {
    Normal, // Virus ordinaire (Fast, Classic, Heavy)
    Boss(BossState), // Boss classique multi-mots
    SummonerBoss(SummonerState), // Boss invocateur : cycles de sbires + mot final
    ReverseBoss(ReverseBossState), // Boss inversé : mots à taper à l'envers
}

impl VirusBehavior {
    /// Construit le bon variant selon le VirusKind et la vague.
    /// Appelé à la création de chaque VirusEntry.
    pub fn for_kind(kind: VirusKind, wave_number: u32) -> Self {
        match kind {
            VirusKind::Boss => Self::Boss(BossState {
                phase: 0,
                words_remaining: CLASSIC_BOSS_WORDS,
            }),
            VirusKind::SummonerBoss => Self::SummonerBoss(SummonerState {
                phase: 0,
                spawn_cycles_done: 0,
            }),
            VirusKind::ReverseBoss => Self::ReverseBoss(ReverseBossState {
                phase: 0,
                words_remaining: reverse_boss_words_for_wave(wave_number),
            }),
            // Fast, Classic, Heavy
            _ => Self::Normal,
        }
    }

    /// Retourne le mot visuel affiché au-dessus du virus.
    /// Pour ReverseBoss, le mot est affiché à l'envers.
    pub fn visual_word<'a>(&self, word: &'a str) -> std::borrow::Cow<'a, str> {
        match self {
            Self::ReverseBoss(_) => word.chars().rev().collect::<String>().into(),
            _ => word.into(),
        }
    }
}

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

/// Premier mot attribué au boss à sa création.
pub fn first_boss_word(wave_number: u32, kind: VirusKind) -> String {
    get_boss_word(kind, wave_number, 0)
}

/// Construction des sbires convoqués par le SummonerBoss.
/// Retourne une liste de (position, mot) prête à être transformée en VirusEntry.
pub fn build_summoned_minions(
    wave_number: u32,
    cycle: usize,
    center: macroquad::prelude::Vec2,
) -> Vec<(macroquad::prelude::Vec2, String)> {
    let mut minions = Vec::with_capacity(SUMMONER_BOSS_MINIONS_PER_CYCLE);
    for i in 0..SUMMONER_BOSS_MINIONS_PER_CYCLE {
        let angle = (i as f32 / SUMMONER_BOSS_MINIONS_PER_CYCLE as f32) * std::f32::consts::TAU;
        let dir = macroquad::prelude::Vec2::new(angle.cos(), angle.sin());
        let position = center + dir * SUMMONER_RADIUS;
        let word = summoned_minion_word(wave_number, cycle, i).to_string();
        minions.push((position, word));
    }
    minions
}

/// Résultat d'une complétion de mot pour le SummonerBoss.
pub enum SummonerResult {
    /// Le boss convoque une nouvelle vague de sbires (cycle n).
    SpawnMinions(usize),
    /// Le boss passe au mot suivant sans convoquer.
    NextWord,
    /// Le boss est mort.
    Killed,
}

/// Appelé quand un mot de boss classique (Boss ou ReverseBoss) est complété.
/// Retourne true si le boss est mort, false si le boss passe au mot suivant.
pub fn on_boss_word_complete(
    state: &mut BossState,
    wave_number: u32,
    kind: VirusKind,
    word_out: &mut String,
    typing_out: &mut TypingState,
) -> bool {
    if state.words_remaining > 1 {
        state.words_remaining -= 1;
        state.phase += 1;
        let next = get_boss_word(kind, wave_number, state.phase);
        *word_out = next.clone();
        *typing_out = TypingState::new(next);
        false // encore vivant
    } else {
        state.words_remaining = 0;
        true // mort
    }
}

/// Idem pour ReverseBoss (même logique, variant séparé pour extensibilité future).
pub fn on_reverse_boss_word_complete(
    state: &mut ReverseBossState,
    wave_number: u32,
    word_out: &mut String,
    typing_out: &mut TypingState,
) -> bool {
    if state.words_remaining > 1 {
        state.words_remaining -= 1;
        state.phase += 1;
        let next = get_boss_word(VirusKind::ReverseBoss, wave_number, state.phase);
        *word_out = next.clone();
        *typing_out = TypingState::new(next);
        false
    } else {
        state.words_remaining = 0;
        true
    }
}

/// Appelé quand le mot du SummonerBoss est complété.
pub fn on_summoner_word_complete(
    state: &mut SummonerState,
    wave_number: u32,
    has_alive_summoned: bool,
    word_out: &mut String,
    typing_out: &mut TypingState,
) -> SummonerResult {
    if has_alive_summoned {
        // Invincible tant qu'un sbire est en vie — on reset juste le mot
        advance_summoner_word(state, wave_number, word_out, typing_out);
        SummonerResult::NextWord
    } else if state.spawn_cycles_done < summoner_boss_cycles_for_wave(wave_number) {
        state.spawn_cycles_done += 1;
        advance_summoner_word(state, wave_number, word_out, typing_out);
        SummonerResult::SpawnMinions(state.spawn_cycles_done)
    } else {
        // Tous les cycles épuisés + plus de sbires → mort
        SummonerResult::Killed
    }
}


fn advance_summoner_word(
    state: &mut SummonerState,
    wave_number: u32,
    word_out: &mut String,
    typing_out: &mut TypingState,
) {
    state.phase += 1;
    let next = get_boss_word(VirusKind::SummonerBoss, wave_number, state.phase);
    *word_out = next.clone();
    *typing_out = TypingState::new(next);
}

fn get_boss_word(kind: VirusKind, wave_number: u32, phase: usize) -> String {
    match kind {
        VirusKind::SummonerBoss => {
            WordList::pick(Difficulty::Hard, wave_number as usize + SUMMONER_BOSS_SEED_OFFSET + phase).to_string()
        }
        VirusKind::ReverseBoss => {
            WordList::pick(Difficulty::Hard, wave_number as usize + REVERSE_BOSS_SEED_OFFSET + phase).to_string()
        }
        _ => {
            WordList::pick(Difficulty::Hard, wave_number as usize + phase).to_string()
        }
    }
}

fn reverse_boss_words_for_wave(wave_number: u32) -> usize {
    REVERSE_BOSS_BASE_WORDS + (wave_number.saturating_sub(15) / 10) as usize
}

fn summoner_boss_cycles_for_wave(wave_number: u32) -> usize {
    SUMMONER_BOSS_BASE_CYCLES + (wave_number.saturating_sub(10) / 10) as usize
}

fn summoned_minion_word(wave_number: u32, cycle: usize, idx: usize) -> &'static str {
    let seed = wave_number as usize + cycle * SUMMONER_BOSS_MINIONS_PER_CYCLE + idx;
    WordList::pick(Difficulty::Easy, seed)
}