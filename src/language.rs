use std::collections::HashSet;

use crate::normalize::NormalizedString;

pub(crate) enum Language {
    Portuguese,
    English,
}

impl Language {
    pub(crate) async fn get_words(&self) -> color_eyre::Result<Vec<String>> {
        let mut words = Vec::new();

        match self {
            Language::Portuguese => {
                let lista_de_palavras = reqwest::get("https://web.archive.org/web/20260403013752/http://200.17.137.109:8081/novobsi/Members/cicerog/disciplinas/introducao-a-programacao/arquivos-2015-2/algoritmos/Lista-de-Palavras.txt").await?.text().await?;
                let verbos = reqwest::get(
                    "https://raw.githubusercontent.com/fserb/pt-br/93ba2a6f3b2f85262fba72df09d448c6bb2fa50a/listas/verbos",
                )
                .await?
                .text()
                .await?;
                let icf = reqwest::get("https://raw.githubusercontent.com/fserb/pt-br/93ba2a6f3b2f85262fba72df09d448c6bb2fa50a/icf")
                    .await?
                    .text()
                    .await?;

                let sensible_word_list: HashSet<String> = lista_de_palavras
                    .trim()
                    .lines()
                    .map(|line| line.trim().to_lowercase())
                    .collect();
                let verbs: HashSet<String> = verbos
                    .trim()
                    .lines()
                    .map(|line| line.trim().to_lowercase())
                    .collect();

                for line in icf.trim().lines() {
                    let Some((word, score)) = line.split_once(',') else {
                        continue;
                    };
                    let Ok(score) = score.parse::<f32>() else {
                        continue;
                    };
                    if score >= 18.0 {
                        break;
                    }
                    let Ok(NormalizedString(string)) = word.parse() else {
                        continue;
                    };
                    if verbs.contains(&string)
                        || sensible_word_list.contains(&string.replace('ç', "c"))
                    {
                        words.push(string);
                    }
                }
            }

            Language::English => {
                let google_10000_english = reqwest::get("https://raw.githubusercontent.com/first20hours/google-10000-english/bdf4c221bc120b0b7f6c3f1eff1cc1abb975f8d8/google-10000-english-no-swears.txt")
                    .await?
                    .text()
                    .await?;
                for line in google_10000_english.trim().lines() {
                    let Ok(NormalizedString(string)) = line.parse() else {
                        continue;
                    };
                    words.push(string);
                }
            }
        }

        Ok(words)
    }
}
