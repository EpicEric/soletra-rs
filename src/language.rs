use std::{collections::HashSet, fmt::Display, str::FromStr};

use async_compat::Compat;
use color_eyre::eyre::eyre;

use crate::normalize::NormalizedString;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Language {
    Portuguese,
    English,
}

const PT_LISTA_DE_PALAVRAS: &str = "https://web.archive.org/web/20260403013752/http://200.17.137.109:8081/novobsi/Members/cicerog/disciplinas/introducao-a-programacao/arquivos-2015-2/algoritmos/Lista-de-Palavras.txt";
const PT_VERBOS: &str = "https://raw.githubusercontent.com/fserb/pt-br/93ba2a6f3b2f85262fba72df09d448c6bb2fa50a/listas/verbos";
const PT_ICF: &str =
    "https://raw.githubusercontent.com/fserb/pt-br/93ba2a6f3b2f85262fba72df09d448c6bb2fa50a/icf";

const EN_GOOGLE_10000: &str = "https://raw.githubusercontent.com/first20hours/google-10000-english/bdf4c221bc120b0b7f6c3f1eff1cc1abb975f8d8/google-10000-english-no-swears.txt";

impl Language {
    pub(crate) fn shortcode(&self) -> &'static str {
        match self {
            Language::Portuguese => "pt",
            Language::English => "en",
        }
    }

    pub(crate) async fn get_words(&self) -> color_eyre::Result<Vec<String>> {
        let mut words = Vec::new();

        match self {
            Language::Portuguese => {
                let lista_de_palavras = Compat::new(async {
                    match reqwest::get(PT_LISTA_DE_PALAVRAS).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
                .await?;
                let verbos = Compat::new(async {
                    match reqwest::get(PT_VERBOS).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
                .await?;
                let icf = Compat::new(async {
                    match reqwest::get(PT_ICF).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
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
                let google_10000_english = Compat::new(async {
                    match reqwest::get(EN_GOOGLE_10000).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
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

impl FromStr for Language {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pt" => Ok(Language::Portuguese),
            "en" => Ok(Language::English),
            unknown => Err(eyre!("Unknown language {unknown}")),
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Language::Portuguese => "Português",
            Language::English => "English (WIP)",
        })
    }
}
