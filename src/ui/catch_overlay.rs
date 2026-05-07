use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

use crate::entities::{
    fish::{Direction, Fish},
    species::FishSpecies,
};
use crate::loot::{CashValue, JunkSprite, LootKind};
use crate::ui::render_fish_segs;

const BG: Color = Color::Reset;
const RIGHT_PANEL_W: u16 = 34;
const OVERLAY_H: u16 = 7;
const CONTENT_H: u16 = 5;

const BREAD: Color = Color::Rgb(245, 230, 180);
const CHEESE: Color = Color::Rgb(220, 180, 50);
const BURGER: Color = Color::Rgb(140, 80, 30);

pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct CatchState {
    pub loot: LootKind,
    pub fish: Option<Fish>,
    pub name_input: String,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    pub junk_qty: u32,
    blink_timer: f32,
}

impl CatchState {
    pub fn new(loot: LootKind, rng: &mut impl RngExt) -> Self {
        let fish = if let LootKind::Fish(species) = &loot {
            let mut f = Fish::new(*species, String::new(), 0.0, 0.0, rng);
            f.facing = Direction::Right;
            f.velocity.dx = f.velocity.dx.abs();
            Some(f)
        } else {
            None
        };
        Self {
            loot,
            fish,
            name_input: String::new(),
            cursor_pos: 0,
            cursor_visible: true,
            junk_qty: 0,
            blink_timer: 0.0,
        }
    }

    pub fn is_fish(&self) -> bool {
        self.fish.is_some()
    }

    pub fn tick(&mut self, fps: f32) {
        if let Some(ref mut fish) = self.fish {
            fish.tick_animation(1.0 / fps);
        }
        if self.is_fish() {
            self.blink_timer += 1.0;
            let half_period = (fps * 0.5).max(1.0);
            if self.blink_timer >= half_period {
                self.blink_timer = 0.0;
                self.cursor_visible = !self.cursor_visible;
            }
        }
    }

    pub fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.blink_timer = 0.0;
    }
}

pub struct CatchOverlay<'a> {
    state: &'a CatchState,
}

impl<'a> CatchOverlay<'a> {
    pub fn new(state: &'a CatchState) -> Self {
        Self { state }
    }
}

impl Widget for CatchOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;

        let left_w = left_panel_inner_w(&state.loot, state.fish.as_ref());
        let inner_w = left_w + 1 + RIGHT_PANEL_W;
        let overlay_w = inner_w + 2;

        if area.width < overlay_w || area.height < OVERLAY_H {
            return;
        }

        let ox = area.x + (area.width - overlay_w) / 2;
        let oy = area.y + (area.height - OVERLAY_H) / 2;

        for dy in 0..OVERLAY_H {
            for dx in 0..overlay_w {
                buf[(ox + dx, oy + dy)].reset();
                buf[(ox + dx, oy + dy)].set_bg(BG);
            }
        }

        let panel_sep_x = ox + 1 + left_w;

        draw_border(
            buf,
            ox,
            oy,
            overlay_w,
            OVERLAY_H,
            overlay_title(&state.loot),
        );

        buf[(panel_sep_x, oy + OVERLAY_H - 1)]
            .set_char('┴')
            .set_fg(Color::White)
            .set_bg(BG);

        for row in 1..=CONTENT_H {
            buf[(panel_sep_x, oy + row)]
                .set_char('│')
                .set_fg(Color::White)
                .set_bg(BG);
        }

        draw_left_panel(buf, state, ox + 1, oy + 1, left_w, CONTENT_H);
        draw_right_panel(
            buf,
            state,
            panel_sep_x + 1,
            oy + 1,
            RIGHT_PANEL_W,
            CONTENT_H,
        );
    }
}

fn left_panel_inner_w(loot: &LootKind, fish: Option<&Fish>) -> u16 {
    match loot {
        LootKind::Fish(_) => fish.map_or(10, |f| f.display_width as u16) + 3,
        LootKind::Cash(_) => 5 + 3,
        LootKind::Food(_) => 11 + 3,
        LootKind::Junk(_) => JunkSprite::hook_col() + 3,
    }
}

fn overlay_title(loot: &LootKind) -> &'static str {
    match loot {
        LootKind::Fish(_) => " Fish to the Fishtank! ",
        LootKind::Cash(_) => " Cash to the Fishtank! ",
        LootKind::Food(_) => " Food to the Fishtank! ",
        LootKind::Junk(_) => " Junk to the Fishtank! ",
    }
}

fn draw_border(buf: &mut Buffer, ox: u16, oy: u16, w: u16, h: u16, title: &str) {
    if w < 2 || h < 2 {
        return;
    }
    let style = Style::default().fg(Color::White).bg(BG);
    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let right = ox + w - 1;
    let bottom = oy + h - 1;

    buf[(ox, oy)].set_char('┌').set_style(style);
    buf[(right, oy)].set_char('┐').set_style(style);
    buf[(ox, bottom)].set_char('└').set_style(style);
    buf[(right, bottom)].set_char('┘').set_style(style);

    for dx in 1..w - 1 {
        buf[(ox + dx, oy)].set_char('─').set_style(style);
        buf[(ox + dx, bottom)].set_char('─').set_style(style);
    }
    for dy in 1..h - 1 {
        buf[(ox, oy + dy)].set_char('│').set_style(style);
        buf[(right, oy + dy)].set_char('│').set_style(style);
    }

    if (title.len() as u16 + 4) < w {
        buf.set_string(ox + 2, oy, title, title_style);
    }
}

fn draw_left_panel(buf: &mut Buffer, state: &CatchState, x: u16, y: u16, w: u16, h: u16) {
    if w == 0 || h == 0 {
        return;
    }
    match &state.loot {
        LootKind::Fish(_) => {
            if let Some(ref fish) = state.fish {
                draw_fish_panel(buf, fish, x, y, w, h);
            }
        }
        LootKind::Cash(cv) => draw_cash_panel(buf, *cv, x, y, w, h),
        LootKind::Food(_) => draw_food_panel(buf, x, y, w, h),
        LootKind::Junk(sprite) => draw_junk_panel(buf, sprite, x, y, w, h),
    }
}

fn draw_right_panel(buf: &mut Buffer, state: &CatchState, x: u16, y: u16, w: u16, h: u16) {
    if w == 0 || h == 0 {
        return;
    }
    match &state.loot {
        LootKind::Fish(species) => draw_fish_right_panel(buf, state, *species, x, y, w, h),
        LootKind::Cash(cv) => draw_cash_right_panel(buf, *cv, x, y, w, h),
        LootKind::Food(amount) => draw_food_right_panel(buf, *amount, x, y, w, h),
        LootKind::Junk(_) => draw_junk_right_panel(buf, state.junk_qty, x, y, w, h),
    }
}

fn draw_fish_panel(buf: &mut Buffer, fish: &Fish, x: u16, y: u16, w: u16, h: u16) {
    let fish_dw = fish.display_width as u16;
    let fish_x = x + 1;
    let hook_x = fish_x + fish_dw;

    let row_offset = if h <= 1 {
        0
    } else {
        (h * 2 / 3).min(h - 1).max(1)
    };
    let fish_y = y + row_offset;

    for i in 0..row_offset {
        if hook_x < x + w {
            buf[(hook_x, y + i)]
                .set_char('⎹')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }

    let segs = fish.segments();
    let max_w = (x + w).saturating_sub(fish_x);
    render_fish_segs(buf, &segs, fish_x, fish_y, max_w, BG);

    if hook_x < x + w {
        buf[(hook_x, fish_y)]
            .set_char('J')
            .set_fg(Color::DarkGray)
            .set_bg(BG);
    }
}

fn draw_cash_panel(buf: &mut Buffer, cv: CashValue, x: u16, y: u16, w: u16, h: u16) {
    let sprite_x = x + 1;
    let sprite_h: u16 = 1;
    let vert_pad = (h.saturating_sub(sprite_h)) / 2;
    let sprite_y = y + vert_pad;
    let hook_x = sprite_x + 5;

    for i in 0..vert_pad {
        if hook_x < x + w {
            buf[(hook_x, y + i)]
                .set_char('⎹')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }

    let color = cv.color();
    let sprite = "[ $ ]";
    if sprite_y < y + h {
        for (i, ch) in sprite.chars().enumerate() {
            let col = sprite_x + i as u16;
            if col >= x + w {
                break;
            }
            buf[(col, sprite_y)].set_char(ch).set_fg(color).set_bg(BG);
        }
    }

    if hook_x < x + w && sprite_y < y + h {
        buf[(hook_x, sprite_y)]
            .set_char('J')
            .set_fg(Color::DarkGray)
            .set_bg(BG);
    }
}

fn food_sprite_rows() -> Vec<Vec<(char, Color)>> {
    vec![
        vec![
            (' ', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
        ],
        vec![
            ('/', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('\\', BREAD),
        ],
        vec![
            ('{', CHEESE),
            ('_', CHEESE),
            ('/', CHEESE),
            ('M', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('\\', CHEESE),
            ('_', CHEESE),
            ('}', CHEESE),
        ],
        vec![
            (':', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('M', BURGER),
            ('\\', CHEESE),
            ('_', CHEESE),
            ('/', CHEESE),
            ('M', BURGER),
            ('M', BURGER),
            (':', BURGER),
        ],
        vec![
            ('\\', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('_', BREAD),
            ('/', BREAD),
        ],
    ]
}

fn draw_food_panel(buf: &mut Buffer, x: u16, y: u16, w: u16, h: u16) {
    let sprite_x = x + 1;
    let rows = food_sprite_rows();
    let sprite_h = rows.len() as u16;
    let vert_pad = (h.saturating_sub(sprite_h)) / 2;
    let sprite_y = y + vert_pad;
    let hook_col: u16 = 11;
    let hook_row: u16 = 2;
    let hook_x = sprite_x + hook_col;
    let hook_y = sprite_y + hook_row;
    let lines_above = vert_pad + hook_row;

    for i in 0..lines_above {
        if hook_x < x + w {
            buf[(hook_x, y + i)]
                .set_char('⎹')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let row_y = sprite_y + row_idx as u16;
        if row_y >= y + h {
            break;
        }
        for (col_idx, (ch, color)) in row.iter().enumerate() {
            let col = sprite_x + col_idx as u16;
            if col >= x + w {
                break;
            }
            buf[(col, row_y)].set_char(*ch).set_fg(*color).set_bg(BG);
        }
    }

    if hook_x < x + w && hook_y < y + h {
        buf[(hook_x, hook_y)]
            .set_char('J')
            .set_fg(Color::DarkGray)
            .set_bg(BG);
    }
}

fn draw_junk_panel(buf: &mut Buffer, sprite: &JunkSprite, x: u16, y: u16, w: u16, h: u16) {
    let sprite_x = x + 1;
    let sprite_h = JunkSprite::sprite_height();
    let vert_pad = (h.saturating_sub(sprite_h)) / 2;
    let sprite_y = y + vert_pad;
    let hook_col = JunkSprite::hook_col();
    let hook_row = JunkSprite::hook_row();
    let hook_x = sprite_x + hook_col;
    let hook_y = sprite_y + hook_row;
    let lines_above = vert_pad + hook_row;

    for i in 0..lines_above {
        if hook_x < x + w {
            buf[(hook_x, y + i)]
                .set_char('⎹')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }

    for (row_idx, row) in sprite.rows.iter().enumerate() {
        let row_y = sprite_y + row_idx as u16;
        if row_y >= y + h {
            break;
        }
        let mut col = 0u16;
        for (ch, color) in row {
            let cw = UnicodeWidthChar::width(*ch).unwrap_or(1) as u16;
            if sprite_x + col + cw > x + w {
                break;
            }
            buf[(sprite_x + col, row_y)]
                .set_char(*ch)
                .set_fg(*color)
                .set_bg(BG);
            col += cw;
        }
    }

    if hook_x < x + w && hook_y < y + h {
        buf[(hook_x, hook_y)]
            .set_char('J')
            .set_fg(Color::DarkGray)
            .set_bg(BG);
    }
}

fn draw_fish_right_panel(
    buf: &mut Buffer,
    state: &CatchState,
    species: FishSpecies,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
) {
    let caught_line = format!("{} captured!", species.display_name());
    let white_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let white = Style::default().fg(Color::White).bg(BG);

    if h > 0 {
        buf.set_string(
            x,
            y,
            truncate_to_width("Congratulations!", w as usize),
            white_bold,
        );
    }
    if h > 1 {
        buf.set_string(x, y + 1, truncate_to_width(&caught_line, w as usize), white);
    }
    if h > 3 {
        buf.set_string(x, y + 3, truncate_to_width("Name it", w as usize), white);
    }
    if h > 4 {
        draw_name_input(buf, state, x, y + 4, w);
    }
}

fn draw_cash_right_panel(buf: &mut Buffer, cv: CashValue, x: u16, y: u16, w: u16, h: u16) {
    let white_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let cash_style = Style::default().fg(cv.color()).bg(BG);
    let hint = Style::default().fg(Color::DarkGray).bg(BG);

    if h > 0 {
        buf.set_string(
            x,
            y,
            truncate_to_width("Congratulations!", w as usize),
            white_bold,
        );
    }
    if h > 1 {
        let msg = format!("${} found!", cv.amount());
        buf.set_string(x, y + 1, truncate_to_width(&msg, w as usize), cash_style);
    }
    if h > 3 {
        let text = "Chasing cash, making money.";
        let tx = x + w - (text.len() as u16).min(w);
        buf.set_string(tx, y + 3, text, hint);
    }
    if h > 4 {
        let text = "ESC/q to close";
        let tx = x + w - (text.len() as u16).min(w);
        buf.set_string(tx, y + 4, text, hint);
    }
}

fn draw_food_right_panel(buf: &mut Buffer, amount: u32, x: u16, y: u16, w: u16, h: u16) {
    let white_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let white = Style::default().fg(Color::White).bg(BG);
    let hint = Style::default().fg(Color::DarkGray).bg(BG);

    if h > 0 {
        buf.set_string(
            x,
            y,
            truncate_to_width("Congratulations!", w as usize),
            white_bold,
        );
    }
    if h > 1 {
        let msg = format!("+{} food!", amount);
        buf.set_string(x, y + 1, truncate_to_width(&msg, w as usize), white);
    }
    if h > 4 {
        let text = "ESC/q to close";
        let tx = x + w - (text.len() as u16).min(w);
        buf.set_string(tx, y + 4, text, hint);
    }
}

fn draw_junk_right_panel(buf: &mut Buffer, qty: u32, x: u16, y: u16, w: u16, h: u16) {
    let white = Style::default().fg(Color::White).bg(BG);
    let hint = Style::default().fg(Color::DarkGray).bg(BG);

    if h > 0 {
        buf.set_string(x, y, truncate_to_width("It's junk...", w as usize), white);
    }
    if h > 1 {
        buf.set_string(
            x,
            y + 1,
            truncate_to_width("Added to inventory.", w as usize),
            white,
        );
    }
    if h > 3 {
        let qty_msg = format!("You now have {} of Junk.", qty);
        let qty_msg = truncate_to_width(&qty_msg, w as usize);
        let tx = x + w - (qty_msg.len() as u16).min(w);
        buf.set_string(tx, y + 3, qty_msg, hint);
    }
    if h > 4 {
        let text = "ESC/q to close";
        let tx = x + w - (text.len() as u16).min(w);
        buf.set_string(tx, y + 4, text, hint);
    }
}

const ENTER_HINT: &str = "(ENTER)";

fn draw_name_input(buf: &mut Buffer, state: &CatchState, x: u16, y: u16, w: u16) {
    if w < 3 {
        return;
    }

    let white = Style::default().fg(Color::White).bg(BG);
    let hint_style = Style::default().fg(Color::DarkGray).bg(BG);

    buf[(x, y)].set_char('>').set_style(white);
    buf[(x + 1, y)].set_char(' ').set_style(white);

    let base_x = x + 2;
    let input = &state.name_input;
    let cursor_pos = state.cursor_pos;
    let at_end = cursor_pos == input.len();
    let cursor_col = base_x + cursor_pos as u16;

    if cursor_pos > 0 && base_x < x + w {
        let avail = (x + w - base_x) as usize;
        buf.set_string(
            base_x,
            y,
            truncate_to_width(&input[..cursor_pos], avail),
            white,
        );
    }

    if cursor_col < x + w {
        let ch = if at_end {
            ' '
        } else {
            input[cursor_pos..].chars().next().unwrap_or(' ')
        };
        if state.cursor_visible {
            buf[(cursor_col, y)]
                .set_char(ch)
                .set_fg(Color::Black)
                .set_bg(Color::White);
        } else if !at_end {
            buf[(cursor_col, y)].set_char(ch).set_style(white);
        }
    }

    if !at_end {
        let char_len = input[cursor_pos..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let after = &input[cursor_pos + char_len..];
        let after_col = cursor_col + 1;
        if !after.is_empty() && after_col < x + w {
            let avail = (x + w - after_col) as usize;
            buf.set_string(after_col, y, truncate_to_width(after, avail), white);
        }
    }

    let hint_w = ENTER_HINT.len() as u16;
    let needed = 2 + input.len() as u16 + 1 + hint_w;
    if !input.is_empty() && needed <= w {
        let hint_x = x + w - hint_w;
        buf.set_string(hint_x, y, ENTER_HINT, hint_style);
    }
}

fn truncate_to_width(s: &str, max_w: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + cw > max_w {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}
