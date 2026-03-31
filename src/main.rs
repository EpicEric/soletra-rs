use serde::Deserialize;

#[derive(Deserialize)]
struct Word {
    original: String,
    normalized: String,
}

#[derive(Deserialize)]
struct Game {
    main_letter: char,
    secondary_letters: [char; 6],
    words: Vec<Word>,
}

fn main() {
    println!("Hello, world!");
}
