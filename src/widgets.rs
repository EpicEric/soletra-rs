use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Size},
    style::Stylize,
    text::ToSpan,
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget},
};
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::game::ActiveGameWord;

pub(crate) struct HoneycombWidget {
    pub(crate) main_letter: char,
    pub(crate) secondary_letters: [char; 6],

    pub(crate) area_button_main: Rect,
    pub(crate) area_button_one: Rect,
    pub(crate) area_button_two: Rect,
    pub(crate) area_button_three: Rect,
    pub(crate) area_button_four: Rect,
    pub(crate) area_button_five: Rect,
    pub(crate) area_button_six: Rect,
}

pub(crate) struct InputWidget<'a> {
    pub(crate) input: &'a str,

    pub(crate) cursor_position: Position,
}

pub(crate) struct ActionsWidget {
    pub(crate) area_button_submit: Rect,
    pub(crate) area_button_shuffle: Rect,
    pub(crate) area_button_reset_shuffle: Rect,
    pub(crate) area_button_clear: Rect,
}

pub(crate) struct GuessesWidget<'a> {
    pub(crate) guesses: &'a [ActiveGameWord],
    pub(crate) scroll_view_state: &'a mut ScrollViewState,
}

impl Widget for &mut HoneycombWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        self.area_button_main = rect_main;
        self.area_button_one = rect_one;
        self.area_button_two = rect_two;
        self.area_button_three = rect_three;
        self.area_button_four = rect_four;
        self.area_button_five = rect_five;
        self.area_button_six = rect_six;

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

impl<'a> Widget for &mut InputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block_input = Block::bordered().border_type(BorderType::Rounded);
        let inner_input = block_input.inner(area);
        block_input.render(area, buf);
        self.input
            .to_lowercase()
            .to_span()
            .into_left_aligned_line()
            .render(inner_input, buf);
        self.cursor_position = Position {
            x: inner_input.x + (self.input.chars().count() as u16),
            y: inner_input.y,
        };
    }
}

impl Widget for &mut ActionsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [rect_clear, rect_shuffle, rect_reset_shuffle, rect_submit] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .flex(Flex::Center)
        .areas(area);

        self.area_button_clear = rect_clear;
        self.area_button_shuffle = rect_shuffle;
        self.area_button_reset_shuffle = rect_reset_shuffle;
        self.area_button_submit = rect_submit;

        let block_clear = Block::bordered().dim();
        let inner_clear = block_clear.inner(rect_clear);
        block_clear.render(rect_clear, buf);
        "".bold().into_centered_line().render(inner_clear, buf);

        let block_shuffle = Block::bordered().dim();
        let inner_shuffle = block_shuffle.inner(rect_shuffle);
        block_shuffle.render(rect_shuffle, buf);
        "".bold().into_centered_line().render(inner_shuffle, buf);

        let block_reset_shuffle = Block::bordered().dim();
        let inner_reset_shuffle = block_reset_shuffle.inner(rect_reset_shuffle);
        block_reset_shuffle.render(rect_reset_shuffle, buf);
        ""
            .bold()
            .into_centered_line()
            .render(inner_reset_shuffle, buf);

        let block_submit = Block::bordered().dim();
        let inner_submit = block_submit.inner(rect_submit);
        block_submit.render(rect_submit, buf);
        "".bold().into_centered_line().render(inner_submit, buf);
    }
}

impl<'a> Widget for &mut GuessesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = 3;
        let cols = self.guesses.len().div_ceil(rows);
        let col_constraints = (0..cols).map(|_| Constraint::Length(22));
        let row_constraints = (0..rows).map(|_| Constraint::Length(3));
        let horizontal = Layout::horizontal(col_constraints).spacing(1);
        let vertical = Layout::vertical(row_constraints);

        let mut scroll_view = ScrollView::new(Size::new(23 * (cols as u16) - 1, 3 * (rows as u16)));
        let cols_layout = scroll_view.area().layout_vec(&horizontal);
        let cells = cols_layout.iter().flat_map(|col| col.layout_vec(&vertical));

        for (cell, guess) in cells.zip(self.guesses.iter()) {
            if guess.discovered {
                Paragraph::new(guess.original.as_str())
                    .block(Block::bordered())
                    .not_dim()
                    .render(cell, scroll_view.buf_mut());
            } else {
                Paragraph::new("???")
                    .block(Block::bordered())
                    .dim()
                    .centered()
                    .render(cell, scroll_view.buf_mut());
            }
        }

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
