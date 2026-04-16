use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Size},
    style::{Color, Style, Stylize},
    text::{Line, ToSpan},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget, Wrap},
};
use rust_i18n::t;
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::{
    app::AppAreas,
    game::{ActiveGameWord, GuessResult},
};

pub(crate) struct HoneycombWidget {
    pub(crate) main_letter: char,
    pub(crate) secondary_letters: [char; 6],
}

pub(crate) struct InputWidget<'a> {
    pub(crate) input: &'a str,
}

pub(crate) struct InputWidgetState {
    pub(crate) cursor_position: Position,
}

pub(crate) struct ActionsWidget;

pub(crate) struct GuessesWidget<'a> {
    pub(crate) guesses: &'a mut [ActiveGameWord],
    pub(crate) scroll_view_state: &'a mut ScrollViewState,
    pub(crate) effects: &'a mut tachyonfx::EffectManager<()>,
    pub(crate) elapsed: Duration,
}

pub(crate) struct GuessResultWidget<'a> {
    pub(crate) result: &'a GuessResult,
}

pub(crate) struct GameOverWidget {
    pub(crate) points: u16,
    pub(crate) words: usize,
}

impl StatefulWidget for HoneycombWidget {
    type State = AppAreas;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppAreas) {
        let area = area.centered_horizontally(Constraint::Length(15));

        let [top, middle, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(area);
        let [rect_one, _, rect_six] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
        ])
        .horizontal_margin(2)
        .areas(top);
        let [rect_two, rect_main, rect_five] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .areas(middle);
        let [rect_three, _, rect_four] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
        ])
        .horizontal_margin(2)
        .areas(bottom);

        state.button_main = rect_main;
        state.button_one = rect_one;
        state.button_two = rect_two;
        state.button_three = rect_three;
        state.button_four = rect_four;
        state.button_five = rect_five;
        state.button_six = rect_six;

        let block_main = Block::bordered().black().on_cyan();
        let inner_main = block_main.inner(rect_main);
        block_main.render(rect_main, buf);
        self.main_letter
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_main, buf);

        let block_one = Block::bordered();
        let inner_one = block_one.inner(rect_one);
        block_one.render(rect_one, buf);
        self.secondary_letters[0]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_one, buf);

        let block_two = Block::bordered();
        let inner_two = block_two.inner(rect_two);
        block_two.render(rect_two, buf);
        self.secondary_letters[1]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_two, buf);

        let block_three = Block::bordered();
        let inner_three = block_three.inner(rect_three);
        block_three.render(rect_three, buf);
        self.secondary_letters[2]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_three, buf);

        let block_four = Block::bordered();
        let inner_four = block_four.inner(rect_four);
        block_four.render(rect_four, buf);
        self.secondary_letters[3]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_four, buf);

        let block_five = Block::bordered();
        let inner_five = block_five.inner(rect_five);
        block_five.render(rect_five, buf);
        self.secondary_letters[4]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_five, buf);

        let block_six = Block::bordered();
        let inner_six = block_six.inner(rect_six);
        block_six.render(rect_six, buf);
        self.secondary_letters[5]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(inner_six, buf);
    }
}

impl<'a> StatefulWidget for InputWidget<'a> {
    type State = InputWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut InputWidgetState) {
        let block_input = Block::bordered().border_type(BorderType::Rounded);
        let inner_input = block_input.inner(area);
        block_input.render(area, buf);
        self.input
            .to_lowercase()
            .to_span()
            .into_left_aligned_line()
            .render(inner_input, buf);
        state.cursor_position = Position {
            x: inner_input.x + (self.input.chars().count() as u16),
            y: inner_input.y,
        };
    }
}

impl StatefulWidget for ActionsWidget {
    type State = AppAreas;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppAreas) {
        let [
            rect_backspace,
            rect_shuffle,
            rect_reset_shuffle,
            rect_submit,
        ] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .flex(Flex::Center)
        .areas(area);

        state.button_backspace = rect_backspace;
        state.button_shuffle = rect_shuffle;
        state.button_reset_shuffle = rect_reset_shuffle;
        state.button_submit = rect_submit;

        let block_backspace = Block::bordered();
        let inner_backspace = block_backspace.inner(rect_backspace);
        block_backspace.render(rect_backspace, buf);
        "󰁮".bold().into_centered_line().render(inner_backspace, buf);

        let block_shuffle = Block::bordered();
        let inner_shuffle = block_shuffle.inner(rect_shuffle);
        block_shuffle.render(rect_shuffle, buf);
        "".bold().into_centered_line().render(inner_shuffle, buf);

        let block_reset_shuffle = Block::bordered();
        let inner_reset_shuffle = block_reset_shuffle.inner(rect_reset_shuffle);
        block_reset_shuffle.render(rect_reset_shuffle, buf);
        ""
            .bold()
            .into_centered_line()
            .render(inner_reset_shuffle, buf);

        let block_submit = Block::bordered();
        let inner_submit = block_submit.inner(rect_submit);
        block_submit.render(rect_submit, buf);
        "".bold().into_centered_line().render(inner_submit, buf);
    }
}

impl<'a> StatefulWidget for GuessesWidget<'a> {
    type State = usize;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut usize) {
        let rows = 1usize.max(((area.height.saturating_sub(2)) / 3) as usize);
        *state = rows;
        let cols = self.guesses.len().div_ceil(rows);
        let col_constraints = (0..cols).map(|_| Constraint::Length(22));
        let row_constraints = (0..rows).map(|_| Constraint::Length(3));
        let horizontal = Layout::horizontal(col_constraints).spacing(1);
        let vertical = Layout::vertical(row_constraints);

        let mut scroll_view = ScrollView::new(Size::new(
            (23 * (cols as u16)).saturating_sub(1),
            3 * (rows as u16),
        ));
        let scroll_view_area = scroll_view.area();
        let scroll_view_buf = scroll_view.buf_mut();
        let cols_layout = scroll_view_area.layout_vec(&horizontal);
        let cells = cols_layout.iter().flat_map(|col| col.layout_vec(&vertical));

        for (cell, guess) in cells.zip(self.guesses.iter_mut()) {
            if guess.discovered {
                Paragraph::new(guess.original.as_str())
                    .block(Block::bordered())
                    .not_dim()
                    .render(cell, scroll_view_buf);
                if !guess.has_effect {
                    self.effects.add_effect(
                        tachyonfx::fx::slide_in(
                            tachyonfx::Motion::LeftToRight,
                            10,
                            0,
                            Color::Reset,
                            (500, tachyonfx::Interpolation::Linear),
                        )
                        .with_area(cell),
                    );
                    guess.has_effect = true;
                }
            } else {
                Paragraph::new(t!(
                    "hidden_guess",
                    letters => guess.normalized.as_ref().chars().count()
                ))
                .block(Block::bordered())
                .dim()
                .centered()
                .render(cell, scroll_view_buf);
            }
        }

        self.effects
            .process_effects(self.elapsed.into(), scroll_view_buf, scroll_view_area);

        scroll_view.render(
            area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            buf,
            self.scroll_view_state,
        );
    }
}

impl<'a> Widget for GuessResultWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let paragraph = match self.result {
            GuessResult::Success {
                points, is_pangram, ..
            } => {
                let text = if *is_pangram {
                    t!("good_guess_pangram", points => points)
                } else {
                    t!("good_guess", points => points)
                };
                Paragraph::new(text.green())
                    .wrap(Wrap { trim: true })
                    .block(Block::bordered().border_style(Style::new().green()))
                    .centered()
            }
            GuessResult::Failure(bad_guess) => Paragraph::new(bad_guess.to_string().red())
                .wrap(Wrap { trim: true })
                .block(Block::bordered().border_style(Style::new().red()))
                .centered(),
        };
        let [_, inner] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(paragraph.line_count(area.width - 2) as u16),
        ])
        .areas(area);
        paragraph.render(inner, buf);
    }
}

impl Widget for GameOverWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(vec![
            Line::from(t!("victory").green()),
            Line::from(t!(
                "victory.line_1",
                words => self.words,
                points => self.points,
            )),
            Line::from(t!("victory.line_2")),
        ])
        .block(
            Block::bordered()
                .border_type(BorderType::QuadrantOutside)
                .border_style(Style::new().green()),
        )
        .centered()
        .render(area, buf);
    }
}
