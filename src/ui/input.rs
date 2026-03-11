// Zéro import macroquad — logique pure, 100% testable

#[derive(Debug, PartialEq)]
pub enum TypingResult {
    Correct,   // bonne lettre, mot pas encore fini
    Wrong,     // mauvaise lettre
    Complete,  // mot entièrement tapé
}

pub struct TypingState {
    pub target: String,
    pub progress: usize, // nombre de lettres correctement tapées
}

impl TypingState {
    pub fn new(target: String) -> Self {
        Self { target, progress: 0 }
    }

    pub fn type_char(&mut self, c: char) -> TypingResult {
        let expected = self.target.chars().nth(self.progress);

        match expected {
            Some(e) if e == c => {
                self.progress += 1;
                if self.progress == self.target.len() {
                    TypingResult::Complete
                } else {
                    TypingResult::Correct
                }
            }
            _ => TypingResult::Wrong,
        }
    }

    // Portion du mot déjà tapée — utile pour le rendu (lettres vertes)
    pub fn typed_part(&self) -> &str {
        &self.target[..self.progress]
    }

    // Portion restante à taper
    pub fn remaining_part(&self) -> &str {
        &self.target[self.progress..]
    }

    pub fn reset(&mut self) {
        self.progress = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_char_advances_progress() {
        let mut state = TypingState::new("rust".to_string());
        assert_eq!(state.type_char('r'), TypingResult::Correct);
        assert_eq!(state.progress, 1);
    }

    #[test]
    fn wrong_char_does_not_advance() {
        let mut state = TypingState::new("rust".to_string());
        assert_eq!(state.type_char('x'), TypingResult::Wrong);
        assert_eq!(state.progress, 0);
    }

    #[test]
    fn completing_word_returns_complete() {
        let mut state = TypingState::new("hi".to_string());
        assert_eq!(state.type_char('h'), TypingResult::Correct);
        assert_eq!(state.type_char('i'), TypingResult::Complete);
    }

    #[test]
    fn typed_and_remaining_parts_are_correct() {
        let mut state = TypingState::new("rust".to_string());
        state.type_char('r');
        state.type_char('u');
        assert_eq!(state.typed_part(), "ru");
        assert_eq!(state.remaining_part(), "st");
    }

    #[test]
    fn reset_clears_progress() {
        let mut state = TypingState::new("rust".to_string());
        state.type_char('r');
        state.type_char('u');
        state.reset();
        assert_eq!(state.progress, 0);
    }
}