use ratatui::{DefaultTerminal, Frame};
use serde::{Deserialize, Serialize};

use crate::game::{ActiveGame, Game};

const GAMES: &str = include_str!("games.json");

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppState {
    pub(crate) previous_games: Vec<ActiveGame>,
    pub(crate) current_game: Option<ActiveGame>,
}

pub(crate) struct App {
    pub(crate) state: AppState,
    pub(crate) games: Option<Vec<Game>>,
}

impl App {
    pub(crate) fn init() -> Self {
        App {
            state: AppState::default(),
            games: None,
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            match self.state.current_game.as_mut() {
                Some(game) => {
                    terminal.draw(|frame| App::render_game(frame, game))?;
                    self.handle_events()?;
                }
                None => match self.games.as_ref() {
                    Some(games) => {
                        if let Some(game) = games.first() {
                            terminal.draw(|frame| App::render_loading(frame))?;
                            self.state.current_game = Some(game.clone().into());
                        } else {
                            terminal.draw(|frame| App::render_no_more_games(frame))?;
                        }
                    }
                    None => {
                        terminal.draw(|frame| App::render_loading(frame))?;
                        self.games = serde_duper::from_string(GAMES)?;
                    }
                },
            }
        }
    }

    fn render_game(frame: &mut Frame, game: &mut ActiveGame) {
        frame.render_widget("Jogo carregado!", frame.area());
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
