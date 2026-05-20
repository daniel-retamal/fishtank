use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::entities::{
    fish::{Direction, Fish},
    species::FishSpecies,
};
use crate::loot::{
    CashValue, ConsumableKind, JunkSprite, LootKind, bait_sprite_rows, coffee_sprite_rows,
};
use crate::ui::{
    render_fish_segs, table,
    text_input::{TextInput, draw_text_cursor},
};

const BG: Color = Color::Reset;
const RIGHT_PANEL_W: u16 = 34;
const OVERLAY_H: u16 = 7;

const NECRO_OVERLAY_H: u16 = 8;
const NECRO_HOOK_COL: u16 = 12;
const NECRO_HOOK_ROW: u16 = 0;
const NECRO_LEFT_PANEL_W: u16 = NECRO_HOOK_COL + 4;
const NECRO_BOOK_COLOR: Color = Color::Red;
const NECRO_EYE_OPEN_CHAR: char = 'ʘ';
const NECRO_EYE_CLOSED_CHAR: char = 'u';
const NECRO_EYE_OPEN_TIME: f32 = 0.7;
const NECRO_EYE_CLOSED_TIME: f32 = 0.2;
const NECRO_EYE_COUNT: usize = 9;

const BREAD: Color = Color::Rgb(245, 230, 180);
const CHEESE: Color = Color::Rgb(220, 180, 50);
const BURGER: Color = Color::Rgb(140, 80, 30);

pub struct CatchState {
    pub loot: LootKind,
    pub fish: Option<Fish>,
    pub name_input: TextInput,
    pub cursor_visible: bool,
    pub item_qty: u32,
    pub anim_phase: bool,
    pub necro_eye_open: Vec<bool>,
    necro_eye_timers: Vec<f32>,
    blink_timer: f32,
    anim_tick: f32,
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
        let necro_eye_open: Vec<bool> =
            (0..NECRO_EYE_COUNT).map(|_| rng.random::<bool>()).collect();
        let necro_eye_timers: Vec<f32> = necro_eye_open
            .iter()
            .map(|&open| {
                if open {
                    rng.random_range(0.0..NECRO_EYE_OPEN_TIME)
                } else {
                    rng.random_range(0.0..NECRO_EYE_CLOSED_TIME)
                }
            })
            .collect();
        Self {
            loot,
            fish,
            name_input: TextInput::new(),
            cursor_visible: true,
            item_qty: 0,
            anim_phase: false,
            necro_eye_open,
            necro_eye_timers,
            blink_timer: 0.0,
            anim_tick: 0.0,
        }
    }

    pub fn is_fish(&self) -> bool {
        self.fish.is_some()
    }

    pub fn tick(&mut self, fps: f32) {
        let dt = 1.0 / fps;
        if let Some(ref mut fish) = self.fish {
            fish.tick_animation(dt);
        }
        if self.is_fish() {
            self.blink_timer += 1.0;
            let half_period = (fps * 0.5).max(1.0);
            if self.blink_timer >= half_period {
                self.blink_timer = 0.0;
                self.cursor_visible = !self.cursor_visible;
            }
        }
        if matches!(self.loot, LootKind::Consumable(ConsumableKind::Coffee)) {
            self.anim_tick += 1.0;
            let half = (fps * 0.3).max(1.0);
            if self.anim_tick >= half {
                self.anim_tick = 0.0;
                self.anim_phase = !self.anim_phase;
            }
        }
        if matches!(self.loot, LootKind::Necronomicon) {
            for i in 0..self.necro_eye_timers.len() {
                self.necro_eye_timers[i] -= dt;
                if self.necro_eye_timers[i] <= 0.0 {
                    self.necro_eye_open[i] = !self.necro_eye_open[i];
                    self.necro_eye_timers[i] = if self.necro_eye_open[i] {
                        NECRO_EYE_OPEN_TIME
                    } else {
                        NECRO_EYE_CLOSED_TIME
                    };
                }
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

        let overlay_h = if matches!(state.loot, LootKind::Necronomicon) {
            NECRO_OVERLAY_H
        } else {
            OVERLAY_H
        };
        let content_h = overlay_h - 2;

        let left_w = left_panel_inner_w(&state.loot, state.fish.as_ref());
        let inner_w = left_w + 1 + RIGHT_PANEL_W;
        let overlay_w = inner_w + 2;

        if area.width < overlay_w || area.height < overlay_h {
            return;
        }

        let ox = area.x + (area.width - overlay_w) / 2;
        let oy = area.y + (area.height - overlay_h) / 2;

        for dy in 0..overlay_h {
            for dx in 0..overlay_w {
                buf[(ox + dx, oy + dy)].reset();
                buf[(ox + dx, oy + dy)].set_bg(BG);
            }
        }

        let panel_sep_x = ox + 1 + left_w;
        let border_col = loot_border_color(state);

        table::draw_box_border(
            buf,
            ox,
            oy,
            overlay_w,
            overlay_h,
            overlay_title(&state.loot),
            border_col,
            BG,
        );

        buf[(panel_sep_x, oy + overlay_h - 1)]
            .set_char('┴')
            .set_fg(border_col)
            .set_bg(BG);

        for row in 1..=content_h {
            buf[(panel_sep_x, oy + row)]
                .set_char('│')
                .set_fg(border_col)
                .set_bg(BG);
        }

        draw_left_panel(buf, state, ox + 1, oy + 1, left_w, content_h);
        draw_right_panel(
            buf,
            state,
            panel_sep_x + 1,
            oy + 1,
            RIGHT_PANEL_W,
            content_h,
        );
    }
}

fn left_panel_inner_w(loot: &LootKind, fish: Option<&Fish>) -> u16 {
    match loot {
        LootKind::Fish(_) => fish.map_or(10, |f| f.display_width as u16) + 3,
        LootKind::Cash(_) => 5 + 3,
        LootKind::Food(_) => 11 + 3,
        LootKind::Junk(_) => JunkSprite::hook_col() + 3,
        LootKind::Consumable(kind) => kind.panel_inner_w(),
        LootKind::GoldBar => 5 + 3,
        LootKind::Necronomicon => NECRO_LEFT_PANEL_W,
    }
}

fn overlay_title(loot: &LootKind) -> &'static str {
    match loot {
        LootKind::Fish(_) => " Fish to the Fishtank! ",
        LootKind::Cash(_) => " Cash to the Fishtank! ",
        LootKind::Food(_) => " Food to the Fishtank! ",
        LootKind::Junk(_) => " Junk to the Fishtank! ",
        LootKind::Consumable(_) => " Junk to the Fishtank! ",
        LootKind::GoldBar => " Cash to the Fishtank! ",
        LootKind::Necronomicon => " Junk to the Fishtank! ",
    }
}

fn loot_border_color(state: &CatchState) -> Color {
    match &state.loot {
        LootKind::Fish(species) => match species {
            FishSpecies::Goldenfish => Color::LightYellow,
            FishSpecies::Mutantfish => state.fish.as_ref().map_or(Color::White, |f| f.color),
            _ => Color::White,
        },
        LootKind::Necronomicon => Color::LightRed,
        _ => Color::White,
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
        LootKind::Consumable(kind) => {
            draw_consumable_panel(buf, *kind, state.anim_phase, x, y, w, h)
        }
        LootKind::GoldBar => draw_goldbar_panel(buf, x, y, w, h),
        LootKind::Necronomicon => draw_necro_panel(buf, &state.necro_eye_open, x, y, w, h),
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
        LootKind::Junk(_) => {
            draw_consumable_item_right_panel(buf, "Junk", state.item_qty, x, y, w, h)
        }
        LootKind::Consumable(kind) => {
            draw_consumable_item_right_panel(buf, kind.display_name(), state.item_qty, x, y, w, h)
        }
        LootKind::GoldBar => draw_goldbar_right_panel(buf, x, y, w, h),
        LootKind::Necronomicon => {
            draw_consumable_item_right_panel(buf, "Necronomicon", state.item_qty, x, y, w, h)
        }
    }
}

fn necro_sprite_rows(eye_open: &[bool]) -> Vec<Vec<(char, Color)>> {
    let c = NECRO_BOOK_COLOR;
    let ec = |i: usize| -> char {
        if eye_open.get(i).copied().unwrap_or(true) {
            NECRO_EYE_OPEN_CHAR
        } else {
            NECRO_EYE_CLOSED_CHAR
        }
    };
    vec![
        vec![
            (' ', c),
            (' ', c),
            (' ', c),
            (' ', c),
            (' ', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
        ],
        vec![
            (' ', c),
            (' ', c),
            (' ', c),
            (' ', c),
            ('/', c),
            (' ', c),
            (ec(0), c),
            (' ', c),
            (' ', c),
            (ec(1), c),
            (ec(2), c),
            (' ', c),
            ('/', c),
            (',', c),
        ],
        vec![
            (' ', c),
            (' ', c),
            (' ', c),
            ('/', c),
            (' ', c),
            (ec(3), c),
            (' ', c),
            (ec(4), c),
            (ec(5), c),
            (' ', c),
            (' ', c),
            ('/', c),
            ('/', c),
        ],
        vec![
            (' ', c),
            (' ', c),
            ('/', c),
            (' ', c),
            (ec(6), c),
            (ec(7), c),
            (' ', c),
            (' ', c),
            (ec(8), c),
            (' ', c),
            ('/', c),
            ('/', c),
        ],
        vec![
            (' ', c),
            ('/', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('/', c),
            ('/', c),
        ],
        vec![
            ('(', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('_', c),
            ('(', c),
            ('/', c),
        ],
    ]
}

fn draw_necro_panel(buf: &mut Buffer, eye_open: &[bool], x: u16, y: u16, w: u16, h: u16) {
    let sprite_x = x + 1;
    let rows = necro_sprite_rows(eye_open);
    let sprite_h = rows.len() as u16;
    let vert_pad = (h.saturating_sub(sprite_h)) / 2;
    let sprite_y = y + vert_pad;
    let hook_x = sprite_x + NECRO_HOOK_COL;
    let hook_y = sprite_y + NECRO_HOOK_ROW;
    let lines_above = vert_pad + NECRO_HOOK_ROW;

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

fn draw_fish_panel(buf: &mut Buffer, fish: &Fish, x: u16, y: u16, w: u16, h: u16) {
    let fish_dw = fish.display_width as u16;
    let fish_x = x + 1;
    let hook_x = fish_x + fish_dw;

    let row_offset = if h <= 1 { 0 } else { (h / 2).max(1) };
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

fn draw_consumable_panel(
    buf: &mut Buffer,
    kind: ConsumableKind,
    anim_phase: bool,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
) {
    let rows = match kind {
        ConsumableKind::Coffee => coffee_sprite_rows(anim_phase),
        ConsumableKind::Bait => bait_sprite_rows(),
    };
    let sprite_h = rows.len() as u16;
    let sprite_x = x + 1;
    let vert_pad = (h.saturating_sub(sprite_h)) / 2;
    let sprite_y = y + vert_pad;
    let hook_col = kind.hook_col();
    let hook_row = kind.hook_row();
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
        for (col, (ch, color)) in row.iter().enumerate() {
            if sprite_x + col as u16 >= x + w {
                break;
            }
            buf[(sprite_x + col as u16, row_y)]
                .set_char(*ch)
                .set_fg(*color)
                .set_bg(BG);
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
            let cw = table::visual_width(&ch.to_string()) as u16;
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
            table::truncate_str(&caught_line, w as usize),
            white_bold,
        );
    }
    if h > 2 {
        buf.set_string(x, y + 2, table::truncate_str("Name it", w as usize), white);
    }
    if h > 3 {
        draw_text_cursor(
            buf,
            &state.name_input,
            state.cursor_visible,
            x,
            y + 3,
            w,
            BG,
        );
    }
    if h > 4 {
        let hint = Style::default().fg(Color::DarkGray).bg(BG);
        let right_hint = "ENTER capture";
        let rw = right_hint.len() as u16;
        if rw < w {
            buf.set_string(x + w - rw - 1, y + 4, right_hint, hint);
        }
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
            table::truncate_str("Congratulations!", w as usize),
            white_bold,
        );
    }
    if h > 1 {
        let msg = format!("${} found!", cv.amount());
        buf.set_string(x, y + 1, table::truncate_str(&msg, w as usize), cash_style);
    }
    if h > 3 {
        let text = "Chasing cash, making money.";
        let tx = x + w - 1 - (text.len() as u16).min(w - 1);
        buf.set_string(tx, y + 3, text, hint);
    }
    if h > 4 {
        let text = "ESC/q to close";
        let tx = x + w - 1 - (text.len() as u16).min(w - 1);
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
            table::truncate_str("Congratulations!", w as usize),
            white_bold,
        );
    }
    if h > 1 {
        let msg = format!("+{} food!", amount);
        buf.set_string(x, y + 1, table::truncate_str(&msg, w as usize), white);
    }
    if h > 4 {
        let text = "ESC/q to close";
        let tx = x + w - 1 - (text.len() as u16).min(w - 1);
        buf.set_string(tx, y + 4, text, hint);
    }
}

fn draw_consumable_item_right_panel(
    buf: &mut Buffer,
    item_name: &str,
    qty: u32,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
) {
    let white = Style::default().fg(Color::White).bg(BG);
    let hint = Style::default().fg(Color::DarkGray).bg(BG);

    let line0 = format!("{}!", item_name);
    let line1 = "Added to inventory";
    if h > 0 {
        buf.set_string(x, y, table::truncate_str(&line0, w as usize), white);
    }
    if h > 1 {
        buf.set_string(x, y + 1, table::truncate_str(line1, w as usize), white);
    }
    if h >= 4 {
        let qty_msg = format!("You now have {} {}(s)", qty, item_name);
        let qty_msg = table::truncate_str(&qty_msg, (w - 1) as usize);
        let tx = x + w - 1 - (qty_msg.len() as u16).min(w - 1);
        buf.set_string(tx, y + h - 2, qty_msg, hint);
    }
    if h >= 3 {
        let text = "ESC/q to close";
        let tx = x + w - 1 - (text.len() as u16).min(w - 1);
        buf.set_string(tx, y + h - 1, text, hint);
    }
}

fn draw_goldbar_panel(buf: &mut Buffer, x: u16, y: u16, w: u16, h: u16) {
    let gold = Color::Rgb(255, 215, 0);
    let sprite = "[≡$≡]";
    let sprite_x = x + 1;
    let hook_x = sprite_x + 5;
    let vert_pad = (h.saturating_sub(1)) / 2;
    let sprite_y = y + vert_pad;

    for i in 0..vert_pad {
        if hook_x < x + w {
            buf[(hook_x, y + i)]
                .set_char('⎹')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }

    if sprite_y < y + h {
        for (i, ch) in sprite.chars().enumerate() {
            let col = sprite_x + i as u16;
            if col >= x + w {
                break;
            }
            buf[(col, sprite_y)].set_char(ch).set_fg(gold).set_bg(BG);
        }
        if hook_x < x + w {
            buf[(hook_x, sprite_y)]
                .set_char('J')
                .set_fg(Color::DarkGray)
                .set_bg(BG);
        }
    }
}

fn draw_goldbar_right_panel(buf: &mut Buffer, x: u16, y: u16, w: u16, h: u16) {
    let gold = Color::Rgb(255, 215, 0);
    let s_gold = Style::default()
        .fg(gold)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let s_white = Style::default().fg(Color::White).bg(BG);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BG);

    if h < 2 {
        return;
    }
    buf.set_string(x, y, table::truncate_str("Gold Bar!", w as usize), s_gold);
    buf.set_string(
        x,
        y + 1,
        table::truncate_str(
            &format!("Worth ${}", crate::loot::GOLD_BAR_VALUE),
            w as usize,
        ),
        s_white,
    );
    if h >= 5 {
        buf.set_string(
            x,
            y + h - 1,
            table::truncate_str("ENTER/ESC collect", w as usize),
            s_dim,
        );
    }
}
