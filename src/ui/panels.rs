use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

use crate::ui::hint_bar::HintBar;
use crate::ui::layout::{Screen, body_rows};
use crate::ui::modal::clear;
use crate::ui::table::draw_title_with;

const BORDER: u16 = 1;
const DIVIDER: u16 = 1;
const HINT_PADDING_ROWS: u16 = 1;
const SIDE_FLOOR_ROWS: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    Tab,
    Full,
}

pub struct PanelSpec<'a> {
    pub title: &'a str,
    pub title_style: Style,
    pub border: Style,
    pub background: Color,
    pub side: (u16, u16),
    pub body_w: u16,
    pub body_min_w: u16,
    pub body_rows: &'a dyn Fn(u16) -> u16,
    pub hints: &'a HintBar,
    pub reach: Reach,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Panels {
    pub rect: Rect,
    pub side: Rect,
    pub body: Rect,
    pub hints: Rect,
    pub stacked: bool,
}

impl Panels {
    pub fn measure(screen: Screen, spec: &PanelSpec) -> Panels {
        let side_w = spec.side.0;
        let beside_w =
            BORDER + side_w + DIVIDER + spec.body_w.max(spec.hints.natural_width()) + BORDER;
        let beside_body_w = beside_w
            .min(screen.whole.width)
            .saturating_sub(BORDER * 2 + DIVIDER + side_w);
        if beside_body_w >= spec.body_min_w.min(spec.body_w) {
            return Self::beside(screen, spec, beside_w);
        }
        Self::stacked(screen, spec)
    }

    fn beside(screen: Screen, spec: &PanelSpec, natural_w: u16) -> Panels {
        let (side_w, side_h) = spec.side;
        let width = natural_w.min(screen.whole.width);
        let body_w = width.saturating_sub(BORDER * 2 + DIVIDER + side_w);
        let hint_rows = spec.hints.height(body_w);
        let content = (spec.body_rows)(body_w);
        let right_h = content + HINT_PADDING_ROWS + hint_rows;
        let height = BORDER * 2 + right_h.max(side_h);
        let rect = screen.place(width, height);
        let inner_h = rect.height.saturating_sub(BORDER * 2);
        let hint_h = hint_rows.min(inner_h.saturating_sub(content.min(1)));
        let rest = inner_h - hint_h;
        let body_h = body_rows(content, rest, HINT_PADDING_ROWS);
        let body_x = rect.x + BORDER + side_w + DIVIDER;
        let top = rect.y + BORDER;
        Panels {
            rect,
            side: Rect::new(
                rect.x + BORDER,
                top,
                side_w,
                match spec.reach {
                    Reach::Tab => side_h.min(inner_h),
                    Reach::Full => inner_h,
                },
            ),
            body: Rect::new(body_x, top, body_w, body_h),
            hints: Rect::new(body_x, top + inner_h - hint_h, body_w, hint_h),
            stacked: false,
        }
    }

    fn stacked(screen: Screen, spec: &PanelSpec) -> Panels {
        let (side_w, side_h) = spec.side;
        let width = (BORDER * 2 + side_w.max(spec.body_w).max(spec.hints.natural_width()))
            .min(screen.whole.width);
        let inner_w = width.saturating_sub(BORDER * 2);
        let hint_rows = spec.hints.height(inner_w);
        let content = (spec.body_rows)(inner_w);
        let height = BORDER * 2 + side_h + DIVIDER + content + HINT_PADDING_ROWS + hint_rows;
        let rect = screen.place(width, height);
        let inner_h = rect.height.saturating_sub(BORDER * 2);
        let after_body = inner_h.saturating_sub(hint_rows + DIVIDER + content + HINT_PADDING_ROWS);
        let side_shown = side_h
            .min(after_body.max(SIDE_FLOOR_ROWS))
            .min(inner_h.saturating_sub(hint_rows + DIVIDER + content.min(1)));
        let after_side = inner_h - side_shown - DIVIDER.min(inner_h - side_shown);
        let hint_h = hint_rows.min(after_side.saturating_sub(content.min(1)));
        let rest = after_side - hint_h;
        let body_h = body_rows(content, rest, HINT_PADDING_ROWS);
        let inner_x = rect.x + BORDER;
        let top = rect.y + BORDER;
        let body_y = top + side_shown + DIVIDER;
        Panels {
            rect,
            side: Rect::new(inner_x, top, inner_w.min(side_w.max(inner_w)), side_shown),
            body: Rect::new(inner_x, body_y, inner_w, body_h),
            hints: Rect::new(inner_x, top + inner_h - hint_h, inner_w, hint_h),
            stacked: true,
        }
    }

    pub fn open(buf: &mut Buffer, screen: Screen, spec: &PanelSpec) -> Panels {
        let panels = Self::measure(screen, spec);
        if panels.stacked {
            panels.draw_stacked(buf, spec);
        } else {
            panels.draw_beside(buf, spec);
        }
        draw_title_with(buf, panels.rect, spec.title, spec.title_style);
        spec.hints.draw(buf, panels.hints, spec.background);
        panels
    }

    pub fn divider_x(&self) -> u16 {
        self.body.x - DIVIDER
    }

    pub fn right_x(&self) -> u16 {
        self.rect.right().saturating_sub(BORDER)
    }

    fn bottom_y(&self) -> u16 {
        self.rect.bottom().saturating_sub(BORDER)
    }

    fn tab_bottom_y(&self, spec: &PanelSpec) -> u16 {
        match spec.reach {
            Reach::Full => self.bottom_y(),
            Reach::Tab => (self.side.bottom()).min(self.bottom_y()),
        }
    }

    fn draw_beside(&self, buf: &mut Buffer, spec: &PanelSpec) {
        let border = spec.border;
        let (left, divider, right) = (self.rect.x, self.divider_x(), self.right_x());
        let (top, bottom, tab_bottom) = (self.rect.y, self.bottom_y(), self.tab_bottom_y(spec));
        clear(
            buf,
            Rect::new(left, top, self.rect.width, tab_bottom - top + 1),
            spec.background,
        );
        clear(
            buf,
            Rect::new(divider, top, right - divider + 1, self.rect.height),
            spec.background,
        );
        horizontal(buf, left, right, top, border);
        set(buf, left, top, '┌', border);
        set(buf, divider, top, '┬', border);
        set(buf, right, top, '┐', border);
        for y in top + 1..bottom {
            set(buf, divider, y, '│', border);
            set(buf, right, y, '│', border);
        }
        for y in top + 1..tab_bottom {
            set(buf, left, y, '│', border);
        }
        horizontal(buf, divider, right, bottom, border);
        horizontal(buf, left, divider, tab_bottom, border);
        set(buf, left, tab_bottom, '└', border);
        set(buf, right, bottom, '┘', border);
        let divider_foot = match (tab_bottom == bottom, spec.reach) {
            (true, _) => '┴',
            (false, Reach::Tab) => '┤',
            (false, Reach::Full) => '┴',
        };
        set(buf, divider, tab_bottom, divider_foot, border);
        if tab_bottom != bottom {
            set(buf, divider, bottom, '└', border);
        }
    }

    fn draw_stacked(&self, buf: &mut Buffer, spec: &PanelSpec) {
        let border = spec.border;
        let (left, right, top, bottom) =
            (self.rect.x, self.right_x(), self.rect.y, self.bottom_y());
        clear(buf, self.rect, spec.background);
        horizontal(buf, left, right, top, border);
        horizontal(buf, left, right, bottom, border);
        for y in top + 1..bottom {
            set(buf, left, y, '│', border);
            set(buf, right, y, '│', border);
        }
        set(buf, left, top, '┌', border);
        set(buf, right, top, '┐', border);
        set(buf, left, bottom, '└', border);
        set(buf, right, bottom, '┘', border);
        let separator = self.side.bottom();
        if separator < bottom {
            horizontal(buf, left, right, separator, border);
            set(buf, left, separator, '├', border);
            set(buf, right, separator, '┤', border);
        }
    }
}

fn set(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    buf[(x, y)].set_char(ch).set_style(style);
}

fn horizontal(buf: &mut Buffer, from: u16, to: u16, y: u16, style: Style) {
    for x in from..=to {
        set(buf, x, y, '─', style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOSE: &str = "ESC/q close";
    const SIDE: (u16, u16) = (7, 4);

    fn measure(cols: u16, rows: u16, body_w: u16) -> Panels {
        let hints = HintBar::new(CLOSE).action("↑↓ navigate");
        let rows_for = |_: u16| 3;
        let spec = PanelSpec {
            title: " Shop ",
            title_style: Style::default(),
            border: Style::default(),
            background: Color::Reset,
            side: SIDE,
            body_w,
            body_min_w: 16,
            body_rows: &rows_for,
            hints: &hints,
            reach: Reach::Tab,
        };
        Panels::measure(Screen::only(Rect::new(0, 0, cols, rows)), &spec)
    }

    #[test]
    fn a_wide_screen_puts_the_side_panel_beside_the_body() {
        let panels = measure(100, 30, 34);
        assert!(!panels.stacked);
        assert_eq!(panels.side.width, SIDE.0);
        assert_eq!(panels.body.width, 34);
        assert_eq!(panels.divider_x(), panels.rect.x + 1 + SIDE.0);
    }

    #[test]
    fn a_narrow_screen_wraps_the_side_panel_above_the_body_instead_of_squeezing_it_away() {
        let panels = measure(22, 30, 34);
        assert!(panels.stacked);
        assert_eq!(panels.body.width, 20);
        assert!(panels.body.y > panels.side.bottom());
        assert!(panels.hints.height >= 1);
    }

    #[test]
    fn a_body_that_fits_its_minimum_stays_beside_the_panel() {
        let panels = measure(30, 30, 34);
        assert!(!panels.stacked);
        assert_eq!(panels.body.width, 30 - 3 - SIDE.0);
    }
}
