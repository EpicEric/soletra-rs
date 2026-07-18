// soletra-rs: TUI version of the game Soletra/Spelling Bee
// Copyright (C) 2026 Eric Rodrigues Pires
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for
// more details.
//
// You should have received a copy of the GNU Affero General Public License along
// with this program. If not, see <https://www.gnu.org/licenses/>.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct NormalizedString(pub(crate) String);

impl FromStr for NormalizedString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: Result<String, ()> = s
            .trim()
            .chars()
            .map(|char| normalize_character(char).ok_or(()))
            .collect();
        inner.map(Self)
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
