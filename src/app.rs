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
use smol::{Timer, fs, stream::StreamExt};
use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind, Write},
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};
use tui_scrollview::ScrollViewState;

use crate::{
    game::{ActiveGame, Game, GuessResult},
    generate::generate_games,
    language::Language,
    widgets::{
        ActionsWidget, GameOverWidget, GuessResultWidget, GuessesWidget, HoneycombWidget,
        InputWidget, InputWidgetState, LanguageSelectWidget,
    },
};

const FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1_000 / FPS);
pub(crate) const MAX_CHARACTERS: usize = 19;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) active_games: Vec<ActiveGame>,
    pub(crate) current_game: usize,
    #[serde(skip)]
    pub(crate) save_path: PathBuf,
}

pub(crate) struct App {
    // Base app data
    language: Option<Language>,
    data: Option<AppData>,
    tx: Option<async_channel::Sender<AppEvent>>,
    games: Option<Vec<Game>>,
    should_quit: bool,
    downloading_files: bool,
    loading_games: bool,

    // Interactive buttons
    areas: AppAreas,

    // Language state
    selected_language: Language,

    // Game state
    result: Option<(GuessResult, Instant)>,
    game_over: Option<Instant>,
    input: String,
    rows: usize,
    scroll_view_state: ScrollViewState,
    guess_result_state: tui_overlay::OverlayState,
    game_over_state: tui_overlay::OverlayState,
    effects: tachyonfx::EffectManager<()>,
    elapsed: Duration,
}

#[derive(Default)]
pub(crate) struct AppAreas {
    // Language areas
    pub(crate) button_left: Rect,
    pub(crate) button_right: Rect,

    // Game areas
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

#[derive(Debug)]
pub(crate) enum AppEvent {
    Key(KeyEvent),
    Mouse(event::MouseEvent),
    DownloadingFiles,
    WordsRetrieved(Vec<String>, Language),
    GamesLoaded(Vec<Game>),
    Error(color_eyre::Report),
}

struct AppDir(PathBuf);

impl AppDir {
    fn new() -> color_eyre::Result<Self> {
        Ok(Self(app_dirs2::get_app_root(
            app_dirs2::AppDataType::UserData,
            &app_dirs2::AppInfo {
                name: "SoletraRs",
                author: "EpicEric",
            },
        )?))
    }

    fn get_base_path(&self) -> PathBuf {
        self.0.clone()
    }

    fn get_language_path(&self) -> PathBuf {
        self.0.join("language")
    }

    fn get_games_path(&self, language: Language) -> PathBuf {
        self.0.join(format!("games_{}.json", language.shortcode()))
    }

    fn get_save_path(&self, language: Language) -> PathBuf {
        self.0.join(format!("{}.json", language.shortcode()))
    }
}

impl AppData {
    async fn load(language: Language) -> color_eyre::Result<Self> {
        let save_path = AppDir::new()?.get_save_path(language);
        if save_path.exists() && save_path.is_file() {
            let path = save_path.clone();
            let data: color_eyre::Result<AppData> = smol::unblock(move || {
                let reader = File::open(path)?;
                Ok(serde_json::from_reader(reader)?)
            })
            .await;
            Ok(AppData { save_path, ..data? })
        } else {
            Ok(AppData {
                save_path,
                ..Default::default()
            })
        }
    }

    async fn save(&self) -> color_eyre::Result<()> {
        fs::write(self.save_path.as_path(), serde_json::to_vec(self)?).await?;
        Ok(())
    }
}

impl App {
    pub(crate) async fn init() -> color_eyre::Result<Self> {
        let app_dir = AppDir::new()?;
        fs::create_dir_all(app_dir.get_base_path()).await?;

        let (language, data) = if let Ok(language_str) =
            fs::read_to_string(app_dir.get_language_path()).await
            && let Ok(language) = Language::from_str(language_str.trim())
        {
            let language_str = language.shortcode();
            rust_i18n::set_locale(language_str);
            (Some(language), Some(AppData::load(language).await?))
        } else {
            (None, None)
        };

        Ok(App {
            language,
            data,
            tx: None,
            games: None,
            should_quit: false,
            downloading_files: false,
            loading_games: false,
            areas: Default::default(),
            selected_language: language.unwrap_or(Language::Portuguese),
            result: None,
            game_over: None,
            input: String::new(),
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
        })
    }

    pub(crate) async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        crossterm::execute!(terminal.backend_mut(), EnableMouseCapture)?;
        let (tx, rx) = async_channel::bounded(32);
        self.tx = Some(tx.clone());

        // Spawn event handler task
        let event_tx = tx.clone();
        smol::spawn(async move {
            event_handler(event_tx).await;
        })
        .detach();

        let mut frame_interval = Timer::interval(FRAME_DURATION);
        let mut prev = Instant::now();

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
            self.render(terminal).await?;
            let curr = frame_interval
                .next()
                .await
                .expect("timer finished unexpectedly");
            self.elapsed = curr.duration_since(prev);
            self.guess_result_state.tick(self.elapsed);
            self.game_over_state.tick(self.elapsed);
            if let Some(game_over) = self.game_over
                && curr >= game_over
            {
                self.game_over_state.open();
                self.game_over = None;
            }
            prev = curr;
        }
    }

    async fn load_games(&mut self, language: Language) -> color_eyre::Result<()> {
        if !self.loading_games {
            self.loading_games = true;
            if let Some(tx) = self.tx.clone() {
                smol::spawn(async move {
                    let games_path = match AppDir::new() {
                        Ok(app_dir) => app_dir.get_games_path(language),
                        Err(error) => {
                            tx.send(AppEvent::Error(error.into()))
                                .await
                                .expect("channel isn't closed");
                            return;
                        }
                    };
                    match File::open(games_path) {
                        Ok(file) => {
                            match smol::unblock(|| serde_json::from_reader(BufReader::new(file)))
                                .await
                            {
                                Ok(games) => tx
                                    .send(AppEvent::GamesLoaded(games))
                                    .await
                                    .expect("channel isn't closed"),
                                Err(error) => tx
                                    .send(AppEvent::Error(error.into()))
                                    .await
                                    .expect("channel isn't closed"),
                            }
                        }
                        Err(error) if error.kind() == ErrorKind::NotFound => {
                            tx.send(AppEvent::DownloadingFiles)
                                .await
                                .expect("channel isn't closed");
                            match language.get_words().await {
                                Ok(words) => tx
                                    .send(AppEvent::WordsRetrieved(words, language))
                                    .await
                                    .expect("channel isn't closed"),
                                Err(error) => tx
                                    .send(AppEvent::Error(error.into()))
                                    .await
                                    .expect("channel isn't closed"),
                            }
                        }
                        Err(error) => tx
                            .send(AppEvent::Error(error.into()))
                            .await
                            .expect("channel isn't closed"),
                    }
                })
                .detach();
            }
        }
        Ok(())
    }

    async fn render(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        if let Some(language) = self.language {
            if let Some(data) = self.data.as_mut() {
                if data.current_game >= data.active_games.len() {
                    match self.games.as_ref() {
                        Some(games) => {
                            let index = {
                                let mut rng = SmallRng::seed_from_u64(42);
                                let range = Uniform::try_from(0..games.len())?;
                                for _ in range.sample_iter(&mut rng).take(data.active_games.len()) {
                                }
                                range.sample(&mut rng)
                            };
                            if let Some(game) = games.get(index) {
                                data.active_games.push(game.clone().into());
                                let current_game = data.active_games.len() - 1;
                                data.current_game = current_game;
                                data.save().await?;
                                terminal.draw(|frame| self.render_game(frame))?;
                            } else {
                                unreachable!("cannot sample another game")
                            }
                        }
                        None => {
                            terminal.draw(|frame| self.render_loading(frame))?;
                            self.load_games(language).await?;
                        }
                    }
                } else {
                    terminal.draw(|frame| self.render_game(frame))?;
                }
            } else {
                unreachable!("language set but data unset");
            }
        } else {
            terminal.draw(|frame| self.render_language_selection(frame))?;
        }
        Ok(())
    }

    fn render_language_selection(&mut self, frame: &mut Frame) {
        self.areas.button_main = Rect::ZERO;
        self.areas.button_one = Rect::ZERO;
        self.areas.button_two = Rect::ZERO;
        self.areas.button_three = Rect::ZERO;
        self.areas.button_four = Rect::ZERO;
        self.areas.button_five = Rect::ZERO;
        self.areas.button_six = Rect::ZERO;
        self.areas.button_shuffle = Rect::ZERO;
        self.areas.button_reset_shuffle = Rect::ZERO;
        self.areas.button_backspace = Rect::ZERO;
        self.areas.button_submit = Rect::ZERO;

        let soletra_frame = Block::bordered()
            .border_type(BorderType::Thick)
            .title_top(Line::from(" soletra-rs ").centered());
        frame.render_widget(&soletra_frame, frame.area());
        let inner_area = soletra_frame.inner(frame.area());
        frame.render_stateful_widget(
            LanguageSelectWidget {
                language: self.selected_language,
            },
            inner_area,
            &mut self.areas,
        );
    }

    fn render_game(&mut self, frame: &mut Frame) {
        self.areas.button_left = Rect::ZERO;
        self.areas.button_right = Rect::ZERO;

        let data = self.data.as_mut().expect("no data in render_game");
        let game = data
            .active_games
            .get_mut(data.current_game)
            .expect("length checked");
        let guess_result_state = &mut self.guess_result_state;
        let game_over_state = &mut self.game_over_state;
        let is_game_over = game_over_state.is_open();

        let mut soletra_frame = Block::bordered()
            .border_type(BorderType::Thick)
            .title_top(Line::from(" soletra-rs ").centered())
            .title_bottom(Line::from(t!("game_number", game => data.current_game + 1)).centered())
            .title_bottom(Line::from(t!("next_game").reversed()).right_aligned());
        if data.current_game > 0 {
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

    fn render_loading(&self, frame: &mut Frame) {
        let soletra_frame = Block::bordered()
            .border_type(BorderType::Thick)
            .title_top(Line::from(" soletra-rs ").centered());
        frame.render_widget(&soletra_frame, frame.area());
        let inner_area = soletra_frame.inner(frame.area());

        frame.render_widget(
            Paragraph::new(if self.downloading_files {
                t!("downloading_files")
            } else {
                t!("loading")
            })
            .centered(),
            inner_area,
        );
    }

    async fn handle_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
        match event {
            AppEvent::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if let Some(data) = self.data.as_mut() {
                        match (key.code, data.active_games.get_mut(data.current_game)) {
                            (KeyCode::Char('c'), _)
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                self.should_quit = true;
                            }
                            (KeyCode::Char('l'), _)
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                self.games = None;
                                self.language = None;
                                self.data = None;
                            }
                            (KeyCode::Char('['), _) => {
                                self.scroll_view_state.set_offset(Position::new(0, 0));
                                self.input.clear();
                                self.guess_result_state.close();
                                self.game_over_state.close();
                                self.result = None;
                                self.game_over = None;
                                if data.current_game > 0 {
                                    for word in data
                                        .active_games
                                        .get_mut(data.current_game)
                                        .expect("length checked")
                                        .words
                                        .iter_mut()
                                    {
                                        word.has_effect = false;
                                    }
                                    data.current_game -= 1;
                                    self.effects = tachyonfx::EffectManager::default();
                                    data.save().await?;
                                }
                            }
                            (KeyCode::Char(']'), _) => {
                                self.scroll_view_state.set_offset(Position::new(0, 0));
                                self.input.clear();
                                self.guess_result_state.close();
                                self.game_over_state.close();
                                self.result = None;
                                self.game_over = None;
                                for word in data
                                    .active_games
                                    .get_mut(data.current_game)
                                    .expect("length checked")
                                    .words
                                    .iter_mut()
                                {
                                    word.has_effect = false;
                                }
                                data.current_game += 1;
                                self.effects = tachyonfx::EffectManager::default();
                                data.save().await?;
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
                                    data.save().await?;
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
                    } else {
                        match key.code {
                            KeyCode::Char('c')
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                self.should_quit = true;
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                self.selected_language = match self.selected_language {
                                    Language::Portuguese => Language::English,
                                    Language::English => Language::Portuguese,
                                };
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                self.selected_language = match self.selected_language {
                                    Language::Portuguese => Language::English,
                                    Language::English => Language::Portuguese,
                                };
                            }
                            KeyCode::Enter => {
                                let language = self.selected_language;
                                self.language = Some(language);
                                self.data = Some(AppData::load(language).await?);
                                let shortcode = language.shortcode();
                                rust_i18n::set_locale(shortcode);
                                fs::write(AppDir::new()?.get_language_path(), shortcode).await?;
                                self.load_games(language).await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            AppEvent::Mouse(mouse) => {
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                    let position = Position::new(mouse.column, mouse.row);
                    if self.language.is_none() {
                        if self.areas.button_left.contains(position) {
                            self.selected_language = match self.selected_language {
                                Language::Portuguese => Language::English,
                                Language::English => Language::Portuguese,
                            };
                        } else if self.areas.button_right.contains(position) {
                            self.selected_language = match self.selected_language {
                                Language::Portuguese => Language::English,
                                Language::English => Language::Portuguese,
                            };
                        }
                    } else if let Some(data) = self.data.as_mut()
                        && let Some(game) = data.active_games.get_mut(data.current_game)
                    {
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
                                data.save().await?;
                                if *is_game_over {
                                    self.game_over =
                                        Instant::now().checked_add(Duration::from_secs(1));
                                }
                            }
                            self.result = Some((result, Instant::now()));
                            self.guess_result_state.open();
                            self.input.clear();
                        }
                    }
                }
            }
            AppEvent::DownloadingFiles => {
                self.downloading_files = true;
            }
            AppEvent::WordsRetrieved(words, language) => {
                self.downloading_files = false;
                if let Some(tx) = self.tx.clone() {
                    smol::spawn(async move {
                        match smol::unblock(move || {
                            let games = generate_games(words)?;
                            let games_path = AppDir::new()?.get_games_path(language);
                            let mut writer = BufWriter::new(File::create(games_path)?);
                            serde_json::to_writer(&mut writer, &games)?;
                            writer.flush()?;
                            Ok(games)
                        })
                        .await
                        {
                            Ok(games) => {
                                tx.send(AppEvent::GamesLoaded(games))
                                    .await
                                    .expect("channel isn't closed");
                            }
                            Err(error) => {
                                tx.send(AppEvent::Error(error))
                                    .await
                                    .expect("channel isn't closed");
                            }
                        }
                    })
                    .detach();
                }
            }
            AppEvent::GamesLoaded(games) => {
                self.games = Some(games);
                self.loading_games = false;
            }
            AppEvent::Error(error) => return Err(error),
        }
        Ok(())
    }
}

async fn event_handler(tx: async_channel::Sender<AppEvent>) {
    loop {
        if event::poll(Duration::from_millis(10)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if tx.send(AppEvent::Key(key)).await.is_err() {
                        break;
                    }
                }
                Ok(Event::Mouse(mouse)) => {
                    if tx.send(AppEvent::Mouse(mouse)).await.is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}
