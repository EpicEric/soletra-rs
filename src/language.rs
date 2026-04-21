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

const EN_ENABLE_1: &str = "https://norvig.com/ngrams/enable1.txt";
const EN_COUNT_1W: &str = "https://norvig.com/ngrams/count_1w.txt";

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
                        words.push(word.to_string());
                    }
                }
            }

            Language::English => {
                let count_1w = Compat::new(async {
                    match reqwest::get(EN_COUNT_1W).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
                .await?;
                let enable_1 = Compat::new(async {
                    match reqwest::get(EN_ENABLE_1).await {
                        Ok(request) => Ok(request.text().await?),
                        Err(error) => Err(color_eyre::Report::from(error)),
                    }
                })
                .await?;

                let sensible_word_list: HashSet<String> = enable_1
                    .trim()
                    .lines()
                    .map(|line| line.trim().to_lowercase())
                    .collect();

                for line in count_1w.trim().lines() {
                    let Some((word, freq)) = line.split_once('\t') else {
                        continue;
                    };
                    let Ok(freq) = freq.parse::<usize>() else {
                        continue;
                    };
                    if freq < 70_000 {
                        break;
                    }
                    let Ok(NormalizedString(string)) = word.parse() else {
                        continue;
                    };
                    if sensible_word_list.contains(&string) {
                        words.push(word.to_string());
                    }
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
            Language::English => "English",
        })
    }
}
