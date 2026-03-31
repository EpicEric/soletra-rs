use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::normalize::NormalizedString;

#[derive(Deserialize, Clone, Hash)]
pub(crate) struct Word {
    pub(crate) original: String,
    pub(crate) normalized: NormalizedString,
    pub(crate) is_pangram: bool,
}

#[derive(Deserialize, Clone, Hash)]
pub(crate) struct Game {
    pub(crate) main_letter: char,
    pub(crate) secondary_letters: [char; 6],
    pub(crate) words: Vec<Word>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ActiveGameWord {
    pub(crate) original: String,
    pub(crate) normalized: NormalizedString,
    pub(crate) is_pangram: bool,
    pub(crate) discovered: bool,
    pub(crate) points: u16,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ActiveGame {
    pub(crate) main_letter: char,
    pub(crate) original_secondary_letters: [char; 6],
    pub(crate) secondary_letters: [char; 6],
    pub(crate) points: u16,
    pub(crate) total_points: u16,
    pub(crate) words: Vec<ActiveGameWord>,
    pub(crate) found_words: usize,
}

pub(crate) enum BadGuess {
    AlreadyDiscovered,
    InvalidCharacters,
    WordNotInGame,
}

pub(crate) enum GuessResult {
    Success {
        points: u16,
        is_pangram: bool,
        is_game_over: bool,
    },
    Failure(BadGuess),
}

impl From<Word> for ActiveGameWord {
    fn from(value: Word) -> Self {
        let mut points = value.original.len() as u16;
        if value.original.len() > 5 {
            points += 2;
        }
        if value.is_pangram {
            points += 2;
        }
        ActiveGameWord {
            original: value.original,
            normalized: value.normalized,
            is_pangram: value.is_pangram,
            discovered: false,
            points,
        }
    }
}

impl From<Game> for ActiveGame {
    fn from(value: Game) -> Self {
        let words: Vec<ActiveGameWord> =
            value.words.into_iter().map(ActiveGameWord::from).collect();
        let total_points = words.iter().map(|word| word.points).sum();
        ActiveGame {
            main_letter: value.main_letter,
            original_secondary_letters: value.secondary_letters,
            secondary_letters: value.secondary_letters,
            words,
            found_words: 0,
            points: 0,
            total_points,
        }
    }
}

impl ActiveGame {
    pub(crate) fn shuffle(&mut self) {
        self.secondary_letters.shuffle(&mut rand::rng());
    }

    pub(crate) fn reset_shuffle(&mut self) {
        self.secondary_letters = self.original_secondary_letters;
    }

    pub(crate) fn guess(&mut self, guess: &str) -> GuessResult {
        let Ok(normalized_guess): Result<NormalizedString, _> = guess.parse() else {
            return GuessResult::Failure(BadGuess::InvalidCharacters);
        };
        if normalized_guess.as_ref().chars().any(|char| {
            char != self.main_letter || !self.original_secondary_letters.contains(&char)
        }) {
            return GuessResult::Failure(BadGuess::InvalidCharacters);
        }

        let Some(word) = self
            .words
            .iter_mut()
            .find(|word| word.normalized == normalized_guess)
        else {
            return GuessResult::Failure(BadGuess::WordNotInGame);
        };

        if word.discovered {
            GuessResult::Failure(BadGuess::AlreadyDiscovered)
        } else {
            word.discovered = true;
            self.points += word.points;
            self.found_words += 1;
            GuessResult::Success {
                points: word.points,
                is_pangram: word.is_pangram,
                is_game_over: self.found_words == self.words.len(),
            }
        }
    }
}
