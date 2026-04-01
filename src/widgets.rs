use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, Widget},
};

pub(crate) struct Honeycomb {
    pub(crate) main_letter: char,
    pub(crate) secondary_letters: [char; 6],
}

impl Widget for Honeycomb {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, middle, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(area);
        let [one_rect, _, six_rect] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
        ])
        .horizontal_margin(2)
        .areas(top);
        let [two_rect, main_rect, five_rect] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .areas(middle);
        let [three_rect, _, four_rect] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
        ])
        .horizontal_margin(2)
        .areas(bottom);

        let main_block = Block::bordered().black().on_cyan();
        let main_inner = main_block.inner(main_rect);
        main_block.render(main_rect, buf);
        self.main_letter
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(main_inner, buf);
        let one_block = Block::bordered().white();
        let one_inner = one_block.inner(one_rect);
        one_block.render(one_rect, buf);
        self.secondary_letters[0]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(one_inner, buf);
        let two_block = Block::bordered().white();
        let two_inner = two_block.inner(two_rect);
        two_block.render(two_rect, buf);
        self.secondary_letters[1]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(two_inner, buf);
        let three_block = Block::bordered().white();
        let three_inner = three_block.inner(three_rect);
        three_block.render(three_rect, buf);
        self.secondary_letters[2]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(three_inner, buf);
        let four_block = Block::bordered().white();
        let four_inner = four_block.inner(four_rect);
        four_block.render(four_rect, buf);
        self.secondary_letters[3]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(four_inner, buf);
        let five_block = Block::bordered().white();
        let five_inner = five_block.inner(five_rect);
        five_block.render(five_rect, buf);
        self.secondary_letters[4]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(five_inner, buf);
        let six_block = Block::bordered().white();
        let six_inner = six_block.inner(six_rect);
        six_block.render(six_rect, buf);
        self.secondary_letters[5]
            .to_uppercase()
            .next()
            .expect("valid character")
            .bold()
            .into_centered_line()
            .render(six_inner, buf);
    }
}
