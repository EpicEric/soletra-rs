use rand::{RngExt, SeedableRng, rngs::SmallRng};
use ratatui::{DefaultTerminal, Frame, style::Stylize, text::Line, widgets::Paragraph};
use serde::{Deserialize, Serialize};

use crate::game::{ActiveGame, Game};

const GAMES: &str = include_str!("games.json");

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) active_games: Vec<ActiveGame>,
    pub(crate) current_game: usize,
}

pub(crate) struct App {
    data: AppData,
    games: Option<Vec<Game>>,
}

impl App {
    pub(crate) fn init() -> Self {
        App {
            data: AppData::default(),
            games: None,
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            if self.data.current_game >= self.data.active_games.len() {
                match self.games.as_mut() {
                    Some(games) => {
                        self.data.current_game = self.data.active_games.len();
                        let index = {
                            let mut rng = SmallRng::seed_from_u64(42);
                            let _ = (&mut rng).random_iter::<u64>().take(self.data.current_game);
                            rng.random_range(0..games.len())
                        };
                        if let Some(game) = games.get(index) {
                            terminal.draw(|frame| App::render_loading(frame))?;
                            self.data.active_games.push(game.clone().into());
                        } else {
                            terminal.draw(|frame| App::render_no_more_games(frame))?;
                        }
                    }
                    None => {
                        terminal.draw(|frame| App::render_loading(frame))?;
                        let games: Vec<Game> = serde_json::from_str(GAMES)?;
                        self.games = Some(games);
                    }
                }
            } else {
                let game = self
                    .data
                    .active_games
                    .get_mut(self.data.current_game)
                    .expect("length checked");
                terminal.draw(|frame| App::render_game(frame, game))?;
            }
            self.handle_events()?;
        }
    }

    fn render_game(frame: &mut Frame, game: &mut ActiveGame) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                game.main_letter.bold(),
                game.secondary_letters
                    .into_iter()
                    .collect::<String>()
                    .not_bold(),
            ])),
            frame.area(),
        );
    }

    fn render_loading(frame: &mut Frame) {
        frame.render_widget("Carregando...", frame.area());
    }

    fn render_no_more_games(frame: &mut Frame) {
        frame.render_widget("Você terminou todos os jogos. Parabéns!", frame.area());
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        Ok(())
    }
}
