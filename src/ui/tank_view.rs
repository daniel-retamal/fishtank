use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

use crate::{
    entities::bubble::Bubble,
    entities::coral::{
        CORAL_A_LINES, CORAL_A_ROWS, CORAL_COLOR, CoralAlgaeInstance, CoralStructure,
        FLOOR_ALGAE_A, FLOOR_ALGAE_COLOR, FloorAlgae, algae_art, algae_row_shift, mirror_char,
        query_anim_char, vert_wave_char,
    },
    entities::fish::Fish,
    entities::food::Food,
    entities::hell::{
        FACE_COLOR, H_WAVE_AMPLITUDE, H_WAVE_ROW_SPREAD, HellBackground, HellPlant,
        RANDOM_FACE_COLOR,
    },
    entities::plant::Plant,
    tank::{Tank, TankKind},
};

pub struct TankView<'a> {
    tank: &'a Tank,
    show_names: bool,
}

impl<'a> TankView<'a> {
    pub fn new(tank: &'a Tank, show_names: bool) -> Self {
        Self { tank, show_names }
    }
}

impl Widget for TankView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.tank.kind {
            TankKind::Base => {
                for plant in &self.tank.plants {
                    if plant.x >= area.width as i32 {
                        break;
                    }
                    render_plant(plant, area, buf);
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
                    render_hell_plant(plant, area, buf);
                }
            }
        }
        for bubble in &self.tank.bubbles {
            render_bubble(bubble, area, buf);
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
        if self.show_names {
            for fish in &self.tank.fish {
                render_fish_name(fish, area, buf);
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

fn render_plant(plant: &Plant, area: Rect, buf: &mut Buffer) {
    let base_x = plant.x;
    for h in 0..plant.height {
        let (x_offset, ch) = plant.segment_at(h);
        let x = base_x + x_offset;
        let y = area.height as i32 - 1 - h as i32;
        if x >= 0 && x < area.width as i32 && y >= 0 {
            buf[(area.x + x as u16, area.y + y as u16)]
                .set_char(ch)
                .set_fg(plant.color);
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

#[allow(clippy::too_many_arguments)]
fn render_algae_line(
    line: &str,
    li: usize,
    instance: &CoralAlgaeInstance,
    start_x: i32,
    screen_y: i32,
    canvas_w: i32,
    area: Rect,
    buf: &mut Buffer,
) {
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
                anchor_screen_x + shift,
                screen_y,
                canvas_w,
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
        buf[(x, y)].set_char(food.food_char).set_style(
            Style::new()
                .fg(Color::Yellow)
                .remove_modifier(Modifier::all()),
        );
    }
}

fn render_fish_name(fish: &Fish, area: Rect, buf: &mut Buffer) {
    let fish_y = area.y + fish.position.y as u16;
    if fish_y <= area.y || fish_y >= area.bottom() {
        return;
    }
    let name_y = fish_y - 1;

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
        buf[(abs_x, name_y)].set_char(ch).set_style(
            Style::new()
                .fg(Color::White)
                .remove_modifier(Modifier::all()),
        );
    }
}

fn render_fish(fish: &Fish, area: Rect, buf: &mut Buffer) {
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

fn render_hell_plant(plant: &HellPlant, area: Rect, buf: &mut Buffer) {
    let base_x = plant.x;
    for h in 0..plant.height {
        let (x_offset, ch) = plant.segment_at(h);
        let x = base_x + x_offset;
        let y = area.height as i32 - 1 - h as i32;
        if x >= 0 && x < area.width as i32 && y >= 0 {
            buf[(area.x + x as u16, area.y + y as u16)]
                .set_char(ch)
                .set_fg(plant.color);
        }
    }
}
