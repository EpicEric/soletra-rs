import json
import locale
import os
from collections.abc import Generator
from pathlib import Path
from typing import Any

from pydantic import BaseModel
from tqdm import tqdm

LETTERS = {
    "a": "aáàãâ",
    "b": "b",
    "c": "c",
    "ç": "ç",
    "d": "d",
    "e": "eêé",
    "f": "f",
    "g": "g",
    "h": "h",
    "i": "ií",
    "j": "j",
    "l": "l",
    "m": "m",
    "n": "n",
    "o": "oóõô",
    "p": "p",
    "q": "q",
    "r": "r",
    "s": "s",
    "t": "t",
    "u": "uúü",
    "v": "v",
    "x": "x",
    "z": "z",
}

LETTERS_INV = {
    letter: norm for norm, letter_list in LETTERS.items() for letter in letter_list
}

MAX_CHARACTERS = 19


class Word(BaseModel):
    original: str
    normalized: str
    is_pangram: bool


class Game(BaseModel):
    main_letter: str
    secondary_letters: list[str]
    words: list[Word]


def generate_pt_words() -> Generator[tuple[Word, set[str]], Any, None]:
    MAX_SCORE = 18.0

    print("Loading pt-br/listas/verbos...")
    with open(Path(os.environ["pt-br"]).joinpath("listas/verbos")) as f:
        verbs = {line.lower().strip() for line in f}

    print("Loading lista-de-palavras.txt...")
    with open(Path(os.environ["lista-de-palavras"])) as f:
        sensible_word_list = {line.lower().strip() for line in f}

    print("Loading pt-br/icf...")
    with open(Path(os.environ["pt-br"]).joinpath("icf")) as f:
        for line in f:
            word_original, score = line.split(",", 1)
            if float(score) >= MAX_SCORE:
                return
            letters_word: set[str] = set()
            word_normalized: str | None = ""
            for letter in word_original.lower():
                if (normalized_letter := LETTERS_INV.get(letter)) is None:
                    word_normalized = None
                    break
                word_normalized += normalized_letter
                letters_word.add(normalized_letter)
            if (
                word_normalized
                and len(letters_word) <= 7
                and len(word_original) >= 4
                and len(word_original) <= MAX_CHARACTERS
                and (
                    word_original in verbs
                    or word_normalized.lower().replace("ç", "c") in sensible_word_list
                )
            ):
                yield (
                    Word(
                        original=word_original,
                        normalized=word_normalized,
                        is_pangram=len(letters_word) == 7,
                    ),
                    letters_word,
                )


def generate_en_words() -> Generator[tuple[Word, set[str]], Any, None]:
    print("Loading google-10000-english/google-10000-english-no-swears.txt...")
    with open(
        Path(os.environ["google-10000-english"]).joinpath(
            "google-10000-english-no-swears.txt"
        )
    ) as f:
        for line in f:
            word = line.strip()
            letters_word: set[str] = set()
            word_normalized: str | None = ""
            for letter in word.lower():
                if (normalized_letter := LETTERS_INV.get(letter)) is None:
                    word_normalized = None
                    break
                word_normalized += normalized_letter
                letters_word.add(normalized_letter)
            if (
                word_normalized
                and len(letters_word) <= 7
                and len(word) >= 4
                and len(word) <= MAX_CHARACTERS
            ):
                yield (
                    Word(
                        original=word,
                        normalized=word_normalized,
                        is_pangram=len(letters_word) == 7,
                    ),
                    letters_word,
                )


def main():
    language = os.environ["language"]
    if language == "pt":
        locale.setlocale(locale.LC_ALL, "pt_BR.UTF-8")
        generator = generate_pt_words()
    elif language == "en":
        locale.setlocale(locale.LC_ALL, "en_US.UTF-8")
        generator = generate_en_words()
    else:
        raise ValueError(f"Unknown language '{language}'. Valid values are: 'pt', 'en'")
    print(f"Selected language: {language}")

    # TODO: Use better structure for lookup
    words: list[Word] = []
    pangrams: set[str] = set()
    for word, letters_word in generator:
        words.append(word)
        if word.is_pangram:
            pangrams.add("".join(sorted(letters_word, key=locale.strxfrm)))

    print(f"Found {len(words)} words and {len(pangrams)} playable letter combinations!")

    games: list[Game] = []
    print("Populating games...")
    for pangram in tqdm(sorted(pangrams, key=locale.strxfrm)):
        letters_pangram: dict[str, dict[str, list[Word]]] = {
            letter: {} for letter in pangram
        }

        for word in words:
            if all(letter in letters_pangram for letter in word.normalized):
                for letter in word.normalized:
                    letters_pangram[letter].setdefault(word.normalized, []).append(word)

        for letter, words_dict in sorted(
            letters_pangram.items(), key=lambda x: locale.strxfrm(x[0])
        ):
            words_game: list[Word] = []
            for normalized, words_list in sorted(
                words_dict.items(), key=lambda x: (len(x[0]), locale.strxfrm(x[0]))
            ):
                words_game.append(
                    Word(
                        original="/".join(
                            sorted(
                                set(p.original for p in words_list),
                                key=locale.strxfrm,
                            )
                        ),
                        normalized=normalized,
                        is_pangram=words_list[0].is_pangram,
                    )
                )
            games.append(
                Game(
                    main_letter=letter,
                    secondary_letters=[
                        letter_pangram
                        for letter_pangram in pangram
                        if letter_pangram != letter
                    ],
                    words=words_game,
                )
            )

    print(f"Exporting {len(games)} games...")
    with open(os.environ["out"], "w") as file_out:
        json.dump([game.model_dump(mode="json") for game in games], file_out)

    print("Done.")


if __name__ == "__main__":
    main()
