use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

use crate::colors::{GRAY, PINK, WHITE, YELLOW};

use crate::{
    entities::bubble::Bubble,
    entities::cow::{Cow, build_speech_bubble, cow_sprite},
    entities::food::Food,
    entities::glistening::{color_for_glisten, derive_glistening_palette},
    entities::plant::Seaweed,
    entities::ufo::{Ufo, ufo_sprite},
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    fishes::unfish::{
        BALL_BASE, BALL_CENTER_ROW, BALL_EYE_COL, BALL_EYE_ROW, BALL_WIDTH, SKULL_CENTER_ROW,
        SKULL_CLOSED, SKULL_OPEN, SKULL_WIDTH, UNFISH_BODY_COLOR, UNFISH_EYE_COLOR, UnfishKind,
        is_multi_row,
    },
    tank::{Tank, TankKind},
    tanks::alien::{AlienPyramid, AlienStar, pyramid_canvas_w, pyramid_lines},
    tanks::coral::{
        CORAL_A_LINES, CORAL_A_ROWS, CORAL_COLOR, CoralAlgaeInstance, CoralStructure,
        FLOOR_ALGAE_A, FLOOR_ALGAE_COLOR, FloorAlgae, algae_art, algae_row_shift, mirror_char,
        query_anim_char, vert_wave_char,
    },
    tanks::haunted::{
        BARS_PER_BLOCK, BAT_COLOR, BAT_SPRITE, Bat, GATE_BAR_PIPE_ROWS, GATE_BAR_TILE, GATE_BAR_W,
        GATE_BLOCK_ROWS, GATE_BLOCK_TILE, GATE_BLOCK_W, GATE_COLOR, GATE_FLOOR_CHAR,
        GATE_FLOOR_COLOR, GRAVE_TRANSPARENT, Ghost, HauntedBackground,
    },
    tanks::hell::{
        FACE_COLOR, H_WAVE_AMPLITUDE, H_WAVE_ROW_SPREAD, HellBackground, RANDOM_FACE_COLOR,
    },
    tanks::void::{VOID_EYE_CENTER_X, VOID_EYE_VERTICAL_OFFSET, VoidBackground},
    void_ritual::VOID_TEXT_BELOW_EYE_OFFSET,
};

pub struct TankView<'a> {
    tank: &'a Tank,
    show_names: bool,
    ritual_text: Option<[Option<String>; 2]>,
}

impl<'a> TankView<'a> {
    pub fn new(tank: &'a Tank, show_names: bool) -> Self {
        Self {
            tank,
            show_names,
            ritual_text: None,
        }
    }

    pub fn with_ritual(mut self, text: [Option<String>; 2]) -> Self {
        self.ritual_text = Some(text);
        self
    }
}

impl Widget for TankView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ritual_active = self.ritual_text.is_some() && self.tank.kind == TankKind::Void;

        match self.tank.kind {
            TankKind::Base => {
                for plant in &self.tank.plants {
                    if plant.x >= area.width as i32 {
                        break;
                    }
                    render_seaweed(plant, area, buf);
                }
            }
            TankKind::CoralReef => {
                for coral in &self.tank.corals {
                    if coral.base_x >= area.width as i32 {
                        break;
                    }
                    render_coral_structure(coral, area, buf);
                }
                for coral in &self.tank.corals {
                    if coral.base_x >= area.width as i32 {
                        break;
                    }
                    render_coral_algae_all(coral, area, buf);
                }
            }
            TankKind::Hell => {
                if let Some(bg) = &self.tank.hell_bg {
                    render_hell_background(bg, area, buf);
                }
                for plant in &self.tank.hell_plants {
                    if plant.x >= area.width as i32 {
                        break;
                    }
                    render_seaweed(plant, area, buf);
                }
            }
            TankKind::Void => {
                if let Some(bg) = &self.tank.void_bg {
                    if ritual_active {
                        render_void_background_at(bg, 0, area, buf);
                    } else {
                        render_void_background(bg, area, buf);
                    }
                }
            }
            TankKind::Alien => {
                if let Some(bg) = &self.tank.alien_bg {
                    for star in &bg.stars {
                        render_alien_star(star, area, buf);
                    }
                    for tentacle in &bg.tentacles {
                        if tentacle.x >= area.width as i32 {
                            break;
                        }
                        render_seaweed(tentacle, area, buf);
                    }
                    for pyramid in &bg.pyramids {
                        if pyramid.base_x >= area.width as i32 {
                            break;
                        }
                        render_alien_pyramid(pyramid, bg.color.pyramid_color(), area, buf);
                    }
                }
            }
            TankKind::Haunted => {
                render_haunted_gate(area, buf, self.tank.haunted_bg.as_ref());
                if let Some(bg) = &self.tank.haunted_bg {
                    let bottom_y = area.y as i32 + area.height as i32 - 1;
                    for grave in &bg.graves {
                        if grave.base_x >= area.width as i32 {
                            break;
                        }
                        render_opaque_grid(&grave.rows(), grave.base_x, bottom_y, area, buf);
                    }
                    for pumpkin in &bg.pumpkins {
                        if pumpkin.base_x >= area.width as i32 {
                            continue;
                        }
                        render_opaque_grid(&pumpkin.rows(), pumpkin.base_x, bottom_y, area, buf);
                    }
                }
            }
            TankKind::Candy => {
                if let Some(bg) = &self.tank.candy_bg {
                    let bottom_y = area.y as i32 + area.height as i32 - 1;
                    for plant in &self.tank.plants {
                        if plant.x >= area.width as i32 {
                            break;
                        }
                        render_seaweed(plant, area, buf);
                    }
                    for plant in &bg.plants {
                        if plant.x >= area.width as i32 {
                            break;
                        }
                        render_opaque_grid(&plant.rows(), plant.x, bottom_y, area, buf);
                    }
                    let sway = self.tank.candy_man_sway();
                    for deco in &bg.decos {
                        if deco.x >= area.width as i32 {
                            break;
                        }
                        render_opaque_grid(&deco.rows(sway), deco.x, bottom_y, area, buf);
                    }
                }
            }
        }
        for bubble in &self.tank.bubbles {
            render_bubble(bubble, area, buf);
        }
        if let Some(bg) = &self.tank.haunted_bg {
            for bat in &bg.bats {
                render_bat(bat, area, buf);
            }
            for ghost in &bg.ghosts {
                render_ghost(ghost, area, buf);
            }
        }
        for cow in &self.tank.cows {
            render_cow(cow, area, buf);
        }
        for food in &self.tank.food {
            render_food(food, area, buf);
        }
        for fish in &self.tank.fish {
            render_fish(fish, area, buf);
        }
        if self.tank.kind == TankKind::CoralReef {
            for fa in &self.tank.floor_algae {
                if fa.base_x >= area.width as i32 {
                    break;
                }
                render_floor_algae(fa, area, buf);
            }
        }
        if let Some(ufo) = &self.tank.ufo {
            render_ufo(ufo, area, buf);
        }
        if self.show_names {
            for fish in &self.tank.fish {
                render_fish_name(fish, area, buf);
            }
            for cow in &self.tank.cows {
                render_cow_name(cow, area, buf);
            }
        }
        for cow in &self.tank.cows {
            render_cow_speech(cow, area, buf);
        }

        if ritual_active {
            for y in area.y..area.bottom() {
                for x in area.x..area.right() {
                    buf[(x, y)].set_style(Style::default().add_modifier(Modifier::DIM));
                }
            }
            if let Some(bg) = &self.tank.void_bg {
                render_void_background_at(bg, 0, area, buf);
            }
            if let Some(text) = &self.ritual_text {
                render_ritual_text(text, area, buf);
            }
        }
    }
}

fn art_canvas_w(lines: &[&str]) -> i32 {
    lines
        .iter()
        .map(|l| {
            l.chars()
                .map(|c| UnicodeWidthChar::width(c).unwrap_or(1) as i32)
                .sum::<i32>()
        })
        .max()
        .unwrap_or(0)
}

fn display_w(line: &str) -> i32 {
    line.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1) as i32)
        .sum()
}

fn render_seaweed(seaweed: &dyn Seaweed, area: Rect, buf: &mut Buffer) {
    let base_x = seaweed.x();
    for row in 0..seaweed.height() {
        let (x_offset, ch) = seaweed.segment_at(row);
        let x = base_x + x_offset;
        let y = area.height as i32 - 1 - row as i32;
        if y < 0 {
            break;
        }
        if x >= 0 && x < area.width as i32 {
            buf[(area.x + x as u16, area.y + y as u16)]
                .set_char(ch)
                .set_fg(seaweed.color_at(row));
        }
    }
}

fn render_coral_line(
    line: &str,
    mirrored: bool,
    start_x: i32,
    canvas_w: i32,
    screen_y: i32,
    area: Rect,
    buf: &mut Buffer,
) {
    let style = Style::new().fg(CORAL_COLOR);
    if mirrored {
        let line_w = display_w(line);
        let effective_x = start_x + canvas_w - line_w;
        let pairs: Vec<(char, i32)> = line
            .chars()
            .map(|c| {
                (
                    mirror_char(c),
                    UnicodeWidthChar::width(c).unwrap_or(1) as i32,
                )
            })
            .rev()
            .collect();
        let mut col = 0i32;
        for (ch, w) in pairs {
            if ch != ' ' && ch != 'X' {
                let sx = effective_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(ch)
                        .set_style(style);
                }
            }
            col += w;
        }
    } else {
        let mut col = 0i32;
        for ch in line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
            if ch != ' ' && ch != 'X' {
                let sx = start_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(ch)
                        .set_style(style);
                }
            }
            col += w;
        }
    }
}

fn render_coral_structure(coral: &CoralStructure, area: Rect, buf: &mut Buffer) {
    let canvas_w = art_canvas_w(CORAL_A_LINES);
    for (row_idx, line) in CORAL_A_LINES.iter().enumerate() {
        let screen_y = area.y as i32 + area.height as i32 - CORAL_A_ROWS as i32 + row_idx as i32;
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        let start_x = area.x as i32 + coral.base_x;
        render_coral_line(line, coral.mirrored, start_x, canvas_w, screen_y, area, buf);
    }
}

#[derive(Clone, Copy)]
struct LinePlacement {
    start_x: i32,
    screen_y: i32,
    canvas_w: i32,
}

fn render_algae_line(
    line: &str,
    li: usize,
    instance: &CoralAlgaeInstance,
    placement: LinePlacement,
    area: Rect,
    buf: &mut Buffer,
) {
    let LinePlacement {
        start_x,
        screen_y,
        canvas_w,
    } = placement;
    let style = Style::new()
        .fg(instance.color)
        .remove_modifier(Modifier::all());
    if instance.mirrored {
        let line_w = display_w(line);
        let effective_x = start_x + canvas_w - line_w;
        let pairs: Vec<(char, i32)> = {
            let mut col = 0usize;
            let mut v: Vec<(char, i32)> = Vec::new();
            for ch in line.chars() {
                let w = UnicodeWidthChar::width(ch).unwrap_or(1);
                let ac = query_anim_char(&instance.anim, li, col, ch);
                v.push((ac, w as i32));
                col += w;
            }
            v.reverse();
            for (c, _) in &mut v {
                *c = mirror_char(*c);
            }
            v
        };
        let mut col = 0i32;
        for (ch, w) in pairs {
            if ch != ' ' {
                let sx = effective_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(ch)
                        .set_style(style);
                }
            }
            col += w;
        }
    } else {
        let mut col = 0i32;
        for ch in line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
            let ac = query_anim_char(&instance.anim, li, col as usize, ch);
            if ac != ' ' {
                let sx = start_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(ac)
                        .set_style(style);
                }
            }
            col += w;
        }
    }
}

fn render_coral_algae_all(coral: &CoralStructure, area: Rect, buf: &mut Buffer) {
    for instance in &coral.algae {
        let anchor_screen_y =
            area.y as i32 + area.height as i32 - CORAL_A_ROWS as i32 + instance.art_row as i32;
        let art = algae_art(instance.algae_idx);
        let n = art.len();
        let bottom_w: i32 = art[n - 1]
            .chars()
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(1) as i32)
            .sum();
        let canvas_w = art_canvas_w(art);
        let anchor_screen_x = if instance.mirrored {
            area.x as i32 + coral.base_x + instance.art_col as i32 - canvas_w + bottom_w / 2
        } else {
            area.x as i32 + coral.base_x + instance.art_col as i32 - bottom_w / 2
        };
        for (li, line) in art.iter().enumerate() {
            let screen_y = anchor_screen_y - (n - 1 - li) as i32;
            if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
                continue;
            }
            let shift = algae_row_shift(&instance.anim, li);
            render_algae_line(
                line,
                li,
                instance,
                LinePlacement {
                    start_x: anchor_screen_x + shift,
                    screen_y,
                    canvas_w,
                },
                area,
                buf,
            );
        }
    }
}

fn render_floor_algae(fa: &FloorAlgae, area: Rect, buf: &mut Buffer) {
    let n = FLOOR_ALGAE_A.len();
    let canvas_w = art_canvas_w(FLOOR_ALGAE_A);
    let style = Style::new()
        .fg(FLOOR_ALGAE_COLOR)
        .remove_modifier(Modifier::all());
    for (li, line) in FLOOR_ALGAE_A.iter().enumerate() {
        let screen_y = area.y as i32 + area.height as i32 - n as i32 + li as i32;
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        let start_x = area.x as i32 + fa.base_x;
        if fa.mirrored {
            let line_w = display_w(line);
            let effective_x = start_x + canvas_w - line_w;
            let pairs: Vec<(char, i32)> = line
                .chars()
                .map(|c| {
                    let ac = vert_wave_char(c, li, fa.phase);
                    (
                        mirror_char(ac),
                        UnicodeWidthChar::width(c).unwrap_or(1) as i32,
                    )
                })
                .rev()
                .collect();
            let mut col = 0i32;
            for (ch, w) in pairs {
                if ch != ' ' {
                    let sx = effective_x + col;
                    if sx >= area.x as i32 && sx < area.right() as i32 {
                        buf[(sx as u16, screen_y as u16)]
                            .set_char(ch)
                            .set_style(style);
                    }
                }
                col += w;
            }
        } else {
            let mut col = 0i32;
            for ch in line.chars() {
                let w = UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
                let ac = vert_wave_char(ch, li, fa.phase);
                if ac != ' ' {
                    let sx = start_x + col;
                    if sx >= area.x as i32 && sx < area.right() as i32 {
                        buf[(sx as u16, screen_y as u16)]
                            .set_char(ac)
                            .set_style(style);
                    }
                }
                col += w;
            }
        }
    }
}

fn render_bubble(bubble: &Bubble, area: Rect, buf: &mut Buffer) {
    let x = area.x + bubble.position.x as u16;
    let y = area.y + bubble.position.y as u16;
    if x < area.right() && y < area.bottom() {
        buf[(x, y)]
            .set_char(bubble.bubble_char)
            .set_style(Style::new().fg(bubble.color).add_modifier(Modifier::DIM));
    }
}

fn render_food(food: &Food, area: Rect, buf: &mut Buffer) {
    let x = area.x + food.position.x as u16;
    let y = area.y + food.position.y as u16;
    if x < area.right() && y < area.bottom() {
        let color = if food.is_candy { PINK } else { YELLOW };
        buf[(x, y)]
            .set_char(food.food_char)
            .set_style(Style::new().fg(color).remove_modifier(Modifier::all()));
    }
}

fn render_fish_name(fish: &Fish, area: Rect, buf: &mut Buffer) {
    if fish.is_invisible() {
        return;
    }
    if let Some(ref us) = fish.unfish_state
        && us.kind == UnfishKind::Worm
    {
        render_worm_name_portal(fish, area, buf);
        return;
    }

    let name_y_i32 = if fish.species == FishSpecies::Unfish {
        if let Some(ref us) = fish.unfish_state {
            if is_multi_row(us.kind) {
                let center_row = unfish_center_row(us.kind);
                area.y as i32 + fish.position.y as i32 - center_row - 1
            } else {
                area.y as i32 + fish.position.y as i32 - 1
            }
        } else {
            area.y as i32 + fish.position.y as i32 - 1
        }
    } else {
        area.y as i32 + fish.position.y as i32 - 1
    };

    if name_y_i32 < area.y as i32 || name_y_i32 >= area.bottom() as i32 {
        return;
    }
    let name_y = name_y_i32 as u16;

    let fish_center = fish.position.x as i32 + fish.display_width as i32 / 2;
    let name_start = fish_center - fish.name.len() as i32 / 2;

    for (i, ch) in fish.name.chars().enumerate() {
        let x = name_start + i as i32;
        if x < 0 {
            continue;
        }
        let abs_x = area.x + x as u16;
        if abs_x >= area.right() {
            break;
        }
        buf[(abs_x, name_y)]
            .set_char(ch)
            .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
    }
}

fn render_fish(fish: &Fish, area: Rect, buf: &mut Buffer) {
    if fish.is_invisible() {
        return;
    }
    if fish.species == FishSpecies::Unfish
        && let Some(ref us) = fish.unfish_state
    {
        if is_multi_row(us.kind) {
            render_multi_row_unfish(fish, area, buf);
            return;
        }
        if us.kind == UnfishKind::Worm {
            render_worm_portal(fish, area, buf);
            return;
        }
    }

    let segs = fish.segments();
    let y = area.y + fish.position.y as u16;

    if y >= area.bottom() {
        return;
    }

    let x_base = area.x + fish.position.x as u16;
    let mut col = 0u16;
    for (ch, color) in &segs {
        let x = x_base + col;
        if x >= area.right() {
            break;
        }
        buf[(x, y)]
            .set_char(*ch)
            .set_style(Style::new().fg(*color).remove_modifier(Modifier::all()));
        col += UnicodeWidthChar::width(*ch).unwrap_or(1) as u16;
    }
}

fn unfish_center_row(kind: UnfishKind) -> i32 {
    match kind {
        UnfishKind::Skull => SKULL_CENTER_ROW,
        _ => BALL_CENTER_ROW,
    }
}

fn interior_col_range(line: &str) -> Option<(usize, usize)> {
    let first = line
        .chars()
        .enumerate()
        .find(|(_, c)| *c != ' ')
        .map(|(i, _)| i)?;
    let last = line
        .chars()
        .rev()
        .position(|c| c != ' ')
        .map(|rev_pos| line.chars().count() - 1 - rev_pos)?;
    if last <= first + 1 {
        return None;
    }
    Some((first + 1, last - 1))
}

#[derive(Clone, Copy)]
struct SpriteStyle<'a> {
    eye_overrides: &'a [(usize, char, Color)],
    body_color: Color,
    interior_fg: Option<Color>,
    glisten_colors: &'a [Color],
    color_patches: &'a [(usize, Color)],
}

fn render_sprite_row(
    line: &str,
    base_x: i32,
    screen_y: i32,
    area: Rect,
    buf: &mut Buffer,
    style: &SpriteStyle,
) {
    let SpriteStyle {
        eye_overrides,
        body_color,
        interior_fg,
        glisten_colors,
        color_patches,
    } = *style;
    let sy = screen_y as u16;
    let interior = interior_col_range(line);
    for (char_idx, ch) in line.chars().enumerate() {
        let override_entry = eye_overrides.iter().find(|(col, _, _)| *col == char_idx);
        if ch == ' ' && override_entry.is_none() {
            if let Some((inner_start, inner_end)) = interior
                && char_idx >= inner_start
                && char_idx <= inner_end
            {
                let sx = base_x + char_idx as i32;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    let style = match interior_fg {
                        Some(c) => Style::default().fg(c).add_modifier(Modifier::DIM),
                        None => Style::default().add_modifier(Modifier::DIM),
                    };
                    buf[(sx as u16, sy)].set_style(style);
                }
            }
            continue;
        }
        let sx = base_x + char_idx as i32;
        if sx < area.x as i32 || sx >= area.right() as i32 {
            continue;
        }
        let (draw_ch, color) = override_entry
            .map(|&(_, ec, c)| (ec, c))
            .unwrap_or_else(|| {
                let c = glisten_colors
                    .get(char_idx)
                    .copied()
                    .or_else(|| {
                        color_patches
                            .iter()
                            .rev()
                            .find(|&&(pos, _)| pos == char_idx)
                            .map(|&(_, c)| c)
                    })
                    .unwrap_or(body_color);
                (ch, c)
            });
        buf[(sx as u16, sy)].set_char(draw_ch).set_fg(color);
    }
}

pub(crate) fn render_multi_row_unfish_at(
    fish: &Fish,
    base_x: i32,
    base_y: i32,
    area: Rect,
    buf: &mut Buffer,
) {
    let unfish_state = match fish.unfish_state.as_ref() {
        Some(s) => s,
        None => return,
    };
    let eye_ch = if unfish_state.eye.is_open { '0' } else { '-' };

    match unfish_state.kind {
        UnfishKind::Ball | UnfishKind::Skull => {
            let body_color = unfish_state.slime_body_color.unwrap_or(UNFISH_BODY_COLOR);
            let interior_fg = unfish_state
                .slime_body_color
                .filter(|&c| c != UNFISH_BODY_COLOR);
            let sprite_w = if unfish_state.kind == UnfishKind::Ball {
                BALL_WIDTH as usize
            } else {
                SKULL_WIDTH as usize
            };
            let glisten_colors: Vec<Color> = if unfish_state.slime_glisten_enabled {
                let (base, mid, peak_default) = derive_glistening_palette(body_color);
                let peak = unfish_state.slime_glisten_color.unwrap_or(peak_default);
                (0..sprite_w)
                    .map(|i| {
                        color_for_glisten(
                            unfish_state.slime_glisten_mode,
                            unfish_state.slime_glisten_phase,
                            i,
                            sprite_w,
                            base,
                            mid,
                            peak,
                        )
                    })
                    .collect()
            } else {
                vec![]
            };
            let lines: &[&str] = if unfish_state.kind == UnfishKind::Ball {
                &BALL_BASE
            } else if unfish_state.wings.is_open {
                &SKULL_OPEN
            } else {
                &SKULL_CLOSED
            };
            for (row_idx, line) in lines.iter().enumerate() {
                let screen_y = base_y + row_idx as i32;
                if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
                    continue;
                }
                let mut eye_overrides: Vec<(usize, char, Color)> = Vec::new();
                let eye_color = unfish_state.slime_eye_color.unwrap_or(UNFISH_EYE_COLOR);
                if unfish_state.kind == UnfishKind::Ball
                    && unfish_state.ball_has_center_eye
                    && row_idx == BALL_EYE_ROW
                {
                    eye_overrides.push((BALL_EYE_COL - 1, '(', eye_color));
                    eye_overrides.push((BALL_EYE_COL, eye_ch, eye_color));
                    eye_overrides.push((BALL_EYE_COL + 1, ')', eye_color));
                }
                for eye in &unfish_state.floating_eyes {
                    if eye.row != row_idx {
                        continue;
                    }
                    let col = eye.col.round() as i32;
                    let ec = if eye.blink.is_open { '0' } else { '-' };
                    if col >= 1 {
                        eye_overrides.push(((col - 1) as usize, '(', eye_color));
                    }
                    if col >= 0 {
                        eye_overrides.push((col as usize, ec, eye_color));
                    }
                    if col + 1 < line.len() as i32 {
                        eye_overrides.push(((col + 1) as usize, ')', eye_color));
                    }
                }
                render_sprite_row(
                    line,
                    base_x,
                    screen_y,
                    area,
                    buf,
                    &SpriteStyle {
                        eye_overrides: &eye_overrides,
                        body_color,
                        interior_fg,
                        glisten_colors: &glisten_colors,
                        color_patches: &unfish_state.slime_color_patches,
                    },
                );
            }
        }
        _ => {}
    }
}

fn render_multi_row_unfish(fish: &Fish, area: Rect, buf: &mut Buffer) {
    let unfish_state = match fish.unfish_state.as_ref() {
        Some(s) => s,
        None => return,
    };
    let base_x = area.x as i32 + fish.position.x as i32;
    let center_row = unfish_center_row(unfish_state.kind);
    let base_y = area.y as i32 + fish.position.y as i32 - center_row;
    render_multi_row_unfish_at(fish, base_x, base_y, area, buf);
}

fn render_worm_portal(fish: &Fish, area: Rect, buf: &mut Buffer) {
    let segs = fish.segments();
    let tank_h = area.height as i32;
    let tank_w = area.width as i32;
    let y_wrapped = (fish.position.y as i32).rem_euclid(tank_h);
    let sy = (area.y as i32 + y_wrapped) as u16;
    if sy < area.y || sy >= area.bottom() {
        return;
    }
    for (i, (ch, color)) in segs.iter().enumerate() {
        let tank_x = (fish.position.x as i32 + i as i32).rem_euclid(tank_w);
        let screen_x = area.x as i32 + tank_x;
        buf[(screen_x as u16, sy)]
            .set_char(*ch)
            .set_style(Style::new().fg(*color).remove_modifier(Modifier::all()));
    }
}

fn render_worm_name_portal(fish: &Fish, area: Rect, buf: &mut Buffer) {
    let tank_h = area.height as i32;
    let tank_w = area.width as i32;
    let name_y_wrapped = (fish.position.y as i32 - 1).rem_euclid(tank_h);
    let name_sy = (area.y as i32 + name_y_wrapped) as u16;
    if name_sy < area.y || name_sy >= area.bottom() {
        return;
    }
    let fish_center = fish.position.x as i32 + fish.display_width as i32 / 2;
    let name_start = fish_center - fish.name.len() as i32 / 2;
    for (i, ch) in fish.name.chars().enumerate() {
        let tank_x = (name_start + i as i32).rem_euclid(tank_w);
        let screen_x = area.x as i32 + tank_x;
        buf[(screen_x as u16, name_sy)]
            .set_char(ch)
            .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
    }
}

fn render_hell_background(bg: &HellBackground, area: Rect, buf: &mut Buffer) {
    if bg.is_random {
        let w = area.width as usize;
        for y in 0..area.height {
            for x in 0..area.width {
                let idx = y as usize * w + x as usize;
                if idx < bg.random_buffer.len() {
                    let ch = bg.random_buffer[idx];
                    if ch != ' ' {
                        buf[(area.x + x, area.y + y)]
                            .set_char(ch)
                            .set_fg(RANDOM_FACE_COLOR);
                    }
                }
            }
        }
        return;
    }

    let face_grid = &bg.face_grid;
    let face_h = face_grid.len();
    if face_h == 0 {
        return;
    }
    let face_w = face_grid[0].len();
    if face_w == 0 {
        return;
    }

    for y in 0..area.height {
        let world_y = bg.offset_y + y as f32;
        let tile_row_f = (world_y / face_h as f32).floor();
        let face_row = world_y.rem_euclid(face_h as f32) as usize;
        let face_row = face_row.min(face_h - 1);
        let beehive_shift = if (tile_row_f as i64).rem_euclid(2) == 1 {
            face_w / 2
        } else {
            0
        };

        let h_wave_shift: i32 =
            (H_WAVE_AMPLITUDE * (bg.h_phase - y as f32 * H_WAVE_ROW_SPREAD).sin()).round() as i32;

        for x in 0..area.width {
            let world_x = bg.offset_x + x as f32 + h_wave_shift as f32;
            let face_col = (world_x + beehive_shift as f32).rem_euclid(face_w as f32) as usize;
            let face_col = face_col.min(face_w - 1);
            let ch = face_grid[face_row][face_col];
            if ch != ' ' {
                buf[(area.x + x, area.y + y)]
                    .set_char(ch)
                    .set_fg(FACE_COLOR);
            }
        }
    }
}

fn render_void_background(bg: &VoidBackground, area: Rect, buf: &mut Buffer) {
    render_void_background_at(bg, bg.current_frame, area, buf);
}

fn render_void_background_at(bg: &VoidBackground, frame_idx: usize, area: Rect, buf: &mut Buffer) {
    if bg.frames.is_empty() {
        return;
    }
    let frame = &bg.frames[frame_idx % bg.frames.len()];
    let frame_h = frame.len() as i32;

    let start_x = area.width as i32 / 2 - VOID_EYE_CENTER_X;
    let target_eye_y = area.height as i32 / 2 + VOID_EYE_VERTICAL_OFFSET;
    let start_y = target_eye_y - frame_h / 2;

    let s = Style::new().fg(WHITE).remove_modifier(Modifier::all());

    for (row_i, line) in frame.iter().enumerate() {
        let screen_y = area.y as i32 + start_y + row_i as i32;
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        for (col_i, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let screen_x = area.x as i32 + start_x + col_i as i32;
            if screen_x < area.x as i32 || screen_x >= area.right() as i32 {
                continue;
            }
            buf[(screen_x as u16, screen_y as u16)]
                .set_char(ch)
                .set_style(s);
        }
    }
}

fn render_alien_pyramid_line(
    line: &str,
    mirrored: bool,
    placement: LinePlacement,
    eye_char: Option<char>,
    color: ratatui::style::Color,
    area: Rect,
    buf: &mut Buffer,
) {
    let LinePlacement {
        start_x,
        screen_y,
        canvas_w,
    } = placement;
    let style = Style::new().fg(color);
    if mirrored {
        let line_w = display_w(line);
        let effective_x = start_x + canvas_w - line_w;
        let pairs: Vec<(char, i32)> = line
            .chars()
            .map(|c| {
                (
                    mirror_char(c),
                    UnicodeWidthChar::width(c).unwrap_or(1) as i32,
                )
            })
            .rev()
            .collect();
        let mut col = 0i32;
        for (ch, w) in pairs {
            let draw_ch = if ch == '0' {
                eye_char.unwrap_or(ch)
            } else {
                ch
            };
            if draw_ch != ' ' {
                let sx = effective_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(draw_ch)
                        .set_style(style);
                }
            }
            col += w;
        }
    } else {
        let mut col = 0i32;
        for ch in line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
            let draw_ch = if ch == '0' {
                eye_char.unwrap_or(ch)
            } else {
                ch
            };
            if draw_ch != ' ' {
                let sx = start_x + col;
                if sx >= area.x as i32 && sx < area.right() as i32 {
                    buf[(sx as u16, screen_y as u16)]
                        .set_char(draw_ch)
                        .set_style(style);
                }
            }
            col += w;
        }
    }
}

fn render_alien_pyramid(
    pyramid: &AlienPyramid,
    color: ratatui::style::Color,
    area: Rect,
    buf: &mut Buffer,
) {
    let lines = pyramid_lines(pyramid.variant);
    let canvas_w = pyramid_canvas_w(pyramid.variant);
    let n = lines.len();
    for (row_idx, line) in lines.iter().enumerate() {
        let screen_y = area.y as i32 + area.height as i32 - n as i32 + row_idx as i32;
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        let start_x = area.x as i32 + pyramid.base_x;
        let eye_char = pyramid.eye_char_for_row(row_idx);
        render_alien_pyramid_line(
            line,
            pyramid.mirrored,
            LinePlacement {
                start_x,
                screen_y,
                canvas_w,
            },
            eye_char,
            color,
            area,
            buf,
        );
    }
}

fn render_alien_star(star: &AlienStar, area: Rect, buf: &mut Buffer) {
    let x = area.x + star.x;
    let y = area.y + star.y;
    if x < area.right() && y < area.bottom() {
        let color = if star.twinkle_lit { WHITE } else { GRAY };
        buf[(x, y)]
            .set_char(star.ch)
            .set_style(Style::new().fg(color).remove_modifier(Modifier::all()));
    }
}

fn render_haunted_gate(area: Rect, buf: &mut Buffer, bg: Option<&HauntedBackground>) {
    let floor_y = area.y as i32 + area.height as i32 - 1;
    let floor_style = Style::new()
        .fg(GATE_FLOOR_COLOR)
        .remove_modifier(Modifier::all());
    for x in area.x..area.right() {
        buf[(x, floor_y as u16)]
            .set_char(GATE_FLOOR_CHAR)
            .set_style(floor_style);
    }

    let mut col = 0i32;
    let mut bars_drawn = 0usize;
    let mut global_bar = 0usize;
    let mut global_block = 0usize;
    while col < area.width as i32 {
        if bars_drawn < BARS_PER_BLOCK {
            let holes = bg.map(|b| &b.bar_hole_mask[global_bar % b.bar_hole_mask.len()]);
            draw_gate_bar_tile(col, floor_y, area, buf, holes);
            col += GATE_BAR_W;
            bars_drawn += 1;
            global_bar += 1;
        } else {
            let col2 = bg.map(|b| &b.block_col2_mask[global_block % b.block_col2_mask.len()]);
            draw_gate_block_tile(col, floor_y, area, buf, col2);
            col += GATE_BLOCK_W;
            bars_drawn = 0;
            global_block += 1;
        }
    }
}

fn draw_gate_bar_tile(
    col: i32,
    floor_y: i32,
    area: Rect,
    buf: &mut Buffer,
    holes: Option<&[bool; GATE_BAR_PIPE_ROWS]>,
) {
    let n = GATE_BAR_TILE.len() as i32;
    let style = Style::new().fg(GATE_COLOR).remove_modifier(Modifier::all());
    let mut pipe_idx = 0usize;
    for (i, &line) in GATE_BAR_TILE.iter().enumerate() {
        let is_pipe = line == " | ";
        let local_pipe_idx = if is_pipe {
            let idx = pipe_idx;
            pipe_idx += 1;
            Some(idx)
        } else {
            None
        };
        let hidden = local_pipe_idx
            .and_then(|idx| holes.map(|h| h[idx]))
            .unwrap_or(false);
        if hidden {
            continue;
        }
        let screen_y = floor_y - 1 - (n - 1 - i as i32);
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        for (c, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let sx = area.x as i32 + col + c as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            buf[(sx as u16, screen_y as u16)]
                .set_char(ch)
                .set_style(style);
        }
    }
}

fn draw_gate_block_tile(
    col: i32,
    floor_y: i32,
    area: Rect,
    buf: &mut Buffer,
    col2_mask: Option<&[char; GATE_BLOCK_ROWS]>,
) {
    let n = GATE_BLOCK_TILE.len() as i32;
    let style = Style::new().fg(GATE_COLOR).remove_modifier(Modifier::all());
    for (i, &line) in GATE_BLOCK_TILE.iter().enumerate() {
        let screen_y = floor_y - 1 - (n - 1 - i as i32);
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        for (c, ch) in line.chars().enumerate() {
            let draw_ch = if c == 2 {
                col2_mask.map_or(ch, |m| m[i])
            } else {
                ch
            };
            if draw_ch == ' ' {
                continue;
            }
            let sx = area.x as i32 + col + c as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            buf[(sx as u16, screen_y as u16)]
                .set_char(draw_ch)
                .set_style(style);
        }
    }
}

fn render_opaque_grid(
    grid: &[Vec<(char, Color)>],
    base_x: i32,
    bottom_y: i32,
    area: Rect,
    buf: &mut Buffer,
) {
    let n = grid.len() as i32;
    for (i, row) in grid.iter().enumerate() {
        let screen_y = bottom_y - (n - 1 - i as i32);
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        for (col, &(ch, color)) in row.iter().enumerate() {
            if ch == GRAVE_TRANSPARENT {
                continue;
            }
            let sx = area.x as i32 + base_x + col as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            if ch == ' ' {
                buf[(sx as u16, screen_y as u16)]
                    .set_char(' ')
                    .set_style(Style::reset());
                continue;
            }
            buf[(sx as u16, screen_y as u16)]
                .set_char(ch)
                .set_style(Style::new().fg(color).remove_modifier(Modifier::all()));
        }
    }
}

fn render_ghost(ghost: &Ghost, area: Rect, buf: &mut Buffer) {
    let base_x = area.x as i32 + ghost.x as i32;
    let base_y = area.y as i32 + ghost.y as i32;
    let style = SpriteStyle {
        eye_overrides: &[],
        body_color: ghost.color,
        interior_fg: Some(ghost.color),
        glisten_colors: &[],
        color_patches: &[],
    };
    for (i, line) in ghost.sprite_rows().iter().enumerate() {
        let screen_y = base_y + i as i32;
        if screen_y < area.y as i32 || screen_y >= area.bottom() as i32 {
            continue;
        }
        render_sprite_row(line, base_x, screen_y, area, buf, &style);
    }
}

fn render_bat(bat: &Bat, area: Rect, buf: &mut Buffer) {
    let y = area.y as i32 + bat.y as i32;
    if y < area.y as i32 || y >= area.bottom() as i32 {
        return;
    }
    let style = Style::new().fg(BAT_COLOR).remove_modifier(Modifier::all());
    for (i, &ch) in BAT_SPRITE.iter().enumerate() {
        let sx = area.x as i32 + bat.x as i32 + i as i32;
        if sx < area.x as i32 || sx >= area.right() as i32 {
            continue;
        }
        buf[(sx as u16, y as u16)].set_char(ch).set_style(style);
    }
}

fn render_cow(cow: &Cow, area: Rect, buf: &mut Buffer) {
    let sprite = cow_sprite(cow);
    let base_x = area.x as i32 + cow.position.x as i32;
    let base_y = area.y as i32 + cow.position.y as i32 - cow.sprite_top_offset() as i32;
    for (row_idx, row) in sprite.iter().enumerate() {
        let sy = base_y + row_idx as i32;
        if sy < area.y as i32 || sy >= area.bottom() as i32 {
            continue;
        }
        for (col_idx, &(ch, color)) in row.iter().enumerate() {
            if ch == crate::entities::cow::COW_TRANSPARENT {
                continue;
            }
            let sx = base_x + col_idx as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            if ch == ' ' {
                buf[(sx as u16, sy as u16)]
                    .set_char(' ')
                    .set_style(Style::reset());
                continue;
            }
            buf[(sx as u16, sy as u16)]
                .set_char(ch)
                .set_style(Style::new().fg(color).remove_modifier(Modifier::all()));
        }
    }
}

fn render_cow_name(cow: &Cow, area: Rect, buf: &mut Buffer) {
    let base_x = cow.position.x as i32 + cow.display_width as i32 / 2;
    let name_start = base_x - cow.name.len() as i32 / 2;
    let name_y_i32 = area.y as i32 + cow.position.y as i32 - 1 - cow.sprite_top_offset() as i32;
    if name_y_i32 < area.y as i32 || name_y_i32 >= area.bottom() as i32 {
        return;
    }
    for (i, ch) in cow.name.chars().enumerate() {
        let x = name_start + i as i32;
        if x < 0 {
            continue;
        }
        let abs_x = area.x + x as u16;
        if abs_x >= area.right() {
            break;
        }
        buf[(abs_x, name_y_i32 as u16)]
            .set_char(ch)
            .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
    }
}

const SPEECH_TAIL_OFFSET_FROM_EYE: i32 = 3;
const SPEECH_BUBBLE_TO_TAIL_OFFSET: i32 = 4;

fn render_cow_speech(cow: &Cow, area: Rect, buf: &mut Buffer) {
    let Some(speech) = &cow.speech else { return };
    if cow.mutant.is_double {
        let head_render = cow.eye_count().clamp(2, 5);
        let head_w = head_render + 2;
        let torso = cow.torso_width();
        let left_eye = 1_i32;
        let right_eye = (head_w + torso + 1 + 1) as i32;
        draw_speech_bubble(cow, &speech.text, left_eye, area, buf);
        draw_speech_bubble(cow, &speech.text, right_eye, area, buf);
        return;
    }
    let left_eye = 1_i32;
    draw_speech_bubble(cow, &speech.text, left_eye, area, buf);
}

fn draw_speech_bubble(cow: &Cow, text: &str, leftmost_eye_col: i32, area: Rect, buf: &mut Buffer) {
    let bubble = build_speech_bubble(text);
    let bubble_h = bubble.len() as i32;
    let cow_x = area.x as i32 + cow.position.x as i32;
    let cow_y = area.y as i32 + cow.position.y as i32;
    let sprite_top = cow_y - cow.sprite_top_offset() as i32;
    let lower_tail_x = cow_x + leftmost_eye_col - SPEECH_TAIL_OFFSET_FROM_EYE;
    let upper_tail_x = lower_tail_x - 1;
    let lower_tail_y = cow_y + 1;
    let upper_tail_y = cow_y;
    let bubble_left_x = upper_tail_x - SPEECH_BUBBLE_TO_TAIL_OFFSET;
    let bubble_top_y = sprite_top - bubble_h;
    if bubble_top_y < area.y as i32 {
        return;
    }
    for (li, line) in bubble.iter().enumerate() {
        let sy = bubble_top_y + li as i32;
        for (ci, ch) in line.chars().enumerate() {
            let sx = bubble_left_x + ci as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            buf[(sx as u16, sy as u16)]
                .set_char(ch)
                .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
        }
    }
    if upper_tail_y >= area.y as i32
        && upper_tail_y < area.bottom() as i32
        && upper_tail_x >= area.x as i32
        && upper_tail_x < area.right() as i32
    {
        buf[(upper_tail_x as u16, upper_tail_y as u16)]
            .set_char('\\')
            .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
    }
    if lower_tail_y >= area.y as i32
        && lower_tail_y < area.bottom() as i32
        && lower_tail_x >= area.x as i32
        && lower_tail_x < area.right() as i32
    {
        buf[(lower_tail_x as u16, lower_tail_y as u16)]
            .set_char('\\')
            .set_style(Style::new().fg(WHITE).remove_modifier(Modifier::all()));
    }
}

fn render_ufo(ufo: &Ufo, area: Rect, buf: &mut Buffer) {
    let sprite = ufo_sprite(ufo);
    let base_x = area.x as i32 + ufo.x as i32;
    let base_y = area.y as i32 + ufo.y as i32;
    for (row_idx, row) in sprite.iter().enumerate() {
        let sy = base_y + row_idx as i32;
        if sy < area.y as i32 || sy >= area.bottom() as i32 {
            continue;
        }
        for (col_idx, &(ch, color)) in row.iter().enumerate() {
            if ch == '\0' {
                continue;
            }
            let sx = base_x + col_idx as i32;
            if sx < area.x as i32 || sx >= area.right() as i32 {
                continue;
            }
            if ch == ' ' {
                buf[(sx as u16, sy as u16)]
                    .set_char(' ')
                    .set_style(Style::reset());
                continue;
            }
            buf[(sx as u16, sy as u16)]
                .set_char(ch)
                .set_style(Style::new().fg(color).remove_modifier(Modifier::all()));
        }
    }
}

fn render_ritual_text(lines: &[Option<String>; 2], area: Rect, buf: &mut Buffer) {
    let target_eye_y = area.height as i32 / 2 + VOID_EYE_VERTICAL_OFFSET;
    let text_y_base = target_eye_y + VOID_TEXT_BELOW_EYE_OFFSET;
    let s = Style::new().fg(WHITE).remove_modifier(Modifier::all());

    for (i, line_opt) in lines.iter().enumerate() {
        if let Some(text) = line_opt {
            let y = area.y as i32 + text_y_base + i as i32;
            if y < area.y as i32 || y >= area.bottom() as i32 {
                continue;
            }
            let text_w = text.chars().count() as i32;
            let x_start = area.x as i32 + area.width as i32 / 2 - text_w / 2;
            for (col, ch) in text.chars().enumerate() {
                let x = x_start + col as i32;
                if x < area.x as i32 || x >= area.right() as i32 {
                    continue;
                }
                buf[(x as u16, y as u16)].set_char(ch).set_style(s);
            }
        }
    }
}
