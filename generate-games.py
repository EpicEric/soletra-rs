import json
import locale
import os
from pathlib import Path

from pydantic import BaseModel
from tqdm import tqdm

MAX_SCORE = 16.0

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


class Word(BaseModel):
    original: str
    normalized: str
    is_pangram: bool


class Game(BaseModel):
    main_letter: str
    secondary_letters: list[str]
    words: list[Word]


def main():
    locale.setlocale(locale.LC_ALL, "pt_BR.UTF-8")

    print("Loading pt-br/listas/verbos...")
    with open(Path(os.environ["src"]).joinpath("listas/verbos")) as f:
        verbs = {line for line in f}

    print("Loading pt-br/conjugações...")
    with open(Path(os.environ["src"]).joinpath("conjugações")) as f:
        conjugations = {line for line in f if line not in verbs}

    words: list[Word] = []
    pangrams: set[str] = set()
    print("Loading pt-br/icf...")
    with open(Path(os.environ["src"]).joinpath("icf")) as f:
        for line in f:
            word_original, score = line.split(",", 1)
            if float(score) >= MAX_SCORE:
                break
            if word_original in conjugations:
                continue
            letters_word: set[str] = set()
            word_normalized: str | None = ""
            for letter in word_original:
                if (normalized_letter := LETTERS_INV.get(letter)) is None:
                    word_normalized = None
                    break
                word_normalized += normalized_letter
                letters_word.add(normalized_letter)
            if word_normalized and len(letters_word) <= 7 and len(word_original) >= 4:
                words.append(
                    Word(
                        original=word_original,
                        normalized=word_normalized,
                        is_pangram=len(letters_word) == 7,
                    )
                )
                if len(letters_word) == 7:
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
                words_dict.items(), key=lambda x: x[0]
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
