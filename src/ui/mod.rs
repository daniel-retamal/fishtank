pub mod catch_overlay;
pub mod command_bar;
pub mod consume_picker;
pub mod fields;
pub mod fishing_overlay;
pub mod fishtanks_overlay;
pub mod hints;
pub mod index_overlay;
pub mod input_action;
pub mod inventory_overlay;
pub mod line_editor;
pub mod scroll_list;
pub mod shop_overlay;
pub mod show_overlay;
pub mod table;
pub mod tank_view;
pub mod text_input;

use ratatui::{buffer::Buffer, style::Color};
use unicode_width::UnicodeWidthChar;

use crate::fishes::fish::LineSprite;
use crate::sprite::TRANSPARENT;

pub fn render_fish_segs(
    buf: &mut Buffer,
    segs: &[(char, Color)],
    x: u16,
    y: u16,
    max_width: u16,
    bg: Color,
) {
    let mut col = 0u16;
    for (ch, color) in segs {
        let cw = UnicodeWidthChar::width(*ch).unwrap_or(1) as u16;
        if col + cw > max_width {
            break;
        }
        buf[(x + col, y)].set_char(*ch).set_fg(*color).set_bg(bg);
        col += cw;
    }
}

pub fn render_fish_sprite(
    buf: &mut Buffer,
    sprite: &LineSprite,
    x: u16,
    y: u16,
    max_width: u16,
    bg: Color,
) {
    for (row_idx, row) in sprite.rows.iter().enumerate() {
        let row_y = y as i32 + row_idx as i32 - sprite.body_row as i32;
        if row_y < 0 {
            continue;
        }
        let row_y = row_y as u16;
        let mut col = 0u16;
        for &(ch, color) in row {
            let cw = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
            if col + cw > max_width {
                break;
            }
            if ch != TRANSPARENT {
                buf[(x + col, row_y)].set_char(ch).set_fg(color).set_bg(bg);
            }
            col += cw;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::species::FishSpecies;
    use crate::tank::{Tank, TankKind};
    use ratatui::layout::Rect;

    #[test]
    fn footed_fish_draws_feet_one_row_below_the_body() {
        let mut rng = rand::rng();
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        tank.spawn_fish(FishSpecies::Merluza, "Steps".to_string(), &mut rng);
        tank.apply_named_mutation("Steps", "feet");
        assert!(tank.fish[0].feet().is_some());
        let area = Rect::new(0, 0, 40, 6);
        let mut buf = Buffer::empty(area);
        render_fish_sprite(
            &mut buf,
            &tank.fish[0].line_sprite(),
            2,
            2,
            30,
            Color::Reset,
        );
        let is_foot = |s: &str| s == "\"" || s == "^";
        let body_feet = (0..area.width)
            .filter(|&x| is_foot(buf[(x, 2)].symbol()))
            .count();
        let below_feet = (0..area.width)
            .filter(|&x| is_foot(buf[(x, 3)].symbol()))
            .count();
        assert_eq!(body_feet, 0, "the body row never carries feet glyphs");
        assert!(
            below_feet > 0,
            "feet land on the row directly below the body"
        );
    }
}
