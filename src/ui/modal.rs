use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::ui::hint_bar::HintBar;
use crate::ui::layout::{Screen, body_rows};
use crate::ui::overdraw;
use crate::ui::table::{self, visual_width};

pub const BORDER: u16 = 1;
const BORDERS: u16 = BORDER * 2;
const HINT_PADDING_ROWS: u16 = 1;
const TITLE_MARGIN: u16 = 4;

pub struct ModalSpec<'a> {
    pub title: &'a str,
    pub border: Color,
    pub background: Color,
    pub content_w: u16,
    pub content_h: u16,
    pub hints: &'a HintBar,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Modal {
    pub rect: Rect,
    pub body: Rect,
    pub hints: Rect,
}

impl Modal {
    pub fn inner_width(screen: Screen, title: &str, content_w: u16, hints: &HintBar) -> u16 {
        let title_w = visual_width(title) as u16 + TITLE_MARGIN;
        let natural_inner_w = content_w.max(hints.natural_width()).max(title_w);
        (natural_inner_w + BORDERS)
            .min(screen.whole.width)
            .saturating_sub(BORDERS)
    }

    pub fn measure(screen: Screen, spec: &ModalSpec) -> Modal {
        let inner_w = Self::inner_width(screen, spec.title, spec.content_w, spec.hints);
        let width = inner_w + BORDERS.min(screen.whole.width);
        let hint_rows = spec.hints.height(inner_w);
        let natural_h = BORDERS + spec.content_h + HINT_PADDING_ROWS + hint_rows;
        let rect = screen.place(width, natural_h);
        let inner_h = rect.height.saturating_sub(BORDERS);
        let hint_h = hint_rows.min(inner_h.saturating_sub(spec.content_h.min(1)));
        let rest = inner_h - hint_h;
        let body_h = body_rows(spec.content_h, rest, HINT_PADDING_ROWS);
        let inner_x = rect.x + BORDER.min(rect.width);
        let inner_y = rect.y + BORDER.min(rect.height);
        Modal {
            rect,
            body: Rect::new(inner_x, inner_y, inner_w, body_h),
            hints: Rect::new(inner_x, inner_y + inner_h - hint_h, inner_w, hint_h),
        }
    }

    pub fn open(buf: &mut Buffer, screen: Screen, spec: &ModalSpec) -> Modal {
        let modal = Self::measure(screen, spec);
        clear(buf, modal.rect, spec.background);
        table::draw_box_border(buf, modal.rect, spec.title, spec.border, spec.background);
        spec.hints.draw(buf, modal.hints, spec.background);
        modal
    }

    pub fn open_fitting(
        buf: &mut Buffer,
        screen: Screen,
        frame: &Frame,
        content: (u16, u16),
        hints: impl Fn(bool) -> HintBar,
    ) -> (Modal, bool) {
        let (content_w, content_h) = content;
        let spec = |bar| ModalSpec {
            title: frame.title,
            border: frame.border,
            background: frame.background,
            content_w,
            content_h,
            hints: bar,
        };
        let calm = hints(false);
        let overflowing = Self::measure(screen, &spec(&calm)).body.height < content_h;
        if !overflowing {
            return (Self::open(buf, screen, &spec(&calm)), false);
        }
        let scrolling = hints(true);
        (Self::open(buf, screen, &spec(&scrolling)), true)
    }

    pub fn scrollbar_x(&self) -> u16 {
        self.rect.right().saturating_sub(BORDER)
    }
}

pub struct Frame<'a> {
    pub title: &'a str,
    pub border: Color,
    pub background: Color,
}

pub fn clear(buf: &mut Buffer, rect: Rect, bg: Color) {
    for y in rect.top()..rect.bottom() {
        for x in rect.left()..rect.right() {
            overdraw(buf, x, y).reset();
            buf[(x, y)].set_bg(bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOSE: &str = "ESC/q close";

    fn spec(hints: &HintBar, content_w: u16, content_h: u16) -> ModalSpec<'_> {
        ModalSpec {
            title: " Box ",
            border: Color::Reset,
            background: Color::Reset,
            content_w,
            content_h,
            hints,
        }
    }

    fn screen(cols: u16, rows: u16) -> Screen {
        Screen::new(
            Rect::new(0, 0, cols, rows),
            Rect::new(0, 0, cols, rows.saturating_sub(4)),
        )
    }

    #[test]
    fn a_roomy_modal_keeps_a_padding_row_between_its_body_and_its_hints() {
        let hints = HintBar::new(CLOSE).action("↑↓ navigate");
        let modal = Modal::measure(screen(100, 30), &spec(&hints, 30, 5));
        assert_eq!(modal.body.height, 5);
        assert_eq!(modal.hints.y, modal.body.bottom() + 1);
        assert_eq!(modal.hints.height, 1);
    }

    #[test]
    fn a_short_screen_shrinks_the_body_but_never_the_hints() {
        let hints = HintBar::new(CLOSE)
            .action("↑↓ navigate")
            .action("ENTER show");
        let modal = Modal::measure(screen(24, 8), &spec(&hints, 30, 20));
        assert_eq!(modal.rect.height, 8);
        assert_eq!(modal.hints.height, hints.height(modal.body.width));
        assert!(modal.body.height >= 1);
        assert!(modal.hints.bottom() <= modal.rect.bottom() - BORDER);
    }

    #[test]
    fn a_box_blanks_a_wide_glyph_whose_right_half_lies_under_its_edge() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 2));
        let style = ratatui::style::Style::default();
        buf.set_string(0, 0, "  彡", style);
        buf.set_string(0, 1, " 彡", style);
        clear(&mut buf, Rect::new(3, 0, 3, 2), Color::Reset);
        assert_eq!(
            buf[(2, 0)].symbol(),
            " ",
            "a terminal would paint the glyph's right half over the box's edge"
        );
        assert_eq!(
            buf[(1, 1)].symbol(),
            "彡",
            "a wide glyph that ends before the box is untouched"
        );
    }

    #[test]
    fn a_tiny_screen_still_shows_one_body_row_and_the_close_hint() {
        let hints = HintBar::new(CLOSE).action("↑↓ navigate");
        let modal = Modal::measure(screen(14, 5), &spec(&hints, 30, 9));
        assert_eq!(modal.body.height, 1);
        assert!(modal.hints.height >= 1);
    }
}
