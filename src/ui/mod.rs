pub mod catch_overlay;
pub mod command_bar;
pub mod fields;
pub mod fishing_overlay;
pub mod fishtanks_overlay;
pub mod index_overlay;
pub mod inventory_overlay;
pub mod scroll_list;
pub mod shop_overlay;
pub mod show_overlay;
pub mod table;
pub mod tank_view;
pub mod text_input;

use ratatui::{buffer::Buffer, style::Color};
use unicode_width::UnicodeWidthChar;

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
