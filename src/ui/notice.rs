use std::cell::Cell;
use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::colors::{STEEL, WHITE};
use crate::ui::{
    hint_bar::HintBar,
    hints::{HINT_CLOSE, HINT_SCROLL},
    layout::{Screen, Scrollbar},
    modal::{Frame, Modal},
    table::wrap_words,
};

const SET_ASIDE_TITLE: &str = " Save#aside ";
const SET_ASIDE_TEXT: &str = "fishtank could not read its last save, so a new game has begun. The old save is kept, untouched, at:";
const TEXT_W: u16 = 48;
const TEXT_PAD: u16 = 1;
const BACKGROUND: Color = Color::Reset;
const PARAGRAPH_GAP: &str = "";

pub struct NoticeState {
    title: &'static str,
    paragraphs: Vec<String>,
    top: usize,
    room: Cell<usize>,
    total: Cell<usize>,
}

impl NoticeState {
    pub fn save_set_aside(path: &Path) -> Self {
        Self {
            title: SET_ASIDE_TITLE,
            paragraphs: vec![SET_ASIDE_TEXT.to_string(), path.display().to_string()],
            top: 0,
            room: Cell::new(0),
            total: Cell::new(0),
        }
    }

    pub fn scroll_up(&mut self) {
        self.top = self.top.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.top + self.room.get() < self.total.get() {
            self.top += 1;
        }
    }

    fn top(&self, total: usize, room: usize) -> usize {
        self.top.min(total.saturating_sub(room))
    }

    fn lines(&self, width: u16) -> Vec<String> {
        let room = width.saturating_sub(TEXT_PAD * 2) as usize;
        let mut lines = Vec::new();
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            if index > 0 {
                lines.push(PARAGRAPH_GAP.to_string());
            }
            lines.extend(wrap_words(paragraph, room));
        }
        lines
    }
}

pub struct NoticePopup<'a> {
    state: &'a NoticeState,
    screen: Screen,
}

impl<'a> NoticePopup<'a> {
    pub fn new(state: &'a NoticeState, screen: Screen) -> Self {
        Self { state, screen }
    }

    fn hints(&self, overflowing: bool, total: usize) -> HintBar {
        let bar = HintBar::new(HINT_CLOSE);
        if !overflowing {
            return bar;
        }
        let top = self.state.top(total, self.state.room.get());
        bar.counted(HINT_SCROLL, Some((top + 1, total)))
    }
}

impl Widget for NoticePopup<'_> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let frame = Frame {
            title: self.state.title,
            border: WHITE,
            background: BACKGROUND,
        };
        let inner_w =
            Modal::inner_width(self.screen, frame.title, TEXT_W, &HintBar::new(HINT_CLOSE));
        let lines = self.state.lines(inner_w);
        let total = lines.len();
        self.state.total.set(total);
        let content = (TEXT_W, total as u16);
        let (modal, _) = Modal::open_fitting(buf, self.screen, &frame, content, |overflowing| {
            self.hints(overflowing, total)
        });
        let body = modal.body;
        let room = body.height as usize;
        self.state.room.set(room);
        let top = self.state.top(total, room);
        let shown = top..(top + room).min(total);
        let style = Style::default().fg(STEEL).bg(BACKGROUND);
        for (row, index) in shown.clone().enumerate() {
            buf.set_stringn(
                body.x + TEXT_PAD,
                body.y + row as u16,
                &lines[index],
                body.width.saturating_sub(TEXT_PAD * 2) as usize,
                style,
            );
        }
        Scrollbar {
            x: modal.scrollbar_x(),
            top: body.y,
            height: body.height,
        }
        .draw(buf, shown, total, WHITE);
    }
}
