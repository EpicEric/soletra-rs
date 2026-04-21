use std::fmt::Display;

use rand::seq::SliceRandom;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::normalize::NormalizedString;

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub(crate) struct Word {
    pub(crate) original: String,
    pub(crate) normalized: NormalizedString,
    pub(crate) is_pangram: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub(crate) struct Game {
    pub(crate) main_letter: char,
    pub(crate) secondary_letters: [char; 6],
    pub(crate) words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ActiveGameWord {
    pub(crate) original: String,
    pub(crate) normalized: NormalizedString,
    pub(crate) is_pangram: bool,
    pub(crate) discovered: bool,
    pub(crate) points: u16,
    #[serde(skip)]
    pub(crate) has_effect: bool,
}

#[derive(Serialize, Deserialize, Debug)]
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
    InvalidCharacters,
    TooShort,
    WordNotInGame,
    AlreadyDiscovered,
}

impl Display for BadGuess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                BadGuess::InvalidCharacters => t!("bad_guess.invalid_characters"),
                BadGuess::TooShort => t!("bad_guess.too_short"),
                BadGuess::WordNotInGame => t!("bad_guess.word_not_in_game"),
                BadGuess::AlreadyDiscovered => t!("bad_guess.already_discovered"),
            }
            .as_ref(),
        )
    }
}

pub(crate) enum GuessResult {
    Success {
        index: usize,
        points: u16,
        is_pangram: bool,
        is_game_over: bool,
    },
    Failure(BadGuess),
}

impl From<Word> for ActiveGameWord {
    fn from(value: Word) -> Self {
        let mut points = 1;
        if value.original.len() > 4 {
            points = value.original.len() as u16;
        }
        if value.is_pangram {
            points *= 2;
        }
        ActiveGameWord {
            original: value.original,
            normalized: value.normalized,
            is_pangram: value.is_pangram,
            discovered: false,
            points,
            has_effect: false,
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

        let mut count = 0usize;
        for char in normalized_guess.as_ref().chars() {
            if char != self.main_letter && !self.original_secondary_letters.contains(&char) {
                return GuessResult::Failure(BadGuess::InvalidCharacters);
            }
            count += 1;
        }
        if count < 4 {
            return GuessResult::Failure(BadGuess::TooShort);
        }

        let Some((index, word)) = self
            .words
            .iter_mut()
            .enumerate()
            .find(|(_, word)| word.normalized == normalized_guess)
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
                index,
                points: word.points,
                is_pangram: word.is_pangram,
                is_game_over: self.found_words >= self.words.len(),
            }
        }
    }
}
