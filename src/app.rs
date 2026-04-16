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
    layout::{Constraint, Layout, Offset, Position, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Gauge, Paragraph, Widget},
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{fs, sync::mpsc, time::interval};
use tui_scrollview::ScrollViewState;

use crate::{
    game::{ActiveGame, Game, GuessResult},
    widgets::{
        ActionsWidget, GameOverWidget, GuessResultWidget, GuessesWidget, HoneycombWidget,
        InputWidget, InputWidgetState,
    },
};

const GAMES: &str = include_str!("games.json");
const FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1_000 / FPS);
const APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
    name: "SoletraRs",
    author: "EpicEric",
};
const SAVE_DATA: &str = concat!(env!("SOLETRA_RS_LANGUAGE"), ".json");
const MAX_CHARACTERS: usize = 19;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) active_games: Vec<ActiveGame>,
    pub(crate) current_game: usize,
    #[serde(skip)]
    pub(crate) app_dir: Option<PathBuf>,
}

pub(crate) struct App {
    data: AppData,
    games: Option<Vec<Game>>,
    result: Option<(GuessResult, Instant)>,
    game_over: Option<Instant>,
    input: String,
    should_quit: bool,
    loading_games: bool,
    areas: AppAreas,
    rows: usize,
    scroll_view_state: ScrollViewState,
    guess_result_state: tui_overlay::OverlayState,
    game_over_state: tui_overlay::OverlayState,
    effects: tachyonfx::EffectManager<()>,
    elapsed: Duration,
}

#[derive(Default)]
pub(crate) struct AppAreas {
    pub(crate) button_main: Rect,
    pub(crate) button_one: Rect,
    pub(crate) button_two: Rect,
    pub(crate) button_three: Rect,
    pub(crate) button_four: Rect,
    pub(crate) button_five: Rect,
    pub(crate) button_six: Rect,
    pub(crate) button_shuffle: Rect,
    pub(crate) button_reset_shuffle: Rect,
    pub(crate) button_backspace: Rect,
    pub(crate) button_submit: Rect,
}

#[derive(Debug, Clone)]
pub(crate) enum AppEvent {
    Key(KeyEvent),
    Mouse(event::MouseEvent),
    GamesLoaded(Vec<Game>),
}

impl AppData {
    async fn init() -> color_eyre::Result<Self> {
        let dir = app_dirs2::get_app_root(app_dirs2::AppDataType::UserData, &APP_INFO)?;
        let save_path = dir.join(SAVE_DATA);
        if save_path.exists() && save_path.is_file() {
            let data: AppData = serde_json::from_slice(&fs::read(&save_path).await?)?;
            Ok(AppData {
                app_dir: Some(dir),
                ..data
            })
        } else {
            Ok(AppData {
                app_dir: Some(dir),
                ..Default::default()
            })
        }
    }

    async fn save(&self) -> color_eyre::Result<()> {
        if let Some(dir) = self.app_dir.as_ref() {
            let save_path = dir.join(SAVE_DATA);
            fs::create_dir_all(dir).await?;
            fs::write(&save_path, serde_json::to_vec(self)?).await?;
        }
        Ok(())
    }
}

impl App {
    pub(crate) async fn init() -> Self {
        App {
            data: AppData::init().await.unwrap_or_default(),
            games: None,
            result: None,
            game_over: None,
            input: String::new(),
            should_quit: false,
            loading_games: false,
            areas: Default::default(),
            rows: 1,
            scroll_view_state: Default::default(),
            guess_result_state: tui_overlay::OverlayState::new()
                .with_duration(Duration::from_millis(150))
                .with_easing(tui_overlay::Easing::EaseInOut),
            game_over_state: tui_overlay::OverlayState::new()
                .with_duration(Duration::from_millis(150))
                .with_easing(tui_overlay::Easing::EaseInOut),
            effects: tachyonfx::EffectManager::default(),
            elapsed: Duration::ZERO,
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
        let mut prev = frame_interval.tick().await;

        loop {
            // Handle events
            while let Ok(event) = rx.try_recv() {
                self.handle_event(event).await?;
                if self.should_quit {
                    crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    return Ok(());
                }
            }

            // Render terminal
            self.render(terminal, &tx).await?;
            let curr = frame_interval.tick().await;
            self.elapsed = curr.duration_since(prev);
            self.guess_result_state.tick(self.elapsed);
            self.game_over_state.tick(self.elapsed);
            if let Some(game_over) = self.game_over
                && curr.into_std() >= game_over
            {
                self.game_over_state.open();
                self.game_over = None;
            }
            prev = curr;
        }
    }

    async fn render(
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
                        let current_game = self.data.active_games.len() - 1;
                        self.data.current_game = current_game;
                        self.data.save().await?;
                        terminal.draw(|frame| self.render_game(frame))?;
                    }
                }
                None => {
                    terminal.draw(App::render_loading)?;

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
            terminal.draw(|frame| self.render_game(frame))?;
        }
        Ok(())
    }

    fn render_game(&mut self, frame: &mut Frame) {
        let game = self
            .data
            .active_games
            .get_mut(self.data.current_game)
            .expect("length checked");
        let guess_result_state = &mut self.guess_result_state;
        let game_over_state = &mut self.game_over_state;
        let is_game_over = game_over_state.is_open();

        let mut soletra_frame = Block::bordered()
            .border_type(BorderType::Thick)
            .title_top(Line::from(" soletra-rs ").centered())
            .title_bottom(
                Line::from(t!("game_number", game => self.data.current_game + 1)).centered(),
            )
            .title_bottom(Line::from(t!("next_game").reversed()).right_aligned());
        if self.data.current_game > 0 {
            soletra_frame = soletra_frame
                .title_bottom(Line::from(t!("previous_game").reversed()).left_aligned());
        }
        frame.render_widget(&soletra_frame, frame.area());
        let inner_area = soletra_frame.inner(frame.area());
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)]).areas(inner_area);
        let [
            alert_area,
            honeycomb_area,
            _,
            input_area,
            actions_area,
            _,
            points_area,
            _,
        ] = Layout::vertical([
            Constraint::Min(5),
            Constraint::Length(9),
            Constraint::Max(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Max(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(left_area);

        let alert = tui_overlay::Overlay::new()
            .anchor(tui_overlay::Anchor::Bottom)
            .slide(tui_overlay::Slide::Bottom)
            .width(Constraint::Fill(1));
        frame.render_stateful_widget(alert, alert_area, guess_result_state);
        if let Some(inner) = guess_result_state.inner_area()
            && let Some((result, instant)) = self.result.as_ref()
        {
            frame.render_widget(GuessResultWidget { result }, inner);
            if instant.elapsed() >= Duration::from_secs(3) {
                guess_result_state.close();
            }
        }

        let honeycomb = HoneycombWidget {
            main_letter: game.main_letter,
            secondary_letters: game.secondary_letters,
        };
        frame.render_stateful_widget(honeycomb, honeycomb_area, &mut self.areas);

        let input = InputWidget { input: &self.input };
        let mut state = InputWidgetState {
            cursor_position: Position::default(),
        };
        frame.render_stateful_widget(input, input_area, &mut state);
        if !is_game_over {
            frame.set_cursor_position(state.cursor_position);
        }

        frame.render_stateful_widget(ActionsWidget {}, actions_area, &mut self.areas);

        Gauge::default()
            .gauge_style(Style::new().green().on_black())
            .label(format!("{}/{}", game.points, game.total_points))
            .ratio((game.points as f64) / (game.total_points as f64))
            .render(points_area, frame.buffer_mut());

        let guesses = GuessesWidget {
            guesses: &mut game.words,
            scroll_view_state: &mut self.scroll_view_state,
            effects: &mut self.effects,
            elapsed: self.elapsed,
        };
        frame.render_stateful_widget(guesses, right_area, &mut self.rows);

        let game_over = tui_overlay::Overlay::new()
            .anchor(tui_overlay::Anchor::Center)
            .width(Constraint::Percentage(60))
            .height(Constraint::Percentage(50))
            .backdrop(tui_overlay::Backdrop::new(ratatui::style::Color::Black));
        frame.render_stateful_widget(game_over, inner_area, game_over_state);
        if let Some(inner) = game_over_state.inner_area() {
            frame.render_widget(
                GameOverWidget {
                    points: game.points,
                    words: game.words.len(),
                },
                inner,
            );
        }
    }

    fn render_loading(frame: &mut Frame) {
        frame.render_widget(Paragraph::new(t!("loading")).centered(), frame.area());
    }

    async fn handle_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
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
                            self.guess_result_state.close();
                            self.game_over_state.close();
                            self.result = None;
                            self.game_over = None;
                            if self.data.current_game > 0 {
                                for word in self
                                    .data
                                    .active_games
                                    .get_mut(self.data.current_game)
                                    .expect("length checked")
                                    .words
                                    .iter_mut()
                                {
                                    word.has_effect = false;
                                }
                                self.data.current_game -= 1;
                                self.effects = tachyonfx::EffectManager::default();
                                self.data.save().await?;
                            }
                        }
                        (KeyCode::Char(']'), _) => {
                            self.scroll_view_state.set_offset(Position::new(0, 0));
                            self.input.clear();
                            self.guess_result_state.close();
                            self.game_over_state.close();
                            self.result = None;
                            self.game_over = None;
                            for word in self
                                .data
                                .active_games
                                .get_mut(self.data.current_game)
                                .expect("length checked")
                                .words
                                .iter_mut()
                            {
                                word.has_effect = false;
                            }
                            self.data.current_game += 1;
                            self.effects = tachyonfx::EffectManager::default();
                            self.data.save().await?;
                        }
                        (KeyCode::Char(c), Some(_))
                            if self.input.chars().count() < MAX_CHARACTERS =>
                        {
                            self.input.push(c);
                        }
                        (KeyCode::Backspace, Some(_)) => {
                            self.input.pop();
                        }
                        (KeyCode::Enter, Some(game)) => {
                            let result = game.guess(&self.input);
                            if let GuessResult::Success {
                                index,
                                is_game_over,
                                ..
                            } = &result
                            {
                                self.scroll_view_state.set_offset(Position {
                                    x: ((index / self.rows) * 23).saturating_sub(1) as u16,
                                    y: 0,
                                });
                                self.data.save().await?;
                                if *is_game_over {
                                    self.game_over =
                                        Instant::now().checked_add(Duration::from_secs(1));
                                }
                            }
                            self.result = Some((result, Instant::now()));
                            self.guess_result_state.open();
                            self.input.clear();
                        }
                        (KeyCode::Right, _) => {
                            self.scroll_view_state.set_offset(
                                self.scroll_view_state
                                    .offset()
                                    .offset(Offset { x: 5, y: 0 }),
                            );
                        }
                        (KeyCode::Left, _) => {
                            self.scroll_view_state.set_offset(
                                self.scroll_view_state
                                    .offset()
                                    .offset(Offset { x: -5, y: 0 }),
                            );
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
                    let not_max_characters = self.input.chars().count() < MAX_CHARACTERS;
                    if not_max_characters && self.areas.button_main.contains(position) {
                        self.input.push(game.main_letter);
                    } else if not_max_characters && self.areas.button_one.contains(position) {
                        self.input.push(game.secondary_letters[0]);
                    } else if not_max_characters && self.areas.button_two.contains(position) {
                        self.input.push(game.secondary_letters[1]);
                    } else if not_max_characters && self.areas.button_three.contains(position) {
                        self.input.push(game.secondary_letters[2]);
                    } else if not_max_characters && self.areas.button_four.contains(position) {
                        self.input.push(game.secondary_letters[3]);
                    } else if not_max_characters && self.areas.button_five.contains(position) {
                        self.input.push(game.secondary_letters[4]);
                    } else if not_max_characters && self.areas.button_six.contains(position) {
                        self.input.push(game.secondary_letters[5]);
                    } else if self.areas.button_shuffle.contains(position) {
                        game.shuffle();
                    } else if self.areas.button_reset_shuffle.contains(position) {
                        game.reset_shuffle();
                    } else if self.areas.button_backspace.contains(position) {
                        self.input.pop();
                    } else if self.areas.button_submit.contains(position) {
                        let result = game.guess(&self.input);
                        if let GuessResult::Success {
                            index,
                            is_game_over,
                            ..
                        } = &result
                        {
                            self.scroll_view_state.set_offset(Position {
                                x: ((index / self.rows) * 23).saturating_sub(1) as u16,
                                y: 0,
                            });
                            self.data.save().await?;
                            if *is_game_over {
                                self.game_over = Instant::now().checked_add(Duration::from_secs(1));
                            }
                        }
                        self.result = Some((result, Instant::now()));
                        self.guess_result_state.open();
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
