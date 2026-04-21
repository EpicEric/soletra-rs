use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use feruca::Collator;
use itertools::Itertools;

use crate::{
    app::MAX_CHARACTERS,
    game::{Game, Word},
    normalize::{NormalizedString, normalize_character},
};

pub(crate) fn generate_games(words: Vec<String>) -> color_eyre::Result<Vec<Game>> {
    let mut collator = Collator::default();

    let mut words_map: HashMap<Vec<char>, HashMap<NormalizedString, (Vec<String>, bool)>> =
        HashMap::new();
    let mut pangrams: HashSet<String> = HashSet::new();

    let mut probe_bytes = [0u8; 4];
    let mut normalized_bytes = [0u8; 4];

    'words: for word in words.into_iter() {
        let mut normalized_characters = Vec::<char>::with_capacity(7);
        let mut normalized_word = String::new();
        let mut count = 0;
        for char in word.chars() {
            let Some(normalized) = normalize_character(char) else {
                continue 'words;
            };
            count += 1;
            if count > MAX_CHARACTERS {
                continue 'words;
            }
            normalized_word.push(normalized);
            let normalized_str = normalized.encode_utf8(&mut normalized_bytes);
            match normalized_characters.binary_search_by(|probe| {
                let probe_str = probe.encode_utf8(&mut probe_bytes);
                collator.collate(probe_str, normalized_str)
            }) {
                Ok(_) => {}
                Err(index) => {
                    if normalized_characters.len() >= 7 {
                        continue 'words;
                    } else {
                        normalized_characters.insert(index, normalized);
                    }
                }
            }
        }
        if count < 4 {
            continue 'words;
        }

        let is_pangram = normalized_characters.len() == 7;
        if is_pangram {
            pangrams.insert(normalized_characters.iter().collect());
        }

        let normalized_word_vec = &mut words_map
            .entry(normalized_characters)
            .or_default()
            .entry(NormalizedString(normalized_word.clone()))
            .or_insert_with(|| (vec![], is_pangram))
            .0;
        match normalized_word_vec
            .binary_search_by(|existing_word| collator.collate(existing_word, &word))
        {
            Ok(_) => {}
            Err(index) => normalized_word_vec.insert(index, word),
        }
    }

    let mut pangrams: Vec<String> = pangrams.into_iter().collect();
    pangrams.sort_by(|a, b| collator.collate(a, b));
    let mut games = Vec::with_capacity(pangrams.len() * 7);

    for pangram in pangrams {
        for i in 0..7 {
            let mut main_letter = '\0';
            let mut secondary_letters = ['\0'; 6];

            for (j, char) in pangram.chars().enumerate() {
                if i == j {
                    main_letter = char;
                } else {
                    secondary_letters[if j > i { j - 1 } else { j }] = char;
                }
            }

            debug_assert!(main_letter != '\0');
            debug_assert!(secondary_letters.iter().all(|char| char != &'\0'));

            let mut game = Game {
                main_letter: main_letter,
                secondary_letters: secondary_letters,
                words: Vec::new(),
            };

            for j in 0b000_001..=0b111_111 {
                let mut key = Vec::new();
                for (mut k, char) in pangram.chars().enumerate() {
                    if k == i {
                        key.push(char);
                    } else {
                        if k > i {
                            k -= 1;
                        }
                        if j & (1 << k) != 0 {
                            key.push(char)
                        }
                    }
                }

                if let Some(words_map) = words_map.get(&key) {
                    for (curr_key, curr_value) in words_map.iter() {
                        let curr_len = curr_key.0.len();
                        let Err(index) = game.words.binary_search_by(|game_word| {
                            match game_word.normalized.0.len().cmp(&curr_len) {
                                Ordering::Equal => {
                                    collator.collate(&game_word.normalized.0, &curr_key.0)
                                }
                                ordering => ordering,
                            }
                        }) else {
                            unreachable!();
                        };
                        game.words.insert(
                            index,
                            Word {
                                original: curr_value.0.iter().join("/"),
                                normalized: curr_key.clone(),
                                is_pangram: curr_value.1,
                            },
                        );
                    }
                }
            }

            games.push(game);
        }
    }

    Ok(games)
}
