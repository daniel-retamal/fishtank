use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthChar;

use crate::{
    entities::bubble::Bubble, entities::fish::Fish, entities::food::Food, entities::plant::Plant,
    tank::Tank,
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
        for plant in &self.tank.plants {
            if plant.x >= area.width as i32 {
                break;
            }
            render_plant(plant, area, buf);
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
        if self.show_names {
            for fish in &self.tank.fish {
                render_fish_name(fish, area, buf);
            }
        }
    }
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
