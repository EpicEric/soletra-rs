use async_compat::Compat;
use color_eyre::eyre::eyre;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
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

#[derive(Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) active_games: Vec<ActiveGame>,
    pub(crate) current_game: usize,
    #[serde(skip)]
    pub(crate) save_path: PathBuf,
    #[serde(skip)]
    pub(crate) language: Language,
}

pub(crate) struct App {
    // Base app data
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

    // Loading state
    throbber_state: throbber_widgets_tui::ThrobberState,

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
    Mouse(MouseEvent),
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

    fn get_games_temp_path(&self, language: Language) -> PathBuf {
        self.0
            .join(format!("games_{}.json.temp", language.shortcode()))
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
            data.map(|data| AppData {
                save_path,
                language,
                active_games: data.active_games,
                current_game: data.current_game,
            })
        } else {
            Ok(AppData {
                save_path,
                language,
                active_games: vec![],
                current_game: 0,
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

        let (selected_language, data) = if let Ok(language_str) =
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
            data,
            tx: None,
            games: None,
            should_quit: false,
            downloading_files: false,
            loading_games: false,
            areas: Default::default(),
            selected_language: selected_language.unwrap_or_default(),
            throbber_state: Default::default(),
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

        let event_tx = tx.clone();
        smol::spawn(event_handler(event_tx)).detach();

        let mut frame_interval = Timer::interval(FRAME_DURATION);
        let mut prev = Instant::now();
        let mut throbber_count = 0u8;

        loop {
            // Handle events
            while let Ok(event) = rx.try_recv() {
                if let Err(error) = self.handle_event(event).await {
                    crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    return Err(error);
                } else if self.should_quit {
                    crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    return Ok(());
                }
            }

            // Render terminal
            self.render(terminal).await?;
            let curr = frame_interval.next().await.expect("looping timer");
            self.elapsed = curr.duration_since(prev);

            if throbber_count >= 1 {
                throbber_count = 0;
                self.throbber_state.calc_next();
            } else {
                throbber_count += 1;
            }
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
                            tx.send(AppEvent::Error(error))
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
                                    .send(AppEvent::Error(error))
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
        if let Some(data) = self.data.as_mut() {
            if data.current_game >= data.active_games.len() {
                match self.games.as_ref() {
                    Some(games) => {
                        let index = {
                            let mut rng = SmallRng::seed_from_u64(42);
                            let range = Uniform::try_from(0..games.len())?;
                            for _ in range.sample_iter(&mut rng).take(data.active_games.len()) {}
                            range.sample(&mut rng)
                        };
                        let game = games.get(index).expect("length checked");
                        data.active_games.push(game.clone().into());
                        let current_game = data.active_games.len() - 1;
                        data.current_game = current_game;
                        data.save().await?;
                        terminal.draw(|frame| self.render_game(frame))?;
                    }
                    None => {
                        let language = data.language;
                        terminal.draw(|frame| self.render_loading(frame))?;
                        self.load_games(language).await?;
                    }
                }
            } else {
                terminal.draw(|frame| self.render_game(frame))?;
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

        let data = self.data.as_mut().expect("precondition: data exists");
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

    fn render_loading(&mut self, frame: &mut Frame) {
        let soletra_frame = Block::bordered()
            .border_type(BorderType::Thick)
            .title_top(Line::from(" soletra-rs ").centered());
        frame.render_widget(&soletra_frame, frame.area());
        let [_, rect_throbber, rect_text, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(soletra_frame.inner(frame.area()));

        frame.render_stateful_widget(
            throbber_widgets_tui::Throbber::default()
                .throbber_set(throbber_widgets_tui::BRAILLE_SIX_DOUBLE)
                .use_type(throbber_widgets_tui::WhichUse::Spin),
            rect_throbber.centered_horizontally(Constraint::Length(1)),
            &mut self.throbber_state,
        );

        frame.render_widget(
            Paragraph::new(if self.downloading_files {
                t!("downloading_files")
            } else {
                t!("loading")
            })
            .centered(),
            rect_text,
        );
    }

    async fn handle_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
        match event {
            AppEvent::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if let Some(data) = self.data.as_mut() {
                        match (key.code, data.active_games.get_mut(data.current_game)) {
                            (KeyCode::Char('c'), _)
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                self.should_quit = true;
                            }
                            (KeyCode::Char('l'), _)
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                self.games = None;
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
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                self.should_quit = true;
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                self.selected_language = self.selected_language.previous();
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                self.selected_language = self.selected_language.next();
                            }
                            KeyCode::Enter => {
                                let language = self.selected_language;
                                if self
                                    .data
                                    .as_ref()
                                    .is_none_or(|data| data.language != language)
                                {
                                    self.data = Some(AppData::load(language).await?);
                                    let shortcode = language.shortcode();
                                    rust_i18n::set_locale(shortcode);
                                    fs::write(AppDir::new()?.get_language_path(), shortcode)
                                        .await?;
                                    self.load_games(language).await?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            AppEvent::Mouse(mouse) => {
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                    let position = Position::new(mouse.column, mouse.row);
                    if let Some(data) = self.data.as_mut() {
                        if let Some(game) = data.active_games.get_mut(data.current_game) {
                            let not_max_characters = self.input.chars().count() < MAX_CHARACTERS;
                            if not_max_characters && self.areas.button_main.contains(position) {
                                self.input.push(game.main_letter);
                            } else if not_max_characters && self.areas.button_one.contains(position)
                            {
                                self.input.push(game.secondary_letters[0]);
                            } else if not_max_characters && self.areas.button_two.contains(position)
                            {
                                self.input.push(game.secondary_letters[1]);
                            } else if not_max_characters
                                && self.areas.button_three.contains(position)
                            {
                                self.input.push(game.secondary_letters[2]);
                            } else if not_max_characters
                                && self.areas.button_four.contains(position)
                            {
                                self.input.push(game.secondary_letters[3]);
                            } else if not_max_characters
                                && self.areas.button_five.contains(position)
                            {
                                self.input.push(game.secondary_letters[4]);
                            } else if not_max_characters && self.areas.button_six.contains(position)
                            {
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
                    } else {
                        if self.areas.button_left.contains(position) {
                            self.selected_language = self.selected_language.previous();
                        } else if self.areas.button_right.contains(position) {
                            self.selected_language = self.selected_language.next();
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
                    smol::spawn(generate_games_from_words(tx, words, language)).detach();
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

async fn generate_games_from_words(
    tx: async_channel::Sender<AppEvent>,
    words: Vec<String>,
    language: Language,
) {
    match smol::unblock(move || {
        let games = generate_games(words)?;
        let app_dir = AppDir::new()?;
        let tmp_path = app_dir.get_games_temp_path(language);
        let tmp_file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(&tmp_file);
        serde_json::to_writer(&mut writer, &games)?;
        writer.flush()?;
        drop(writer);
        drop(tmp_file);
        std::fs::rename(tmp_path, app_dir.get_games_path(language))?;
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
}

async fn event_handler(tx: async_channel::Sender<AppEvent>) {
    let mut stream = EventStream::new();
    loop {
        let Some(Ok(event)) = Compat::new(stream.next()).await else {
            break;
        };
        match event {
            Event::Key(key) => {
                tx.send(AppEvent::Key(key))
                    .await
                    .expect("channel isn't closed");
            }
            Event::Mouse(mouse) => {
                tx.send(AppEvent::Mouse(mouse))
                    .await
                    .expect("channel isn't closed");
            }
            _ => {}
        }
    }
    tx.send(AppEvent::Error(eyre!("EventStream was closed")))
        .await
        .expect("channel isn't closed");
}
