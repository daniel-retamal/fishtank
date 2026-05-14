use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

const FISH_FORCE: f32 = 0.022;
const DAMPING: f32 = 0.95;
const PLAYER_FORCE: f32 = 0.028;
const MAX_VELOCITY: f32 = 0.05;
const TARGET_DURATION_MIN: f32 = 20.0;
const TARGET_DURATION_MAX: f32 = 50.0;
const REEL_RATE: f32 = 0.009;
const EDGE_DRAIN_RATE: f32 = 0.004;
const EDGE_DRAIN_SLOPE: f32 = 1.0;
const BAD_REEL_PENALTY: f32 = 7.3;
pub const DANGER_THRESHOLD: f32 = 0.10;
pub const COMPLETION_START: f32 = 0.2;
const REEL_PENALTY_THRESHOLD: f32 = 0.45;
const INDICATOR_WARNING_ZONE: f32 = 0.4;
const INDICATOR_DANGER_ZONE: f32 = 0.75;
const COMPLETION_WARN: f32 = 0.35;
const COMPLETION_MID: f32 = 0.66;
const OVERLAY_FILL: f32 = 0.75;
const COMP_W: u16 = 3;
const INDICATOR_FRACTION: f32 = 0.15;
const ART_CONTENT_W: u16 = 16;
const ART_LINES: u16 = 6;
const REEL_HANDLE_FREQ: u32 = 6;
const BG: Color = Color::Reset;

const ART_FRAMES: [[&str; 6]; 5] = [
    [
        "       ﾄ､       ",
        "      ╱  ⟍      ",
        "    ⟋      ⟍    ",
        "(⊂ ' ⊃)     ╲   ",
        "             @  ",
        "             ⎹  ",
    ],
    [
        "       ﾄ､       ",
        "      |  ⟍      ",
        "     |     ⟍    ",
        "  (⊂ ' ⊃)   ╲   ",
        "             @  ",
        "             ⎹  ",
    ],
    [
        "        ﾄ､      ",
        "       j  ⟍     ",
        "      l    ⟍    ",
        "   (⊂ ' ⊃)  ╲   ",
        "             @  ",
        "             ⎹  ",
    ],
    [
        "         ﾄ､     ",
        "        j  ⟍    ",
        "       l    ⟍   ",
        "    (⊂ ' ⊃)  ╲  ",
        "              @ ",
        "              ⎹ ",
    ],
    [
        "           ﾄ､   ",
        "          j  ⟍  ",
        "         l    ⟍ ",
        "       (⊂ ' ⊃) ╲",
        "               @",
        "              ╱ ",
    ],
];

pub struct FishingState {
    pub fish_pos: f32,
    pub completion: f32,
    pub fish_velocity: f32,
    pub target_pos: f32,
    pub target_timer: f32,
    pub danger_timer: f32,
    pub game_over: bool,
    pub captured: bool,
    pub is_reeling: bool,
    pub is_pushing_left: bool,
    pub is_pushing_right: bool,
    pub no_death: bool,
    pub no_fish: bool,
    pub reel_punish_timer: u32,
    pub reel_anim_tick: u32,
}

impl FishingState {
    pub fn new() -> Self {
        Self {
            fish_pos: 0.5,
            completion: COMPLETION_START,
            fish_velocity: 0.0,
            target_pos: 0.5,
            target_timer: 0.0,
            danger_timer: 0.0,
            game_over: false,
            captured: false,
            is_reeling: false,
            is_pushing_left: false,
            is_pushing_right: false,
            no_death: false,
            no_fish: false,
            reel_punish_timer: 0,
            reel_anim_tick: 0,
        }
    }

    pub fn tick(&mut self, fps: f32, coffee_stacks: u32) {
        if self.game_over || self.captured {
            return;
        }

        if !self.no_fish {
            self.target_timer -= 1.0;
            if self.target_timer <= 0.0 {
                let mut rng = rand::rng();
                let r = rng.random::<f32>();
                self.target_pos = if r < 0.35 {
                    rng.random_range(0.05..0.20f32)
                } else if r < 0.70 {
                    rng.random_range(0.80..0.95f32)
                } else {
                    rng.random_range(0.30..0.70f32)
                };
                self.target_timer = rng.random_range(TARGET_DURATION_MIN..TARGET_DURATION_MAX);
            }
            let dx = self.target_pos - self.fish_pos;
            self.fish_velocity += dx.signum() * FISH_FORCE;
        }

        if self.is_pushing_left {
            self.fish_velocity -= PLAYER_FORCE;
        }
        if self.is_pushing_right {
            self.fish_velocity += PLAYER_FORCE;
        }

        self.fish_velocity *= DAMPING;
        self.fish_velocity = self.fish_velocity.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        self.fish_pos = (self.fish_pos + self.fish_velocity).clamp(0.0, 1.0);

        if self.fish_pos <= 0.0 || self.fish_pos >= 1.0 {
            self.fish_velocity *= -0.5;
        }

        let abs_offset = (self.fish_pos - 0.5).abs() * 2.0;
        let drain = (EDGE_DRAIN_SLOPE * abs_offset) * EDGE_DRAIN_RATE;

        let reel_punish = self.is_reeling && abs_offset > REEL_PENALTY_THRESHOLD;

        let reel_rate = REEL_RATE + 0.002 * coffee_stacks as f32;
        if self.is_reeling {
            if reel_punish {
                self.completion -= drain * BAD_REEL_PENALTY;
            } else {
                self.completion += reel_rate;
            }
        } else {
            self.completion -= drain;
        }

        self.reel_punish_timer = if reel_punish {
            self.reel_punish_timer.saturating_add(1)
        } else {
            0
        };

        self.reel_anim_tick = if self.is_reeling {
            self.reel_anim_tick.wrapping_add(1)
        } else {
            0
        };

        self.completion = self.completion.clamp(0.0, 1.0);

        if self.completion <= DANGER_THRESHOLD {
            self.danger_timer += 1.0;
            if self.danger_timer >= fps && !self.no_death {
                self.game_over = true;
            }
        } else {
            self.danger_timer = 0.0;
        }

        if self.completion >= 1.0 {
            self.captured = true;
        }
    }
}

pub struct FishingOverlay<'a> {
    state: &'a FishingState,
}

impl<'a> FishingOverlay<'a> {
    pub fn new(state: &'a FishingState) -> Self {
        Self { state }
    }
}

impl Widget for FishingOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        if area.width < 6 || area.height < 6 {
            return;
        }

        let max_w = ((area.width as f32 * OVERLAY_FILL) as u16)
            .max(6)
            .min(area.width);
        let max_h = ((area.height as f32 * OVERLAY_FILL) as u16)
            .max(6)
            .min(area.height);
        let side = (max_w / 2).min(max_h);
        let overlay_w = (side * 2).max(6);
        let overlay_h = side.max(6);
        let ox = area.x + (area.width - overlay_w) / 2;
        let oy = area.y + (area.height - overlay_h) / 2;

        let inner_w = overlay_w.saturating_sub(2);
        let art_w = inner_w.saturating_sub(1 + COMP_W);
        let art_h = overlay_h.saturating_sub(6);
        let art_vert_pad = art_h.saturating_sub(ART_LINES) / 2;
        let art_x_offset = art_w.saturating_sub(ART_CONTENT_W) / 2;

        let handle_char = if state.is_reeling && (state.reel_anim_tick / REEL_HANDLE_FREQ) % 2 == 1
        {
            'Ə'
        } else {
            '@'
        };

        for dy in 0..overlay_h {
            for dx in 0..overlay_w {
                buf[(ox + dx, oy + dy)].reset();
                buf[(ox + dx, oy + dy)].set_bg(BG);
            }
        }

        let bcolor = state_border_color(state);
        draw_border(buf, ox, oy, overlay_w, overlay_h, bcolor);

        let inner_x = ox + 1;
        let art_y = oy + 1;
        let sep_x = inner_x + art_w;
        let comp_x = sep_x + 1;
        let frame = &ART_FRAMES[art_frame_idx(state.fish_pos)];
        let art_style = Style::default().fg(Color::White).bg(BG);

        for row in 0..art_h {
            let y = art_y + row;

            let art_line = row.checked_sub(art_vert_pad).and_then(|r| {
                if r < ART_LINES {
                    Some(frame[r as usize])
                } else {
                    None
                }
            });

            for dx in 0..art_w {
                buf[(inner_x + dx, y)].set_char(' ').set_style(art_style);
            }
            if let Some(line) = art_line {
                let content_w = ART_CONTENT_W.min(art_w.saturating_sub(art_x_offset));
                let line_with_handle: String;
                let line_final = if handle_char != '@' && line.contains('@') {
                    line_with_handle = line.replace('@', "Ə");
                    &line_with_handle
                } else {
                    line
                };
                draw_art_content(
                    buf,
                    inner_x + art_x_offset,
                    y,
                    line_final,
                    content_w,
                    art_style,
                );
            }

            if sep_x < ox + overlay_w {
                buf[(sep_x, y)]
                    .set_char('│')
                    .set_fg(Color::DarkGray)
                    .set_bg(BG);
            }

            if comp_x < ox + overlay_w {
                draw_comp_row(buf, comp_x, y, row, art_h, state);
            }
        }

        let sep_y = art_y + art_h;
        draw_inner_separator(buf, ox, sep_y, overlay_w, bcolor);

        let ctrl_y = sep_y + 1;
        draw_control_bar(buf, inner_x, ctrl_y, inner_w, state);

        let sep2_y = ctrl_y + 1;
        draw_inner_separator(buf, ox, sep2_y, overlay_w, bcolor);

        let footer_y = sep2_y + 1;
        draw_footer(buf, inner_x, footer_y, inner_w);
    }
}

fn art_frame_idx(fish_pos: f32) -> usize {
    if fish_pos < 0.2 {
        0
    } else if fish_pos < 0.4 {
        1
    } else if fish_pos < 0.6 {
        2
    } else if fish_pos < 0.8 {
        3
    } else {
        4
    }
}

fn state_border_color(state: &FishingState) -> Color {
    if state.danger_timer > 0.0 {
        if (state.danger_timer as u32) % 6 < 3 {
            Color::LightRed
        } else {
            Color::White
        }
    } else if state.reel_punish_timer > 0 {
        if state.reel_punish_timer % 6 < 3 {
            Color::LightRed
        } else {
            Color::White
        }
    } else {
        Color::White
    }
}

fn draw_border(buf: &mut Buffer, ox: u16, oy: u16, w: u16, h: u16, border_color: Color) {
    if w < 2 || h < 2 {
        return;
    }
    let border_style = Style::default().fg(border_color).bg(BG);
    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let right = ox + w - 1;
    let bottom = oy + h - 1;

    buf[(ox, oy)].set_char('┌').set_style(border_style);
    buf[(right, oy)].set_char('┐').set_style(border_style);
    buf[(ox, bottom)].set_char('└').set_style(border_style);
    buf[(right, bottom)].set_char('┘').set_style(border_style);

    for dx in 1..w - 1 {
        buf[(ox + dx, oy)].set_char('─').set_style(border_style);
        buf[(ox + dx, bottom)].set_char('─').set_style(border_style);
    }
    for dy in 1..h - 1 {
        buf[(ox, oy + dy)].set_char('│').set_style(border_style);
        buf[(right, oy + dy)].set_char('│').set_style(border_style);
    }

    let title = " Fishing ";
    if (title.len() as u16 + 4) < w {
        buf.set_string(ox + 2, oy, title, title_style);
    }
}

fn draw_inner_separator(buf: &mut Buffer, ox: u16, y: u16, w: u16, color: Color) {
    if w < 2 {
        return;
    }
    let style = Style::default().fg(color).bg(BG);
    buf[(ox, y)].set_char('├').set_style(style);
    buf[(ox + w - 1, y)].set_char('┤').set_style(style);
    for dx in 1..w - 1 {
        buf[(ox + dx, y)].set_char('─').set_style(style);
    }
}

fn draw_art_content(buf: &mut Buffer, x: u16, y: u16, line: &str, max_w: u16, style: Style) {
    let mut col = 0u16;
    for ch in line.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
        if col + cw > max_w {
            break;
        }
        buf[(x + col, y)].set_char(ch).set_style(style);
        if cw == 2 && col + 1 < max_w {
            buf[(x + col + 1, y)].set_char(' ').set_style(style);
        }
        col += cw;
    }
}

fn draw_comp_row(buf: &mut Buffer, x: u16, y: u16, row: u16, art_h: u16, state: &FishingState) {
    let in_danger = state.danger_timer > 0.0;
    let danger_flash = (state.danger_timer as u32) % 6 < 3;

    let empty_rows = (art_h as f32 * (1.0 - state.completion)) as u16;
    let filled = row >= empty_rows;

    let (content, fg) = if state.captured {
        ("███", Color::LightYellow)
    } else if filled {
        let c = if state.completion <= DANGER_THRESHOLD {
            if in_danger && danger_flash {
                Color::LightRed
            } else {
                Color::Red
            }
        } else if state.completion < COMPLETION_WARN {
            Color::Yellow
        } else if state.completion < COMPLETION_MID {
            Color::Green
        } else {
            Color::LightGreen
        };
        ("███", c)
    } else {
        ("   ", BG)
    };

    buf.set_string(x, y, content, Style::default().fg(fg).bg(BG));
}

fn draw_control_bar(buf: &mut Buffer, x: u16, y: u16, inner_w: u16, state: &FishingState) {
    if inner_w < 4 {
        return;
    }
    let indicator_w = ((inner_w as f32 * INDICATOR_FRACTION) as u16).max(1) | 1;
    let movable = inner_w.saturating_sub(2 + indicator_w);
    let indicator_col = (state.fish_pos * movable as f32) as u16;

    let offset = (state.fish_pos - 0.5).abs() * 2.0;
    let indicator_color = if state.captured {
        Color::LightYellow
    } else if offset > INDICATOR_DANGER_ZONE {
        Color::LightRed
    } else if offset > INDICATOR_WARNING_ZONE {
        Color::LightYellow
    } else {
        Color::LightGreen
    };

    let bracket_style = Style::default().fg(Color::DarkGray).bg(BG);
    let line_style = Style::default().fg(Color::DarkGray).bg(BG);
    let indicator_style = Style::default()
        .fg(indicator_color)
        .add_modifier(Modifier::BOLD)
        .bg(BG);

    buf[(x, y)].set_char('[').set_style(bracket_style);
    buf[(x + inner_w - 1, y)]
        .set_char(']')
        .set_style(bracket_style);

    let ind_start = 1 + indicator_col;
    let ind_end = ind_start + indicator_w;
    for dx in 1..inner_w - 1 {
        if dx >= ind_start && dx < ind_end {
            buf[(x + dx, y)].set_char('█').set_style(indicator_style);
        } else {
            buf[(x + dx, y)].set_char('─').set_style(line_style);
        }
    }
}

fn draw_footer(buf: &mut Buffer, x: u16, y: u16, inner_w: u16) {
    let style = Style::default().fg(Color::DarkGray).bg(BG);
    let left = " ←→ control the fish  ↓ reel";
    let right = "ESC/q close";
    let total = inner_w as usize;
    buf.set_string(x, y, truncate_to_width(left, total), style);
    let left_w = visual_width(left);
    let right_w = visual_width(right);
    if left_w + 4 + right_w <= total {
        buf.set_string(x + total as u16 - right_w as u16 - 1, y, right, style);
    }
}

fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
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
