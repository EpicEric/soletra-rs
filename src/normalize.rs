use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct NormalizedString(String);

impl FromStr for NormalizedString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: Result<String, ()> = s
            .trim()
            .chars()
            .map(|char| normalize_character(char).ok_or(()))
            .collect();
        inner.map(NormalizedString)
    }
}

impl AsRef<str> for NormalizedString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

pub(crate) fn normalize_character(char: char) -> Option<char> {
    match char.to_lowercase().next()? {
        'a' | 'á' | 'à' | 'ã' | 'â' => Some('a'),
        'b' => Some('b'),
        'c' => Some('c'),
        'ç' => Some('ç'),
        'd' => Some('d'),
        'e' | 'ê' | 'é' => Some('e'),
        'f' => Some('f'),
        'g' => Some('g'),
        'h' => Some('h'),
        'i' | 'í' => Some('i'),
        'j' => Some('j'),
        'l' => Some('l'),
        'm' => Some('m'),
        'n' => Some('n'),
        'o' | 'ó' | 'õ' | 'ô' => Some('o'),
        'p' => Some('p'),
        'q' => Some('q'),
        'r' => Some('r'),
        's' => Some('s'),
        't' => Some('t'),
        'u' | 'ú' | 'ü' => Some('u'),
        'v' => Some('v'),
        'x' => Some('x'),
        'z' => Some('z'),
        _ => None,
    }
}
