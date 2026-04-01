use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    MouseButton, MouseEventKind,
};
use rand::{
    SeedableRng,
    distr::{Distribution, Uniform},
    rngs::SmallRng,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Position, Rect},
    widgets::Paragraph,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, time::interval};
use tui_scrollview::ScrollViewState;

use crate::{
    game::{ActiveGame, Game, GuessResult},
    widgets::{GuessesWidget, HoneycombWidget, InputWidget},
};

const GAMES: &str = include_str!("games.json");
const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1_000 / FPS);

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) active_games: Vec<ActiveGame>,
    pub(crate) current_game: usize,
}

pub(crate) struct App {
    data: AppData,
    games: Option<Vec<Game>>,
    result: Option<(GuessResult, Instant)>,
    input: String,
    should_quit: bool,
    loading_games: bool,
    areas: AppAreas,
    scroll_view_state: ScrollViewState,
}

#[derive(Default)]
pub(crate) struct AppAreas {
    button_main: Rect,
    button_one: Rect,
    button_two: Rect,
    button_three: Rect,
    button_four: Rect,
    button_five: Rect,
    button_six: Rect,
    button_shuffle: Rect,
    button_reset_shuffle: Rect,
    button_clear: Rect,
    button_submit: Rect,
}

#[derive(Debug, Clone)]
pub(crate) enum AppEvent {
    Key(KeyEvent),
    Mouse(event::MouseEvent),
    GamesLoaded(Vec<Game>),
}

impl App {
    pub(crate) fn init() -> Self {
        App {
            data: AppData::default(),
            games: None,
            result: None,
            input: String::new(),
            should_quit: false,
            loading_games: false,
            areas: Default::default(),
            scroll_view_state: Default::default(),
        }
    }

    pub(crate) async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        crossterm::execute!(terminal.backend_mut(), EnableMouseCapture)?;
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn event handler task
        let event_tx = tx.clone();
        tokio::spawn(async move {
            event_handler(event_tx).await;
        });

        let mut frame_interval = interval(FRAME_DURATION);
        frame_interval.tick().await;

        loop {
            // Handle events
            while let Ok(event) = rx.try_recv() {
                self.handle_event(event)?;
                if self.should_quit {
                    crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    return Ok(());
                }
            }

            // Render terminal
            self.render(terminal, &tx)?;
            frame_interval.tick().await;
        }
    }

    fn render(
        &mut self,
        terminal: &mut DefaultTerminal,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) -> color_eyre::Result<()> {
        if self.data.current_game >= self.data.active_games.len() {
            match self.games.as_ref() {
                Some(games) => {
                    let index = {
                        let mut rng = SmallRng::seed_from_u64(42);
                        let range = Uniform::try_from(0..games.len())?;
                        for _ in range
                            .sample_iter(&mut rng)
                            .take(self.data.active_games.len())
                        {}
                        range.sample(&mut rng)
                    };
                    if let Some(game) = games.get(index) {
                        self.data.active_games.push(game.clone().into());
                        self.data.current_game = self.data.active_games.len() - 1;
                        let game = self
                            .data
                            .active_games
                            .get_mut(self.data.current_game)
                            .expect("length checked");
                        terminal.draw(|frame| {
                            App::render_game(
                                frame,
                                game,
                                &self.input,
                                &mut self.areas,
                                &mut self.scroll_view_state,
                            )
                        })?;
                    }
                }
                None => {
                    terminal.draw(|frame| App::render_loading(frame))?;

                    // Only spawn the loading task once
                    if !self.loading_games {
                        self.loading_games = true;
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let result = tokio::task::spawn_blocking(|| {
                                serde_json::from_str::<Vec<Game>>(GAMES)
                            })
                            .await;
                            if let Ok(Ok(games)) = result {
                                let _ = tx.send(AppEvent::GamesLoaded(games));
                            }
                        });
                    }
                }
            }
        } else {
            let game = self
                .data
                .active_games
                .get_mut(self.data.current_game)
                .expect("length checked");
            terminal.draw(|frame| {
                App::render_game(
                    frame,
                    game,
                    &self.input,
                    &mut self.areas,
                    &mut self.scroll_view_state,
                )
            })?;
        }
        Ok(())
    }

    fn render_game(
        frame: &mut Frame,
        game: &mut ActiveGame,
        input: &str,
        areas: &mut AppAreas,
        scroll_view_state: &mut ScrollViewState,
    ) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(17), Constraint::Fill(1)]).areas(frame.area());
        let [honeycomb_area, input_area] =
            Layout::vertical([Constraint::Length(9), Constraint::Length(3)]).areas(left_area);

        let mut honeycomb = HoneycombWidget {
            main_letter: game.main_letter,
            secondary_letters: game.secondary_letters,
            area_button_main: areas.button_main,
            area_button_one: areas.button_one,
            area_button_two: areas.button_two,
            area_button_three: areas.button_three,
            area_button_four: areas.button_four,
            area_button_five: areas.button_five,
            area_button_six: areas.button_six,
        };
        frame.render_widget(&mut honeycomb, honeycomb_area);
        areas.button_main = honeycomb.area_button_main;
        areas.button_one = honeycomb.area_button_one;
        areas.button_two = honeycomb.area_button_two;
        areas.button_three = honeycomb.area_button_three;
        areas.button_four = honeycomb.area_button_four;
        areas.button_five = honeycomb.area_button_five;
        areas.button_six = honeycomb.area_button_six;

        let input = InputWidget { input };
        frame.render_widget(input, input_area);

        let guesses = GuessesWidget {
            guesses: &game.words,
            scroll_view_state,
        };
        frame.render_widget(guesses, right_area);
    }

    fn render_loading(frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Carregando...").centered(), frame.area());
    }

    fn handle_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
        match event {
            AppEvent::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match (
                        key.code,
                        self.data.active_games.get_mut(self.data.current_game),
                    ) {
                        (KeyCode::Char('c'), _)
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            self.should_quit = true;
                        }
                        (KeyCode::Char('['), _) => {
                            self.scroll_view_state.set_offset(Position::new(0, 0));
                            self.input.clear();
                            self.result = None;
                            self.data.current_game = self.data.current_game.saturating_sub(1);
                        }
                        (KeyCode::Char(']'), _) => {
                            self.scroll_view_state.set_offset(Position::new(0, 0));
                            self.input.clear();
                            self.result = None;
                            self.data.current_game += 1;
                        }
                        (KeyCode::Char(c), Some(_)) => {
                            self.input.push(c);
                        }
                        (KeyCode::Backspace, Some(_)) => {
                            self.input.pop();
                        }
                        (KeyCode::Enter, Some(game)) => {
                            let result = game.guess(&self.input);
                            self.result = Some((result, Instant::now()));
                            self.input.clear();
                        }
                        (KeyCode::Right, _) => {
                            self.scroll_view_state.scroll_right();
                        }
                        (KeyCode::Left, _) => {
                            self.scroll_view_state.scroll_left();
                        }
                        _ => {}
                    }
                }
            }
            AppEvent::Mouse(mouse) => {
                if let Some(game) = self.data.active_games.get_mut(self.data.current_game)
                    && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                {
                    let position = Position::new(mouse.column, mouse.row);
                    if self.areas.button_main.contains(position) {
                        self.input.push(game.main_letter);
                    }
                    if self.areas.button_one.contains(position) {
                        self.input.push(game.secondary_letters[0]);
                    }
                    if self.areas.button_two.contains(position) {
                        self.input.push(game.secondary_letters[1]);
                    }
                    if self.areas.button_three.contains(position) {
                        self.input.push(game.secondary_letters[2]);
                    }
                    if self.areas.button_four.contains(position) {
                        self.input.push(game.secondary_letters[3]);
                    }
                    if self.areas.button_five.contains(position) {
                        self.input.push(game.secondary_letters[4]);
                    }
                    if self.areas.button_six.contains(position) {
                        self.input.push(game.secondary_letters[5]);
                    }
                    if self.areas.button_shuffle.contains(position) {
                        game.shuffle();
                    }
                    if self.areas.button_reset_shuffle.contains(position) {
                        game.reset_shuffle();
                    }
                    if self.areas.button_clear.contains(position) {
                        self.input.clear();
                    }
                    if self.areas.button_submit.contains(position) {
                        let result = game.guess(&self.input);
                        self.result = Some((result, Instant::now()));
                        self.input.clear();
                    }
                }
            }
            AppEvent::GamesLoaded(games) => {
                self.games = Some(games);
                self.loading_games = false;
            }
        }
        Ok(())
    }
}

async fn event_handler(tx: mpsc::UnboundedSender<AppEvent>) {
    loop {
        if event::poll(Duration::from_millis(10)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(Event::Mouse(mouse)) => {
                    if tx.send(AppEvent::Mouse(mouse)).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
