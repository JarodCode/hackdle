// Pas de serde/JSON pour l'instant — mots en dur, on ajoutera le chargement
// depuis des fichiers JSON à la Phase 3.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct WordList;

impl WordList {
    pub fn get(difficulty: Difficulty) -> &'static [&'static str] {
        match difficulty {
            Difficulty::Easy => EASY_WORDS,
            Difficulty::Medium => MEDIUM_WORDS,
            Difficulty::Hard => HARD_WORDS,
        }
    }

    // Sélectionne un mot aléatoire selon la difficulté
    // `index` est fourni par l'appelant — on n'utilise pas rand ici encore
    pub fn pick(difficulty: Difficulty, index: usize) -> &'static str {
        let list = Self::get(difficulty);
        list[index % list.len()]
    }
}

// Mots courts et simples — virus Fast et Classic en début de partie
static EASY_WORDS: &[&str] = &[
    "if", "let", "mut", "fn", "use", "mod", "pub", "for", "in",
    "loop", "move", "ref", "str", "vec", "map", "key", "val",
];

// Mots moyens — Classic et Heavy
static MEDIUM_WORDS: &[&str] = &[
    "match", "enum", "impl", "trait", "where", "super", "self",
    "async", "await", "spawn", "clone", "debug", "error", "panic",
    "stdin", "stack", "alloc", "token", "parse",
];

// Mots longs et complexes — Heavy et Boss
static HARD_WORDS: &[&str] = &[
    "borrowing", "lifetime", "ownership", "iteration", "recursive",
    "inference", "compiler", "serialize", "deserialize", "reference",
    "immutable", "concurrency", "allocation", "propagate", "implement",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_wraps_around() {
        let list = WordList::get(Difficulty::Easy);
        let len = list.len();
        // L'index modulo la longueur ne doit jamais dépasser les bornes
        assert_eq!(WordList::pick(Difficulty::Easy, 0), list[0]);
        assert_eq!(WordList::pick(Difficulty::Easy, len), list[0]);
        assert_eq!(WordList::pick(Difficulty::Easy, len + 1), list[1]);
    }

    #[test]
    fn all_difficulties_have_words() {
        assert!(!WordList::get(Difficulty::Easy).is_empty());
        assert!(!WordList::get(Difficulty::Medium).is_empty());
        assert!(!WordList::get(Difficulty::Hard).is_empty());
    }
}