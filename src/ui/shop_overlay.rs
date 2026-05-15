use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use std::collections::HashMap;
use unicode_width::UnicodeWidthChar;

use crate::{
    entities::{fish::Fish, species::FishSpecies},
    ui::render_fish_segs,
};

const BG: Color = Color::Reset;
const PENGUIN_W: u16 = 7;
const RIGHT_INNER_W: u16 = 34;
const INNER_W: u16 = PENGUIN_W + 1 + RIGHT_INNER_W;
const OVERLAY_W: u16 = INNER_W + 2;
const PENGUIN_H: u16 = 4;
const PENGUIN_BOX_ROW: u16 = PENGUIN_H + 1;
const MAX_LIST_VISIBLE: usize = 7;
const BUY_CAT_ITEM_COUNT: usize = 5;

const PENGUIN_LINES: &[&str] = &["  __   ", " ( o>  ", " ///\\  ", " \\V_/_ "];

pub const FOOD_BUY_PRICE: u32 = 1;
pub const JUNK_SELL_PRICE: u32 = 1;
pub const COFFEE_BUY_PRICE: u32 = 10;
pub const COFFEE_SELL_PRICE: u32 = 8;
pub const BAIT_BUY_PRICE: u32 = 15;
pub const BAIT_SELL_PRICE: u32 = 12;
pub const TANK_BUY_PRICE: u32 = 3000;
pub const TANK_SELL_PRICE: u32 = 2500;

pub struct BuyEntry {
    pub species: FishSpecies,
    pub price: u32,
    pub name: &'static str,
}

pub const FISH_CATALOG: &[BuyEntry] = &[
    BuyEntry {
        species: FishSpecies::Merluza,
        price: 126,
        name: "Merluza",
    },
    BuyEntry {
        species: FishSpecies::Betta,
        price: 126,
        name: "Betta",
    },
    BuyEntry {
        species: FishSpecies::Salmon,
        price: 126,
        name: "Salmon",
    },
    BuyEntry {
        species: FishSpecies::Chromis,
        price: 126,
        name: "Chromis",
    },
    BuyEntry {
        species: FishSpecies::Tang,
        price: 126,
        name: "Tang",
    },
    BuyEntry {
        species: FishSpecies::Carpin,
        price: 126,
        name: "Carpin",
    },
    BuyEntry {
        species: FishSpecies::Anchoveta,
        price: 126,
        name: "Anchoveta",
    },
    BuyEntry {
        species: FishSpecies::Goldfish,
        price: 126,
        name: "Goldfish",
    },
    BuyEntry {
        species: FishSpecies::Snapper,
        price: 126,
        name: "Snapper",
    },
    BuyEntry {
        species: FishSpecies::Nishiki,
        price: 126,
        name: "Nishiki",
    },
    BuyEntry {
        species: FishSpecies::Aka,
        price: 126,
        name: "Aka",
    },
    BuyEntry {
        species: FishSpecies::Kuro,
        price: 126,
        name: "Kuro",
    },
    BuyEntry {
        species: FishSpecies::Koi,
        price: 341,
        name: "Koi",
    },
    BuyEntry {
        species: FishSpecies::Deadfish,
        price: 341,
        name: "Deadfish",
    },
    BuyEntry {
        species: FishSpecies::Jellyfish,
        price: 341,
        name: "Jellyfish",
    },
    BuyEntry {
        species: FishSpecies::Turbofish,
        price: 341,
        name: "Turbofish",
    },
    BuyEntry {
        species: FishSpecies::Goldenfish,
        price: 7700,
        name: "Goldenfish",
    },
    BuyEntry {
        species: FishSpecies::Mutantfish,
        price: 7700,
        name: "Mutantfish",
    },
];

pub struct BuyCategoryPopup {
    pub option_idx: usize,
    pub qty: u32,
    pub max_qty: u32,
}

pub struct BuyTankPopup {
    pub name_input: String,
    pub cursor_pos: usize,
}

pub fn buy_cat_available(idx: usize, money: u32) -> bool {
    match idx {
        0 => FISH_CATALOG.iter().any(|e| e.price <= money),
        1 => money >= COFFEE_BUY_PRICE,
        2 => money >= BAIT_BUY_PRICE,
        3 => money >= FOOD_BUY_PRICE,
        4 => money >= TANK_BUY_PRICE,
        _ => false,
    }
}

pub fn buy_cat_first_available(money: u32) -> usize {
    (0..5).find(|&i| buy_cat_available(i, money)).unwrap_or(0)
}

pub fn buy_cat_next(current: usize, down: bool, money: u32) -> usize {
    let step: i32 = if down { 1 } else { -1 };
    let mut idx = current as i32 + step;
    while (0..5).contains(&idx) {
        if buy_cat_available(idx as usize, money) {
            return idx as usize;
        }
        idx += step;
    }
    current
}

pub struct FishNamePopup {
    pub catalog_idx: usize,
    pub fish: Fish,
    pub name_input: String,
    pub cursor_pos: usize,
}

pub enum SellEntry {
    Fish { name: String, species: FishSpecies, sell_value: u32 },
    Junk { qty: u32 },
    Coffee { qty: u32 },
    Bait { qty: u32 },
    Tank { name: String },
}

impl SellEntry {
    pub fn label(&self) -> String {
        match self {
            SellEntry::Fish { name, species, .. } => format!("{} ({})", name, species.display_name()),
            SellEntry::Junk { qty } => format!("Junk ({})", qty),
            SellEntry::Coffee { qty } => format!("Coffee ({})", qty),
            SellEntry::Bait { qty } => format!("Bait ({})", qty),
            SellEntry::Tank { name } => format!("{} (Fishtank)", name),
        }
    }

    pub fn price_label(&self) -> String {
        match self {
            SellEntry::Fish { sell_value, .. } => format!("${}", sell_value),
            SellEntry::Junk { .. } => format!("${}", JUNK_SELL_PRICE),
            SellEntry::Coffee { .. } => format!("${}", COFFEE_SELL_PRICE),
            SellEntry::Bait { .. } => format!("${}", BAIT_SELL_PRICE),
            SellEntry::Tank { .. } => format!("${}", TANK_SELL_PRICE),
        }
    }

    pub fn unit_price(&self) -> u32 {
        match self {
            SellEntry::Fish { sell_value, .. } => *sell_value,
            SellEntry::Junk { .. } => JUNK_SELL_PRICE,
            SellEntry::Coffee { .. } => COFFEE_SELL_PRICE,
            SellEntry::Bait { .. } => BAIT_SELL_PRICE,
            SellEntry::Tank { .. } => TANK_SELL_PRICE,
        }
    }

    pub fn max_qty(&self) -> u32 {
        match self {
            SellEntry::Fish { .. } | SellEntry::Tank { .. } => 1,
            SellEntry::Junk { qty } => *qty,
            SellEntry::Coffee { qty } => *qty,
            SellEntry::Bait { qty } => *qty,
        }
    }
}

pub struct SellConfirm {
    pub item_idx: usize,
    pub sell_qty: u32,
}

impl SellConfirm {
    pub fn qty_up(&mut self, max: u32) {
        if self.sell_qty < max {
            self.sell_qty += 1;
        }
    }

    pub fn qty_down(&mut self) {
        if self.sell_qty > 1 {
            self.sell_qty -= 1;
        }
    }
}

pub struct SellMenuState {
    pub items: Vec<SellEntry>,
    pub selected: usize,
    pub scroll: usize,
    pub confirm: Option<SellConfirm>,
}

impl SellMenuState {
    pub fn new(
        tank_fish: &[(String, FishSpecies, u32)],
        inventory: &HashMap<String, u32>,
        sellable_tanks: &[String],
    ) -> Option<Self> {
        let mut items: Vec<SellEntry> = Vec::new();
        let junk = inventory.get("Junk").copied().unwrap_or(0);
        if junk > 0 {
            items.push(SellEntry::Junk { qty: junk });
        }
        let coffee = inventory.get("Coffee").copied().unwrap_or(0);
        if coffee > 0 {
            items.push(SellEntry::Coffee { qty: coffee });
        }
        let bait = inventory.get("Bait").copied().unwrap_or(0);
        if bait > 0 {
            items.push(SellEntry::Bait { qty: bait });
        }
        for tank_name in sellable_tanks {
            items.push(SellEntry::Tank {
                name: tank_name.clone(),
            });
        }
        let mut fish_entries: Vec<SellEntry> = tank_fish
            .iter()
            .map(|(n, s, sv)| SellEntry::Fish {
                name: n.clone(),
                species: *s,
                sell_value: *sv,
            })
            .collect();
        fish_entries.sort_by(|a, b| {
            let pa = match a { SellEntry::Fish { sell_value, .. } => *sell_value, _ => 0 };
            let pb = match b { SellEntry::Fish { sell_value, .. } => *sell_value, _ => 0 };
            pa.cmp(&pb).then_with(|| {
                let na = match a {
                    SellEntry::Fish { name, .. } => name.as_str(),
                    _ => "",
                };
                let nb = match b {
                    SellEntry::Fish { name, .. } => name.as_str(),
                    _ => "",
                };
                na.cmp(nb)
            })
        });
        items.extend(fish_entries);
        if items.is_empty() {
            return None;
        }
        Some(Self {
            items,
            selected: 0,
            scroll: 0,
            confirm: None,
        })
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll {
                self.scroll = self.selected;
            }
        }
    }

    pub fn scroll_down(&mut self, visible: usize) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
            if self.selected >= self.scroll + visible {
                self.scroll = self.selected + 1 - visible;
            }
        }
    }
}

pub struct FishListState {
    pub selected: usize,
    pub scroll: usize,
    pub popup: Option<FishNamePopup>,
}

impl FishListState {
    pub fn new(money: u32) -> Self {
        let selected = FISH_CATALOG
            .iter()
            .position(|e| e.price <= money)
            .unwrap_or(0);
        Self {
            selected,
            scroll: 0,
            popup: None,
        }
    }

    pub fn scroll_up(&mut self, money: u32) {
        let mut idx = self.selected;
        while idx > 0 {
            idx -= 1;
            if FISH_CATALOG[idx].price <= money {
                self.selected = idx;
                if self.selected < self.scroll {
                    self.scroll = self.selected;
                }
                return;
            }
        }
    }

    pub fn scroll_down(&mut self, money: u32, visible: usize) {
        let n = FISH_CATALOG.len();
        let mut idx = self.selected;
        loop {
            idx += 1;
            if idx >= n {
                return;
            }
            if FISH_CATALOG[idx].price <= money {
                self.selected = idx;
                if self.selected >= self.scroll + visible {
                    self.scroll = self.selected + 1 - visible;
                }
                return;
            }
        }
    }
}

pub enum ShopPage {
    Main {
        selected: usize,
    },
    BuyCategory {
        selected: usize,
        buy_popup: Option<BuyCategoryPopup>,
        buy_tank_popup: Option<BuyTankPopup>,
    },
    BuyFishList(FishListState),
    Sell(SellMenuState),
}

pub struct ShopState {
    pub page: ShopPage,
    pub cursor_visible: bool,
    blink_timer: f32,
}

impl ShopState {
    pub fn new() -> Self {
        Self {
            page: ShopPage::Main { selected: 0 },
            cursor_visible: true,
            blink_timer: 0.0,
        }
    }

    pub fn tick(&mut self, fps: f32) {
        self.blink_timer += 1.0;
        let half = (fps * 0.5).max(1.0);
        if self.blink_timer >= half {
            self.blink_timer = 0.0;
            self.cursor_visible = !self.cursor_visible;
        }
        if let ShopPage::BuyFishList(ref mut fl) = self.page
            && let Some(ref mut p) = fl.popup
        {
            p.fish.tick_animation(1.0 / fps);
        }
    }

    pub fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.blink_timer = 0.0;
    }
}

pub struct ShopOverlay<'a> {
    pub state: &'a ShopState,
    pub money: u32,
}

impl<'a> ShopOverlay<'a> {
    pub fn new(state: &'a ShopState, money: u32) -> Self {
        Self { state, money }
    }
}

fn overlay_h(state: &ShopState, area_h: u16) -> u16 {
    const MIN_OH: u16 = PENGUIN_BOX_ROW + 2;
    const MAX_OH: u16 = 13;
    match &state.page {
        ShopPage::Main { .. } => MIN_OH.min(area_h),
        ShopPage::BuyCategory { .. } => (MIN_OH + 2).min(area_h),
        ShopPage::BuyFishList(_) => {
            let visible = FISH_CATALOG.len().min(MAX_LIST_VISIBLE);
            ((visible as u16) + 6).clamp(MIN_OH, MAX_OH).min(area_h)
        }
        ShopPage::Sell(sm) => {
            let visible = sm.items.len().min(MAX_LIST_VISIBLE);
            ((visible as u16) + 6).clamp(MIN_OH, MAX_OH).min(area_h)
        }
    }
}

pub fn list_visible_rows(state: &ShopState, area_h: u16) -> usize {
    let oh = overlay_h(state, area_h);
    oh.saturating_sub(6) as usize
}

impl Widget for ShopOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let oh = overlay_h(state, area.height);

        if area.width < OVERLAY_W || area.height < oh {
            return;
        }

        let ox = area.x + (area.width - OVERLAY_W) / 2;
        let oy = area.y + (area.height - oh) / 2;

        let has_popup = match &state.page {
            ShopPage::BuyCategory {
                buy_popup,
                buy_tank_popup,
                ..
            } => buy_popup.is_some() || buy_tank_popup.is_some(),
            ShopPage::BuyFishList(fl) => fl.popup.is_some(),
            ShopPage::Sell(sm) => sm.confirm.is_some(),
            _ => false,
        };

        for dy in 0..oh {
            for dx in 0..OVERLAY_W {
                buf[(ox + dx, oy + dy)].reset();
                buf[(ox + dx, oy + dy)].set_bg(BG);
            }
        }

        let fg = if has_popup {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(fg).bg(BG);
        let s_title = Style::default().fg(fg).add_modifier(Modifier::BOLD).bg(BG);
        let sep_x = ox + 1 + PENGUIN_W;
        let right_x = ox + OVERLAY_W - 1;

        buf[(ox, oy)].set_char('┌').set_style(s);
        for dx in 1..=PENGUIN_W {
            buf[(ox + dx, oy)].set_char('─').set_style(s);
        }
        buf[(sep_x, oy)].set_char('┬').set_style(s);
        for x in sep_x + 1..right_x {
            buf[(x, oy)].set_char('─').set_style(s);
        }
        buf[(right_x, oy)].set_char('┐').set_style(s);
        let page_title = match &state.page {
            ShopPage::Main { .. } => " Shop ",
            ShopPage::BuyCategory { .. } | ShopPage::BuyFishList(_) => " Buy ",
            ShopPage::Sell(_) => " Sell ",
        };
        buf.set_string(ox + 2, oy, page_title, s_title);

        for dy in 1..PENGUIN_BOX_ROW {
            buf[(ox, oy + dy)].set_char('│').set_style(s);
        }

        for dy in 1..oh - 1 {
            buf[(right_x, oy + dy)].set_char('│').set_style(s);
        }

        for dy in 1..oh - 1 {
            let ch = if dy == PENGUIN_BOX_ROW { '┤' } else { '│' };
            buf[(sep_x, oy + dy)].set_char(ch).set_style(s);
        }

        buf[(ox, oy + PENGUIN_BOX_ROW)].set_char('└').set_style(s);
        for dx in 1..=PENGUIN_W {
            buf[(ox + dx, oy + PENGUIN_BOX_ROW)]
                .set_char('─')
                .set_style(s);
        }

        buf[(sep_x, oy + oh - 1)].set_char('└').set_style(s);
        for dx in 1..=RIGHT_INNER_W {
            buf[(sep_x + dx, oy + oh - 1)].set_char('─').set_style(s);
        }
        buf[(right_x, oy + oh - 1)].set_char('┘').set_style(s);

        draw_penguin(buf, ox + 1, oy + 1, has_popup);

        let rx = sep_x + 1;
        let content_y = oy + 1;
        let footer_y = oy + oh - 2;
        let footer_w = RIGHT_INNER_W - 1;

        match &state.page {
            ShopPage::Main { selected } => {
                let content_h = oh.saturating_sub(3);
                draw_main_right(
                    buf,
                    *selected,
                    state.cursor_visible,
                    rx,
                    content_y,
                    RIGHT_INNER_W,
                    content_h,
                    has_popup,
                );
                draw_footer_text(
                    buf,
                    rx,
                    footer_y,
                    footer_w,
                    " \u{2191}\u{2193} navigate",
                    "ESC/q close",
                );
            }
            ShopPage::BuyCategory {
                selected,
                buy_popup,
                buy_tank_popup,
            } => {
                let content_h = oh.saturating_sub(3);
                let available = [
                    buy_cat_available(0, self.money),
                    buy_cat_available(1, self.money),
                    buy_cat_available(2, self.money),
                    buy_cat_available(3, self.money),
                    buy_cat_available(4, self.money),
                ];
                let draw_h = content_h.saturating_sub(1);
                draw_buy_category_right(
                    buf,
                    *selected,
                    state.cursor_visible,
                    &available,
                    rx,
                    content_y + 1,
                    RIGHT_INNER_W,
                    draw_h,
                    has_popup,
                );
                let visible = draw_h as usize;
                let footer_left = if BUY_CAT_ITEM_COUNT > visible {
                    format!(
                        " \u{2191}\u{2193} scroll ({}/{})",
                        selected + 1,
                        BUY_CAT_ITEM_COUNT
                    )
                } else {
                    " \u{2191}\u{2193} navigate".to_string()
                };
                draw_footer_text(buf, rx, footer_y, footer_w, &footer_left, "ESC/q back");
                if let Some(popup) = buy_tank_popup {
                    draw_buy_tank_popup(buf, popup, state.cursor_visible, area);
                } else if let Some(popup) = buy_popup {
                    draw_category_buy_popup(buf, popup, area);
                }
            }
            ShopPage::BuyFishList(fl) => {
                let visible = oh.saturating_sub(6) as usize;
                let right_border_x = ox + OVERLAY_W - 1;
                draw_fish_list(
                    buf,
                    fl,
                    self.money,
                    state.cursor_visible,
                    sep_x,
                    content_y,
                    right_border_x,
                    RIGHT_INNER_W,
                    visible,
                    has_popup,
                );
                let affordable_count = FISH_CATALOG
                    .iter()
                    .filter(|e| e.price <= self.money)
                    .count();
                let affordable_pos = FISH_CATALOG[..=fl.selected]
                    .iter()
                    .filter(|e| e.price <= self.money)
                    .count();
                let footer_left = format!(
                    " \u{2191}\u{2193} scroll ({}/{})",
                    affordable_pos, affordable_count
                );
                draw_footer_text(buf, rx, footer_y, footer_w, &footer_left, "ESC/q back");
                if let Some(ref popup) = fl.popup {
                    draw_fish_name_popup(buf, popup, state.cursor_visible, area);
                }
            }
            ShopPage::Sell(sm) => {
                let visible = oh.saturating_sub(6) as usize;
                let right_border_x = ox + OVERLAY_W - 1;
                draw_sell_list(
                    buf,
                    sm,
                    state.cursor_visible,
                    sep_x,
                    content_y,
                    right_border_x,
                    RIGHT_INNER_W,
                    visible,
                    has_popup,
                );
                let n = sm.items.len();
                let footer_left = format!(" \u{2191}\u{2193} scroll ({}/{})", sm.selected + 1, n);
                draw_footer_text(buf, rx, footer_y, footer_w, &footer_left, "ESC/q back");
                if let Some(ref confirm) = sm.confirm {
                    draw_sell_confirm_popup(buf, confirm, &sm.items[confirm.item_idx], area);
                }
            }
        }
    }
}

fn draw_border(buf: &mut Buffer, ox: u16, oy: u16, w: u16, h: u16, title: &str, dim: bool) {
    let fg = if dim { Color::DarkGray } else { Color::White };
    let s = Style::default().fg(fg).bg(BG);
    let ts = Style::default().fg(fg).add_modifier(Modifier::BOLD).bg(BG);
    let r = ox + w - 1;
    let b = oy + h - 1;
    buf[(ox, oy)].set_char('┌').set_style(s);
    buf[(r, oy)].set_char('┐').set_style(s);
    buf[(ox, b)].set_char('└').set_style(s);
    buf[(r, b)].set_char('┘').set_style(s);
    for dx in 1..w - 1 {
        buf[(ox + dx, oy)].set_char('─').set_style(s);
        buf[(ox + dx, b)].set_char('─').set_style(s);
    }
    for dy in 1..h - 1 {
        buf[(ox, oy + dy)].set_char('│').set_style(s);
        buf[(r, oy + dy)].set_char('│').set_style(s);
    }
    if (title.len() as u16 + 4) < w {
        buf.set_string(ox + 2, oy, title, ts);
    }
}

fn draw_penguin(buf: &mut Buffer, x: u16, y: u16, dim: bool) {
    let fg = if dim { Color::DarkGray } else { Color::White };
    let s = Style::default().fg(fg).bg(BG);
    for (i, line) in PENGUIN_LINES.iter().enumerate() {
        if i < PENGUIN_H as usize {
            buf.set_string(x, y + i as u16, *line, s);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_main_right(
    buf: &mut Buffer,
    selected: usize,
    cursor_vis: bool,
    x: u16,
    y: u16,
    w: u16,
    content_h: u16,
    dim: bool,
) {
    const ITEMS: &[&str] = &["Buy", "Sell"];
    let start_y = y + (content_h.saturating_sub(ITEMS.len() as u16)) / 2;
    let eff_cursor_vis = dim || cursor_vis;
    for (i, &label) in ITEMS.iter().enumerate() {
        let row_y = start_y + i as u16;
        let is_sel = i == selected;
        let cursor = if is_sel && eff_cursor_vis { ">" } else { " " };
        let line = format!("{} {}", cursor, label);
        let fg = if dim || !is_sel {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(fg).bg(BG);
        buf.set_string(x, row_y, trunc(&line, w as usize), s);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_buy_category_right(
    buf: &mut Buffer,
    selected: usize,
    cursor_vis: bool,
    available: &[bool],
    x: u16,
    y: u16,
    w: u16,
    content_h: u16,
    dim: bool,
) {
    const ITEMS: &[&str] = &["Fishes", "Coffee", "Bait", "Food", "Fishtank"];
    let visible = content_h as usize;
    let scroll = if selected >= visible {
        selected + 1 - visible
    } else {
        0
    };
    let eff_cursor_vis = dim || cursor_vis;
    for row in 0..visible {
        let i = scroll + row;
        if i >= ITEMS.len() {
            break;
        }
        let is_sel = i == selected;
        let is_avail = available.get(i).copied().unwrap_or(false);
        let cursor = if is_sel && eff_cursor_vis { ">" } else { " " };
        let line = format!("{} {}", cursor, ITEMS[i]);
        let fg = if dim || !is_avail || !is_sel {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(fg).bg(BG);
        buf.set_string(x, y + row as u16, trunc(&line, w as usize), s);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_fish_list(
    buf: &mut Buffer,
    state: &FishListState,
    money: u32,
    cursor_vis: bool,
    sep_x: u16,
    content_y: u16,
    right_border_x: u16,
    right_w: u16,
    visible: usize,
    dim: bool,
) {
    let rx = sep_x + 1;
    let hdr_fg = if dim { Color::DarkGray } else { Color::White };
    let s_bold = Style::default()
        .fg(hdr_fg)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let s_sep = Style::default().fg(hdr_fg).bg(BG);

    buf.set_string(rx, content_y, pad("Name", (right_w / 2) as usize), s_bold);
    let price_hdr = "Price/unit";
    buf.set_string(
        right_border_x.saturating_sub(1 + price_hdr.len() as u16),
        content_y,
        price_hdr,
        s_bold,
    );

    let sep_y = content_y + 1;
    buf[(sep_x, sep_y)].set_char('├').set_style(s_sep);
    for dx in 1..right_w + 1 {
        buf[(sep_x + dx, sep_y)].set_char('─').set_style(s_sep);
    }
    buf[(right_border_x, sep_y)].set_char('┤').set_style(s_sep);

    let data_y = sep_y + 1;
    let n = FISH_CATALOG.len();
    let eff_cursor_vis = dim || cursor_vis;

    for row in 0..visible {
        let idx = state.scroll + row;
        if idx >= n {
            break;
        }
        let entry = &FISH_CATALOG[idx];
        let row_y = data_y + row as u16;
        let affordable = entry.price <= money;
        let is_sel = idx == state.selected;

        let price_str = format!("${}", entry.price);
        let price_x = right_border_x.saturating_sub(1 + price_str.len() as u16);
        let name_max = price_x.saturating_sub(rx + 2) as usize;

        let item_fg = if dim || !affordable {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(item_fg).bg(BG);

        let prefix = if is_sel && eff_cursor_vis { "> " } else { "  " };
        let label = format!("{}{}", prefix, trunc(entry.name, name_max));
        buf.set_string(rx, row_y, label, s);
        buf.set_string(price_x, row_y, &price_str, s);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_sell_list(
    buf: &mut Buffer,
    state: &SellMenuState,
    cursor_vis: bool,
    sep_x: u16,
    content_y: u16,
    right_border_x: u16,
    right_w: u16,
    visible: usize,
    dim: bool,
) {
    let rx = sep_x + 1;
    let hdr_fg = if dim { Color::DarkGray } else { Color::White };
    let s_bold = Style::default()
        .fg(hdr_fg)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let s_sep = Style::default().fg(hdr_fg).bg(BG);
    let s_white = Style::default()
        .fg(if dim { Color::DarkGray } else { Color::White })
        .bg(BG);

    buf.set_string(rx, content_y, pad("Item", (right_w / 2) as usize), s_bold);
    let price_hdr = "Price/unit";
    buf.set_string(
        right_border_x.saturating_sub(1 + price_hdr.len() as u16),
        content_y,
        price_hdr,
        s_bold,
    );

    let sep_y = content_y + 1;
    buf[(sep_x, sep_y)].set_char('├').set_style(s_sep);
    for dx in 1..right_w + 1 {
        buf[(sep_x + dx, sep_y)].set_char('─').set_style(s_sep);
    }
    buf[(right_border_x, sep_y)].set_char('┤').set_style(s_sep);

    let data_y = sep_y + 1;
    let n = state.items.len();
    let eff_cursor_vis = dim || cursor_vis;

    for row in 0..visible {
        let idx = state.scroll + row;
        if idx >= n {
            break;
        }
        let entry = &state.items[idx];
        let row_y = data_y + row as u16;
        let is_sel = idx == state.selected;

        let price_str = entry.price_label();
        let price_x = right_border_x.saturating_sub(1 + price_str.len() as u16);
        let name_max = price_x.saturating_sub(rx + 2) as usize;

        let prefix = if is_sel && eff_cursor_vis { "> " } else { "  " };
        let label = format!("{}{}", prefix, trunc(&entry.label(), name_max));
        buf.set_string(rx, row_y, label, s_white);
        buf.set_string(price_x, row_y, &price_str, s_white);
    }
}

fn draw_footer_text(buf: &mut Buffer, x: u16, y: u16, inner_w: u16, left: &str, right: &str) {
    let s = Style::default().fg(Color::DarkGray).bg(BG);
    buf.set_string(x, y, trunc(left, inner_w as usize), s);
    let rw = vw(right) as u16;
    if rw + vw(left) as u16 + 3 <= inner_w {
        buf.set_string(x + inner_w - rw, y, right, s);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_qty_row(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    max_w: usize,
    qty_str: &str,
    total_str: &str,
    l_s: Style,
    r_s: Style,
    mid_s: Style,
) {
    let mut col = x;
    let qty_len = qty_str.len();
    for (i, c) in qty_str.chars().enumerate() {
        let s = if i == 0 {
            l_s
        } else if i == qty_len - 1 {
            r_s
        } else {
            mid_s
        };
        buf[(col, y)].set_char(c).set_style(s);
        col += 1;
    }
    let remaining = (x + max_w as u16).saturating_sub(col) as usize;
    buf.set_string(col, y, trunc(total_str, remaining), mid_s);
}

fn draw_qty_popup(
    buf: &mut Buffer,
    title: &str,
    qty: u32,
    max_qty: u32,
    unit_price: u32,
    confirm_hint: &str,
    area: Rect,
) {
    const POP_W: u16 = 28;
    const POP_H: u16 = 5;
    if area.width < POP_W || area.height < POP_H {
        return;
    }
    let ox = area.x + (area.width - POP_W) / 2;
    let oy = area.y + (area.height - POP_H) / 2;

    for dy in 0..POP_H {
        for dx in 0..POP_W {
            buf[(ox + dx, oy + dy)].reset();
            buf[(ox + dx, oy + dy)].set_bg(BG);
        }
    }
    draw_border(buf, ox, oy, POP_W, POP_H, title, false);

    let s_white = Style::default().fg(Color::White).bg(BG);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BG);
    let inner_x = ox + 2;
    let inner_w = (POP_W - 4) as usize;

    let total = qty * unit_price;
    let l_s = if qty <= 1 { s_dim } else { s_white };
    let r_s = if qty >= max_qty { s_dim } else { s_white };
    let qty_str = format!("< {} >", qty);
    let total_str = format!("  Total: ${}", total);
    draw_qty_row(
        buf,
        inner_x,
        oy + 1,
        inner_w,
        &qty_str,
        &total_str,
        l_s,
        r_s,
        s_white,
    );

    let left_hint = "ESC/q cancel";
    buf.set_string(inner_x, oy + POP_H - 2, trunc(left_hint, inner_w), s_dim);
    let rw = confirm_hint.len() as u16;
    if (left_hint.len() as u16 + rw + 2) <= inner_w as u16 {
        buf.set_string(
            inner_x + inner_w as u16 - rw,
            oy + POP_H - 2,
            confirm_hint,
            s_dim,
        );
    }
}

fn draw_category_buy_popup(buf: &mut Buffer, popup: &BuyCategoryPopup, area: Rect) {
    let (title, unit_price) = match popup.option_idx {
        1 => (" Buy Coffee ", COFFEE_BUY_PRICE),
        2 => (" Buy Bait ", BAIT_BUY_PRICE),
        _ => (" Buy Food ", FOOD_BUY_PRICE),
    };
    draw_qty_popup(
        buf,
        title,
        popup.qty,
        popup.max_qty,
        unit_price,
        "ENTER buy",
        area,
    );
}

fn draw_fish_name_popup(buf: &mut Buffer, popup: &FishNamePopup, cursor_vis: bool, area: Rect) {
    const RIGHT_W: u16 = 30;
    const POP_H: u16 = 7;
    const CONTENT_H: u16 = 5;

    let fish = &popup.fish;
    let fish_dw = fish.display_width as u16;
    let left_w = fish_dw + 3;
    let inner_w = left_w + 1 + RIGHT_W;
    let pop_w = inner_w + 2;

    if area.width < pop_w || area.height < POP_H {
        return;
    }

    let ox = area.x + (area.width - pop_w) / 2;
    let oy = area.y + (area.height - POP_H) / 2;

    for dy in 0..POP_H {
        for dx in 0..pop_w {
            buf[(ox + dx, oy + dy)].reset();
            buf[(ox + dx, oy + dy)].set_bg(BG);
        }
    }

    let species = FISH_CATALOG[popup.catalog_idx].name;
    let title = format!(" Buy {} ", species);
    draw_border(buf, ox, oy, pop_w, POP_H, &title, false);

    let sep_x = ox + 1 + left_w;
    buf[(sep_x, oy + POP_H - 1)]
        .set_char('┴')
        .set_fg(Color::White)
        .set_bg(BG);
    for row in 1..=CONTENT_H {
        buf[(sep_x, oy + row)]
            .set_char('│')
            .set_fg(Color::White)
            .set_bg(BG);
    }

    let fish_x = ox + 2;
    let fish_y = oy + 1 + CONTENT_H / 2;
    let segs = fish.segments();
    render_fish_segs(buf, &segs, fish_x, fish_y, fish_dw, BG);

    let rx = sep_x + 1;
    let s_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let s_white = Style::default().fg(Color::White).bg(BG);

    let price = FISH_CATALOG[popup.catalog_idx].price;
    buf.set_string(
        rx,
        oy + 1,
        trunc(
            &format!("{} for sale! Only ${}", species, price),
            RIGHT_W as usize,
        ),
        s_bold,
    );
    buf.set_string(rx, oy + 3, "Name it", s_white);
    draw_text_cursor(
        buf,
        &popup.name_input,
        popup.cursor_pos,
        cursor_vis,
        rx,
        oy + 4,
        RIGHT_W,
    );

    let s_dim = Style::default().fg(Color::DarkGray).bg(BG);
    let left_hint = "ESC/q cancel";
    let right_hint = "ENTER buy";
    buf.set_string(rx, oy + 5, trunc(left_hint, RIGHT_W as usize), s_dim);
    let rw = right_hint.len() as u16;
    if (left_hint.len() as u16 + rw + 4) <= RIGHT_W {
        buf.set_string(rx + RIGHT_W - rw - 1, oy + 5, right_hint, s_dim);
    }
}

fn draw_text_cursor(
    buf: &mut Buffer,
    input: &str,
    cursor_pos: usize,
    cursor_vis: bool,
    x: u16,
    y: u16,
    w: u16,
) {
    if w < 3 {
        return;
    }
    let s_white = Style::default().fg(Color::White).bg(BG);

    buf[(x, y)].set_char('>').set_style(s_white);
    buf[(x + 1, y)].set_char(' ').set_style(s_white);

    let base_x = x + 2;
    let cp = cursor_pos;
    let at_end = cp == input.len();
    let cursor_col = base_x + cp as u16;

    if cp > 0 && base_x < x + w {
        let avail = (x + w - base_x) as usize;
        buf.set_string(base_x, y, trunc(&input[..cp], avail), s_white);
    }

    if cursor_col < x + w {
        let ch = if at_end {
            ' '
        } else {
            input[cp..].chars().next().unwrap_or(' ')
        };
        if cursor_vis {
            buf[(cursor_col, y)]
                .set_char(ch)
                .set_fg(Color::Black)
                .set_bg(Color::White);
        } else if !at_end {
            buf[(cursor_col, y)].set_char(ch).set_style(s_white);
        }
    }

    if !at_end {
        let char_len = input[cp..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let after = &input[cp + char_len..];
        let after_col = cursor_col + 1;
        if !after.is_empty() && after_col < x + w {
            buf.set_string(
                after_col,
                y,
                trunc(after, (x + w - after_col) as usize),
                s_white,
            );
        }
    }
}

fn draw_sell_confirm_popup(buf: &mut Buffer, confirm: &SellConfirm, entry: &SellEntry, area: Rect) {
    if let SellEntry::Junk { qty } = entry {
        draw_qty_popup(
            buf,
            " Sell Junk ",
            confirm.sell_qty,
            *qty,
            JUNK_SELL_PRICE,
            "ENTER sell",
            area,
        );
        return;
    }
    if let SellEntry::Coffee { qty } = entry {
        draw_qty_popup(
            buf,
            " Sell Coffee ",
            confirm.sell_qty,
            *qty,
            COFFEE_SELL_PRICE,
            "ENTER sell",
            area,
        );
        return;
    }
    if let SellEntry::Bait { qty } = entry {
        draw_qty_popup(
            buf,
            " Sell Bait ",
            confirm.sell_qty,
            *qty,
            BAIT_SELL_PRICE,
            "ENTER sell",
            area,
        );
        return;
    }

    let pop_h: u16 = 5;
    let pop_w: u16 = 40;
    if area.width < pop_w || area.height < pop_h {
        return;
    }

    let ox = area.x + (area.width - pop_w) / 2;
    let oy = area.y + (area.height - pop_h) / 2;

    for dy in 0..pop_h {
        for dx in 0..pop_w {
            buf[(ox + dx, oy + dy)].reset();
            buf[(ox + dx, oy + dy)].set_bg(BG);
        }
    }

    let (title, msg) = match entry {
        SellEntry::Fish { name, species, .. } => {
            let t = format!(" Sell {} ", species.display_name());
            let inner_w = (pop_w - 4) as usize;
            let total = confirm.sell_qty * entry.unit_price();
            let m = format!(
                "Sell {} for ${}?",
                trunc(name, inner_w.saturating_sub(16)),
                total
            );
            (t, m)
        }
        SellEntry::Tank { name } => {
            let t = " Sell Fishtank ".to_string();
            let inner_w = (pop_w - 4) as usize;
            let total = confirm.sell_qty * entry.unit_price();
            let m = format!(
                "Sell {} for ${}?",
                trunc(name, inner_w.saturating_sub(16)),
                total
            );
            (t, m)
        }
        SellEntry::Junk { .. } | SellEntry::Coffee { .. } | SellEntry::Bait { .. } => {
            unreachable!()
        }
    };
    draw_border(buf, ox, oy, pop_w, pop_h, &title, false);

    let s_white = Style::default().fg(Color::White).bg(BG);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BG);
    let inner_x = ox + 2;
    let inner_w = (pop_w - 4) as usize;

    buf.set_string(inner_x, oy + 1, trunc(&msg, inner_w), s_white);

    let left_hint = "ESC/q cancel";
    let right_hint = "ENTER sell";
    let hint_y = oy + pop_h - 2;
    buf.set_string(inner_x, hint_y, trunc(left_hint, inner_w), s_dim);
    let rw = right_hint.len() as u16;
    if (left_hint.len() as u16 + rw + 2) <= inner_w as u16 {
        buf.set_string(inner_x + inner_w as u16 - rw, hint_y, right_hint, s_dim);
    }
}

fn draw_buy_tank_popup(buf: &mut Buffer, popup: &BuyTankPopup, cursor_vis: bool, area: Rect) {
    const POP_W: u16 = 36;
    const POP_H: u16 = 7;

    if area.width < POP_W || area.height < POP_H {
        return;
    }

    let ox = area.x + (area.width - POP_W) / 2;
    let oy = area.y + (area.height - POP_H) / 2;

    for dy in 0..POP_H {
        for dx in 0..POP_W {
            buf[(ox + dx, oy + dy)].reset();
            buf[(ox + dx, oy + dy)].set_bg(BG);
        }
    }

    draw_border(buf, ox, oy, POP_W, POP_H, " Buy Fishtank ", false);

    let s_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BG);
    let s_white = Style::default().fg(Color::White).bg(BG);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BG);
    let inner_x = ox + 2;
    let inner_w = (POP_W - 4) as usize;

    let header = format!("Fishtank for sale! Only ${}", TANK_BUY_PRICE);
    buf.set_string(inner_x, oy + 1, trunc(&header, inner_w), s_bold);
    buf.set_string(inner_x, oy + 3, "Name it", s_white);
    draw_text_cursor(
        buf,
        &popup.name_input,
        popup.cursor_pos,
        cursor_vis,
        inner_x,
        oy + 4,
        POP_W - 4,
    );

    let left_hint = "ESC/q cancel";
    let right_hint = "ENTER buy";
    buf.set_string(inner_x, oy + POP_H - 2, trunc(left_hint, inner_w), s_dim);
    let rw = right_hint.len() as u16;
    if (left_hint.len() as u16 + rw + 2) <= inner_w as u16 {
        buf.set_string(
            inner_x + inner_w as u16 - rw,
            oy + POP_H - 2,
            right_hint,
            s_dim,
        );
    }
}

fn vw(s: &str) -> usize {
    s.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
}

fn trunc(s: &str, max: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(1);
        if w + cw > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    out
}

fn pad(s: &str, width: usize) -> String {
    let w = vw(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - w))
    }
}
