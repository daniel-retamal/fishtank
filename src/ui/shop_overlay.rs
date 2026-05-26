use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use std::collections::HashMap;

use crate::{
    fishes::{fish::Fish, species::FishSpecies},
    loot::ConsumableKind,
    tank::TankKind,
    ui::{
        hints::{HINT_BACK, HINT_CANCEL, HINT_CLOSE, HINT_ENTER_BUY, HINT_ENTER_SELL, HINT_ENTER_SUMMON, HINT_NAV, HINT_SCROLL},
        render_fish_segs, scroll_list, table,
        text_input::{TextInput, draw_text_cursor},
    },
};

const BACKGROUND: Color = Color::Reset;
const PENGUIN_WIDTH: u16 = 7;
const RIGHT_INNER_WIDTH: u16 = 34;
const INNER_WIDTH: u16 = PENGUIN_WIDTH + 1 + RIGHT_INNER_WIDTH;
const OVERLAY_WIDTH: u16 = INNER_WIDTH + 2;
const PENGUIN_HEIGHT: u16 = 4;
const PENGUIN_BOX_ROW: u16 = PENGUIN_HEIGHT + 1;
const MAX_LIST_VISIBLE: usize = 7;
const BUY_CAT_ITEM_COUNT: usize = 5;

const PENGUIN_LINES: &[&str] = &["  __   ", " ( o>  ", " ///\\  ", " \\V_/_ "];

pub const FOOD_BUY_PRICE: u32 = 1;
pub const JUNK_SELL_PRICE: u32 = 1;
const NECRONOMICON_SELL_PRICE: u32 = 7_000;


pub struct BuyCategoryPopup {
    pub option_idx: usize,
    pub qty: u32,
    pub max_qty: u32,
}

pub struct BuyTankPopup {
    pub catalog_idx: usize,
    pub name_input: TextInput,
}

pub fn buy_cat_available(idx: usize, cash: u32) -> bool {
    match idx {
        0 => FishSpecies::all_buyable().iter().any(|s| s.buy_price() <= cash),
        1 => cash >= ConsumableKind::Coffee.buy_price(),
        2 => cash >= ConsumableKind::Bait.buy_price(),
        3 => cash >= FOOD_BUY_PRICE,
        4 => TankKind::all().iter().any(|k| k.buy_price() <= cash),
        _ => false,
    }
}

pub fn buy_cat_first_available(cash: u32) -> usize {
    (0..5).find(|&i| buy_cat_available(i, cash)).unwrap_or(0)
}

pub fn buy_cat_next(current: usize, down: bool, cash: u32) -> usize {
    let step: i32 = if down { 1 } else { -1 };
    let mut idx = current as i32 + step;
    while (0..5).contains(&idx) {
        if buy_cat_available(idx as usize, cash) {
            return idx as usize;
        }
        idx += step;
    }
    current
}

pub struct FishNamePopup {
    pub catalog_idx: usize,
    pub fish: Fish,
    pub name_input: TextInput,
}

pub enum SellEntry {
    Fish {
        name: String,
        species: FishSpecies,
        sell_value: u32,
    },
    Junk {
        qty: u32,
    },
    Coffee {
        qty: u32,
    },
    Bait {
        qty: u32,
    },
    Tank {
        name: String,
        sell_price: u32,
    },
    Necronomicon {
        qty: u32,
    },
}

impl SellEntry {
    pub fn label(&self) -> String {
        match self {
            SellEntry::Fish { name, species, .. } => {
                format!("{} ({})", name, species.display_name())
            }
            SellEntry::Junk { qty } => format!("Junk ({})", qty),
            SellEntry::Coffee { qty } => format!("Coffee ({})", qty),
            SellEntry::Bait { qty } => format!("Bait ({})", qty),
            SellEntry::Tank { name, .. } => format!("{} (Fishtank)", name),
            SellEntry::Necronomicon { qty } => format!("Necronomicon ({})", qty),
        }
    }

    pub fn price_label(&self) -> String {
        match self {
            SellEntry::Fish { sell_value, .. } => format!("${}", sell_value),
            SellEntry::Junk { .. } => format!("${}", JUNK_SELL_PRICE),
            SellEntry::Coffee { .. } => format!("${}", ConsumableKind::Coffee.sell_price()),
            SellEntry::Bait { .. } => format!("${}", ConsumableKind::Bait.sell_price()),
            SellEntry::Tank { sell_price, .. } => format!("${}", sell_price),
            SellEntry::Necronomicon { .. } => format!("${}", NECRONOMICON_SELL_PRICE),
        }
    }

    pub fn unit_price(&self) -> u32 {
        match self {
            SellEntry::Fish { sell_value, .. } => *sell_value,
            SellEntry::Junk { .. } => JUNK_SELL_PRICE,
            SellEntry::Coffee { .. } => ConsumableKind::Coffee.sell_price(),
            SellEntry::Bait { .. } => ConsumableKind::Bait.sell_price(),
            SellEntry::Tank { sell_price, .. } => *sell_price,
            SellEntry::Necronomicon { .. } => NECRONOMICON_SELL_PRICE,
        }
    }

    pub fn max_qty(&self) -> u32 {
        match self {
            SellEntry::Fish { .. } | SellEntry::Tank { .. } => 1,
            SellEntry::Junk { qty }
            | SellEntry::Coffee { qty }
            | SellEntry::Bait { qty }
            | SellEntry::Necronomicon { qty } => *qty,
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
        sellable_tanks: &[(String, u32)],
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
        let necro_qty = inventory.get("Necronomicon").copied().unwrap_or(0);
        if necro_qty > 0 {
            items.push(SellEntry::Necronomicon { qty: necro_qty });
        }
        for (tank_name, sell_price) in sellable_tanks {
            items.push(SellEntry::Tank {
                name: tank_name.clone(),
                sell_price: *sell_price,
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
            let pa = match a {
                SellEntry::Fish { sell_value, .. } => *sell_value,
                _ => 0,
            };
            let pb = match b {
                SellEntry::Fish { sell_value, .. } => *sell_value,
                _ => 0,
            };
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
        scroll_list::scroll_up(&mut self.selected, &mut self.scroll);
    }

    pub fn scroll_down(&mut self, visible: usize) {
        scroll_list::scroll_down(&mut self.selected, &mut self.scroll, self.items.len(), visible);
    }
}

pub struct FishListState {
    pub selected: usize,
    pub scroll: usize,
    pub popup: Option<FishNamePopup>,
}

impl FishListState {
    pub fn new(cash: u32) -> Self {
        let selected = FishSpecies::all_buyable()
            .iter()
            .position(|s| s.buy_price() <= cash)
            .unwrap_or(0);
        Self {
            selected,
            scroll: 0,
            popup: None,
        }
    }

    pub fn scroll_up(&mut self, cash: u32) {
        let catalog = FishSpecies::all_buyable();
        let mut idx = self.selected;
        while idx > 0 {
            idx -= 1;
            if catalog[idx].buy_price() <= cash {
                self.selected = idx;
                if self.selected < self.scroll {
                    self.scroll = self.selected;
                }
                return;
            }
        }
    }

    pub fn scroll_down(&mut self, cash: u32, visible: usize) {
        let catalog = FishSpecies::all_buyable();
        let n = catalog.len();
        let mut idx = self.selected;
        loop {
            idx += 1;
            if idx >= n {
                return;
            }
            if catalog[idx].buy_price() <= cash {
                self.selected = idx;
                if self.selected >= self.scroll + visible {
                    self.scroll = self.selected + 1 - visible;
                }
                return;
            }
        }
    }
}

pub struct TankListState {
    pub selected: usize,
    pub scroll: usize,
    pub popup: Option<BuyTankPopup>,
}

impl TankListState {
    pub fn new(cash: u32) -> Self {
        let selected = TankKind::all()
            .iter()
            .position(|k| k.buy_price() <= cash)
            .unwrap_or(0);
        Self {
            selected,
            scroll: 0,
            popup: None,
        }
    }

    pub fn scroll_up(&mut self, cash: u32) {
        let mut idx = self.selected;
        while idx > 0 {
            idx -= 1;
            if TankKind::all()[idx].buy_price() <= cash {
                self.selected = idx;
                if self.selected < self.scroll {
                    self.scroll = self.selected;
                }
                return;
            }
        }
    }

    pub fn scroll_down(&mut self, cash: u32, visible: usize) {
        let n = TankKind::all().len();
        let mut idx = self.selected;
        loop {
            idx += 1;
            if idx >= n {
                return;
            }
            if TankKind::all()[idx].buy_price() <= cash {
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
    },
    BuyFishList(FishListState),
    BuyTankList(TankListState),
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
    pub cash: u32,
}

impl<'a> ShopOverlay<'a> {
    pub fn new(state: &'a ShopState, cash: u32) -> Self {
        Self { state, cash }
    }
}

fn overlay_h(state: &ShopState, area_h: u16) -> u16 {
    const MIN_OH: u16 = PENGUIN_BOX_ROW + 2;
    const MAX_OH: u16 = 13;
    match &state.page {
        ShopPage::Main { .. } => MIN_OH.min(area_h),
        ShopPage::BuyCategory { .. } => (MIN_OH + 3).min(area_h),
        ShopPage::BuyFishList(_) => {
            let visible = FishSpecies::all_buyable().len().min(MAX_LIST_VISIBLE);
            ((visible as u16) + 6).clamp(MIN_OH, MAX_OH).min(area_h)
        }
        ShopPage::BuyTankList(_) => {
            let visible = TankKind::all().len().min(MAX_LIST_VISIBLE);
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

        if area.width < OVERLAY_WIDTH || area.height < oh {
            return;
        }

        let ox = area.x + (area.width - OVERLAY_WIDTH) / 2;
        let oy = area.y + (area.height - oh) / 2;

        let has_popup = match &state.page {
            ShopPage::BuyCategory { buy_popup, .. } => buy_popup.is_some(),
            ShopPage::BuyFishList(fl) => fl.popup.is_some(),
            ShopPage::BuyTankList(tl) => tl.popup.is_some(),
            ShopPage::Sell(sm) => sm.confirm.is_some(),
            _ => false,
        };

        let fg = if has_popup {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(fg).bg(BACKGROUND);
        let s_title = Style::default().fg(fg).add_modifier(Modifier::BOLD).bg(BACKGROUND);
        let sep_x = ox + 1 + PENGUIN_WIDTH;
        let right_x = ox + OVERLAY_WIDTH - 1;

        for dy in 0..=PENGUIN_BOX_ROW {
            for dx in 0..OVERLAY_WIDTH {
                buf[(ox + dx, oy + dy)].reset();
            }
        }
        for dy in PENGUIN_BOX_ROW + 1..oh {
            for dx in PENGUIN_WIDTH + 1..OVERLAY_WIDTH {
                buf[(ox + dx, oy + dy)].reset();
            }
        }

        buf[(ox, oy)].set_char('┌').set_style(s);
        for dx in 1..=PENGUIN_WIDTH {
            buf[(ox + dx, oy)].set_char('─').set_style(s);
        }
        buf[(sep_x, oy)].set_char('┬').set_style(s);
        for x in sep_x + 1..right_x {
            buf[(x, oy)].set_char('─').set_style(s);
        }
        buf[(right_x, oy)].set_char('┐').set_style(s);
        let page_title = match &state.page {
            ShopPage::Main { .. } => " Shop ",
            ShopPage::BuyCategory { .. } | ShopPage::BuyFishList(_) | ShopPage::BuyTankList(_) => {
                " Buy "
            }
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
        for dx in 1..=PENGUIN_WIDTH {
            buf[(ox + dx, oy + PENGUIN_BOX_ROW)]
                .set_char('─')
                .set_style(s);
        }

        buf[(sep_x, oy + oh - 1)].set_char('└').set_style(s);
        for dx in 1..=RIGHT_INNER_WIDTH {
            buf[(sep_x + dx, oy + oh - 1)].set_char('─').set_style(s);
        }
        buf[(right_x, oy + oh - 1)].set_char('┘').set_style(s);

        draw_penguin(buf, ox + 1, oy + 1, has_popup);

        let rx = sep_x + 1;
        let content_y = oy + 1;
        let footer_y = oy + oh - 2;
        let footer_w = RIGHT_INNER_WIDTH - 1;

        match &state.page {
            ShopPage::Main { selected } => {
                let content_h = oh.saturating_sub(3);
                draw_main_right(
                    buf,
                    *selected,
                    state.cursor_visible,
                    rx,
                    content_y,
                    RIGHT_INNER_WIDTH,
                    content_h,
                    has_popup,
                );
                table::draw_hint_bar(buf, rx, footer_y, footer_w, HINT_NAV, HINT_CLOSE, BACKGROUND);
            }
            ShopPage::BuyCategory {
                selected,
                buy_popup,
            } => {
                let content_h = oh.saturating_sub(3);
                let available = [
                    buy_cat_available(0, self.cash),
                    buy_cat_available(1, self.cash),
                    buy_cat_available(2, self.cash),
                    buy_cat_available(3, self.cash),
                    buy_cat_available(4, self.cash),
                ];
                let draw_h = content_h.saturating_sub(1);
                draw_buy_category_right(
                    buf,
                    *selected,
                    state.cursor_visible,
                    &available,
                    rx,
                    content_y + 1,
                    RIGHT_INNER_WIDTH,
                    draw_h,
                    has_popup,
                );
                let visible = draw_h as usize;
                let footer_left = if BUY_CAT_ITEM_COUNT > visible {
                    format!(" {} ({}/{})", HINT_SCROLL, selected + 1, BUY_CAT_ITEM_COUNT)
                } else {
                    format!(" {}", HINT_NAV)
                };
                table::draw_hint_bar(buf, rx, footer_y, footer_w, &footer_left, HINT_BACK, BACKGROUND);
                if let Some(popup) = buy_popup {
                    draw_category_buy_popup(buf, popup, area);
                }
            }
            ShopPage::BuyTankList(tl) => {
                let visible = oh.saturating_sub(6) as usize;
                let right_border_x = ox + OVERLAY_WIDTH - 1;
                draw_tank_list(
                    buf,
                    tl,
                    self.cash,
                    state.cursor_visible,
                    sep_x,
                    content_y,
                    right_border_x,
                    RIGHT_INNER_WIDTH,
                    visible,
                    has_popup,
                );
                let affordable_count = TankKind::all().iter().filter(|k| k.buy_price() <= self.cash).count();
                let affordable_pos = TankKind::all()[..=tl.selected]
                    .iter()
                    .filter(|k| k.buy_price() <= self.cash)
                    .count();
                let footer_left = format!(" {} ({}/{})", HINT_SCROLL, affordable_pos, affordable_count);
                table::draw_hint_bar(buf, rx, footer_y, footer_w, &footer_left, HINT_BACK, BACKGROUND);
                if let Some(ref popup) = tl.popup {
                    draw_buy_tank_name_popup(buf, popup, state.cursor_visible, area);
                }
            }
            ShopPage::BuyFishList(fl) => {
                let visible = oh.saturating_sub(6) as usize;
                let right_border_x = ox + OVERLAY_WIDTH - 1;
                draw_fish_list(
                    buf,
                    fl,
                    self.cash,
                    state.cursor_visible,
                    sep_x,
                    content_y,
                    right_border_x,
                    RIGHT_INNER_WIDTH,
                    visible,
                    has_popup,
                );
                let catalog = FishSpecies::all_buyable();
                let affordable_count = catalog.iter().filter(|s| s.buy_price() <= self.cash).count();
                let affordable_pos = catalog[..=fl.selected].iter().filter(|s| s.buy_price() <= self.cash).count();
                let footer_left = format!(" {} ({}/{})", HINT_SCROLL, affordable_pos, affordable_count);
                table::draw_hint_bar(buf, rx, footer_y, footer_w, &footer_left, HINT_BACK, BACKGROUND);
                if let Some(ref popup) = fl.popup {
                    draw_fish_name_popup(buf, popup, state.cursor_visible, area);
                }
            }
            ShopPage::Sell(sm) => {
                let visible = oh.saturating_sub(6) as usize;
                let right_border_x = ox + OVERLAY_WIDTH - 1;
                draw_sell_list(
                    buf,
                    sm,
                    state.cursor_visible,
                    sep_x,
                    content_y,
                    right_border_x,
                    RIGHT_INNER_WIDTH,
                    visible,
                    has_popup,
                );
                let n = sm.items.len();
                let footer_left = format!(" {} ({}/{})", HINT_SCROLL, sm.selected + 1, n);
                table::draw_hint_bar(buf, rx, footer_y, footer_w, &footer_left, HINT_BACK, BACKGROUND);
                if let Some(ref confirm) = sm.confirm {
                    draw_sell_confirm_popup(buf, confirm, &sm.items[confirm.item_idx], area);
                }
            }
        }
    }
}

fn draw_penguin(buf: &mut Buffer, x: u16, y: u16, dim: bool) {
    let fg = if dim { Color::DarkGray } else { Color::White };
    let s = Style::default().fg(fg).bg(BACKGROUND);
    for (i, line) in PENGUIN_LINES.iter().enumerate() {
        if i < PENGUIN_HEIGHT as usize {
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
        let s = Style::default().fg(fg).bg(BACKGROUND);
        buf.set_string(x, row_y, table::truncate_str(&line, w as usize), s);
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
        let s = Style::default().fg(fg).bg(BACKGROUND);
        buf.set_string(x, y + row as u16, table::truncate_str(&line, w as usize), s);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_fish_list(
    buf: &mut Buffer,
    state: &FishListState,
    cash: u32,
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
        .bg(BACKGROUND);
    let s_sep = Style::default().fg(hdr_fg).bg(BACKGROUND);

    buf.set_string(
        rx,
        content_y,
        table::pad_right("Name", (right_w / 2) as usize),
        s_bold,
    );
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
    let catalog = FishSpecies::all_buyable();
    let n = catalog.len();
    let eff_cursor_vis = dim || cursor_vis;

    for row in 0..visible {
        let idx = state.scroll + row;
        if idx >= n {
            break;
        }
        let species = catalog[idx];
        let price = species.buy_price();
        let row_y = data_y + row as u16;
        let affordable = price <= cash;
        let is_sel = idx == state.selected;

        let price_str = format!("${}", price);
        let price_x = right_border_x.saturating_sub(1 + price_str.len() as u16);
        let name_max = price_x.saturating_sub(rx + 2) as usize;

        let item_fg = if dim || !affordable {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(item_fg).bg(BACKGROUND);

        let prefix = if is_sel && eff_cursor_vis { "> " } else { "  " };
        let label = format!("{}{}", prefix, table::truncate_str(species.display_name(), name_max));
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
        .bg(BACKGROUND);
    let s_sep = Style::default().fg(hdr_fg).bg(BACKGROUND);
    let s_white = Style::default()
        .fg(if dim { Color::DarkGray } else { Color::White })
        .bg(BACKGROUND);

    buf.set_string(
        rx,
        content_y,
        table::pad_right("Item", (right_w / 2) as usize),
        s_bold,
    );
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
        let label = format!(
            "{}{}",
            prefix,
            table::truncate_str(&entry.label(), name_max)
        );
        buf.set_string(rx, row_y, label, s_white);
        buf.set_string(price_x, row_y, &price_str, s_white);
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
    buf.set_string(col, y, table::truncate_str(total_str, remaining), mid_s);
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
    const POP_WIDTH: u16 = 28;
    const POP_HEIGHT: u16 = 5;
    let Some(layout) = table::OverlayLayout::centered(area, POP_WIDTH, POP_HEIGHT) else {
        return;
    };
    layout.clear_bg(buf, BACKGROUND);
    layout.draw_border(buf, title, Color::White, BACKGROUND);

    let (ox, oy) = (layout.ox, layout.oy);
    let s_white = Style::default().fg(Color::White).bg(BACKGROUND);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BACKGROUND);
    let inner_x = ox + 2;
    let inner_w = (POP_WIDTH - 4) as usize;

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

    table::draw_hint_bar(buf, inner_x, oy + POP_HEIGHT - 2, inner_w as u16, HINT_CANCEL, confirm_hint, BACKGROUND);
}

fn draw_category_buy_popup(buf: &mut Buffer, popup: &BuyCategoryPopup, area: Rect) {
    let (title, unit_price) = match popup.option_idx {
        1 => (" Buy Coffee ", ConsumableKind::Coffee.buy_price()),
        2 => (" Buy Bait ", ConsumableKind::Bait.buy_price()),
        _ => (" Buy Food ", FOOD_BUY_PRICE),
    };
    draw_qty_popup(
        buf,
        title,
        popup.qty,
        popup.max_qty,
        unit_price,
        HINT_ENTER_BUY,
        area,
    );
}

fn draw_fish_name_popup(buf: &mut Buffer, popup: &FishNamePopup, cursor_vis: bool, area: Rect) {
    const RIGHT_WIDTH: u16 = 30;
    const POP_HEIGHT: u16 = 7;
    const CONTENT_HEIGHT: u16 = 5;

    let fish = &popup.fish;
    let fish_dw = fish.display_width as u16;
    let left_w = fish_dw + 3;
    let inner_w = left_w + 1 + RIGHT_WIDTH;
    let pop_w = inner_w + 2;

    let Some(layout) = table::OverlayLayout::centered(area, pop_w, POP_HEIGHT) else {
        return;
    };
    layout.clear_bg(buf, BACKGROUND);
    let species = FishSpecies::all_buyable()[popup.catalog_idx];
    let species_name = species.display_name();
    let price = species.buy_price();
    let title = format!(" Buy {} ", species_name);
    layout.draw_border(buf, &title, Color::White, BACKGROUND);

    let (ox, oy) = (layout.ox, layout.oy);

    let sep_x = ox + 1 + left_w;
    buf[(sep_x, oy + POP_HEIGHT - 1)]
        .set_char('┴')
        .set_fg(Color::White)
        .set_bg(BACKGROUND);
    for row in 1..=CONTENT_HEIGHT {
        buf[(sep_x, oy + row)]
            .set_char('│')
            .set_fg(Color::White)
            .set_bg(BACKGROUND);
    }

    let fish_x = ox + 2;
    let fish_y = oy + 1 + CONTENT_HEIGHT / 2;
    let segs = fish.segments();
    render_fish_segs(buf, &segs, fish_x, fish_y, fish_dw, BACKGROUND);

    let rx = sep_x + 1;
    let s_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BACKGROUND);
    let s_white = Style::default().fg(Color::White).bg(BACKGROUND);

    buf.set_string(
        rx,
        oy + 1,
        table::truncate_str(
            &format!("{} for sale! Only ${}", species_name, price),
            RIGHT_WIDTH as usize,
        ),
        s_bold,
    );
    buf.set_string(rx, oy + 3, "Name it", s_white);
    draw_text_cursor(buf, &popup.name_input, cursor_vis, rx, oy + 4, RIGHT_WIDTH, BACKGROUND);

    table::draw_hint_bar(buf, rx, oy + 5, RIGHT_WIDTH, "ESC cancel", HINT_ENTER_BUY, BACKGROUND);
}

fn draw_sell_confirm_popup(buf: &mut Buffer, confirm: &SellConfirm, entry: &SellEntry, area: Rect) {
    if let SellEntry::Junk { qty } = entry {
        draw_qty_popup(
            buf,
            " Sell Junk ",
            confirm.sell_qty,
            *qty,
            JUNK_SELL_PRICE,
            HINT_ENTER_SELL,
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
            ConsumableKind::Coffee.sell_price(),
            HINT_ENTER_SELL,
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
            ConsumableKind::Bait.sell_price(),
            HINT_ENTER_SELL,
            area,
        );
        return;
    }
    if let SellEntry::Necronomicon { qty } = entry {
        draw_qty_popup(
            buf,
            " Sell Necronomicon ",
            confirm.sell_qty,
            *qty,
            NECRONOMICON_SELL_PRICE,
            HINT_ENTER_SELL,
            area,
        );
        return;
    }

    let pop_h: u16 = 5;
    let pop_w: u16 = 40;
    let Some(layout) = table::OverlayLayout::centered(area, pop_w, pop_h) else {
        return;
    };
    layout.clear_bg(buf, BACKGROUND);
    let (title, msg) = match entry {
        SellEntry::Fish { name, species, .. } => {
            let t = format!(" Sell {} ", species.display_name());
            let inner_w = (pop_w - 4) as usize;
            let total = confirm.sell_qty * entry.unit_price();
            let m = format!(
                "Sell {} for ${}?",
                table::truncate_str(name, inner_w.saturating_sub(16)),
                total
            );
            (t, m)
        }
        SellEntry::Tank { name, .. } => {
            let t = " Sell Fishtank ".to_string();
            let inner_w = (pop_w - 4) as usize;
            let total = confirm.sell_qty * entry.unit_price();
            let m = format!(
                "Sell {} for ${}?",
                table::truncate_str(name, inner_w.saturating_sub(16)),
                total
            );
            (t, m)
        }
        SellEntry::Junk { .. }
        | SellEntry::Coffee { .. }
        | SellEntry::Bait { .. }
        | SellEntry::Necronomicon { .. } => unreachable!(),
    };
    layout.draw_border(buf, &title, Color::White, BACKGROUND);

    let (ox, oy) = (layout.ox, layout.oy);
    let s_white = Style::default().fg(Color::White).bg(BACKGROUND);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BACKGROUND);
    let inner_x = ox + 2;
    let inner_w = (pop_w - 4) as usize;

    buf.set_string(inner_x, oy + 1, table::truncate_str(&msg, inner_w), s_white);

    let left_hint = HINT_CANCEL;
    let right_hint = HINT_ENTER_SELL;
    let hint_y = oy + pop_h - 2;
    buf.set_string(
        inner_x,
        hint_y,
        table::truncate_str(left_hint, inner_w),
        s_dim,
    );
    let rw = right_hint.len() as u16;
    if (left_hint.len() as u16 + rw + 2) <= inner_w as u16 {
        buf.set_string(inner_x + inner_w as u16 - rw, hint_y, right_hint, s_dim);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_tank_list(
    buf: &mut Buffer,
    state: &TankListState,
    cash: u32,
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
        .bg(BACKGROUND);
    let s_sep = Style::default().fg(hdr_fg).bg(BACKGROUND);

    buf.set_string(
        rx,
        content_y,
        table::pad_right("Name", (right_w / 2) as usize),
        s_bold,
    );
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
    let all_kinds = TankKind::all();
    let n = all_kinds.len();
    let eff_cursor_vis = dim || cursor_vis;

    for row in 0..visible {
        let idx = state.scroll + row;
        if idx >= n {
            break;
        }
        let kind = all_kinds[idx];
        let row_y = data_y + row as u16;
        let affordable = kind.buy_price() <= cash;
        let is_sel = idx == state.selected;

        let price_str = format!("${}", kind.buy_price());
        let price_x = right_border_x.saturating_sub(1 + price_str.len() as u16);
        let name_max = price_x.saturating_sub(rx + 2) as usize;

        let item_fg = if dim || !affordable {
            Color::DarkGray
        } else {
            Color::White
        };
        let s = Style::default().fg(item_fg).bg(BACKGROUND);

        let prefix = if is_sel && eff_cursor_vis { "> " } else { "  " };
        let label = format!("{}{}", prefix, table::truncate_str(kind.shop_name(), name_max));
        buf.set_string(rx, row_y, label, s);
        buf.set_string(price_x, row_y, &price_str, s);
    }
}

fn draw_buy_tank_name_popup(buf: &mut Buffer, popup: &BuyTankPopup, cursor_vis: bool, area: Rect) {
    const POP_HEIGHT: u16 = 7;
    const LEFT_HINT: &str = "ESC cancel";
    const RIGHT_HINT: &str = HINT_ENTER_BUY;

    let kind = TankKind::all()[popup.catalog_idx];
    let header = format!("{} for sale! Only ${}", kind.shop_name(), kind.buy_price());
    let min_hints_w = (LEFT_HINT.len() + RIGHT_HINT.len() + 2) as u16;
    let inner_w = (header.len() as u16).max(min_hints_w);
    let pop_w = inner_w + 4;

    let Some(layout) = table::OverlayLayout::centered(area, pop_w, POP_HEIGHT) else {
        return;
    };
    layout.clear_bg(buf, BACKGROUND);
    let title = format!(" Buy {} ", kind.shop_name());
    layout.draw_border(buf, &title, Color::White, BACKGROUND);

    let (ox, oy) = (layout.ox, layout.oy);
    let s_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BACKGROUND);
    let s_white = Style::default().fg(Color::White).bg(BACKGROUND);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BACKGROUND);
    let inner_x = ox + 2;
    let inner_w_usize = inner_w as usize;

    buf.set_string(inner_x, oy + 1, &header, s_bold);
    buf.set_string(inner_x, oy + 3, "Name it", s_white);
    draw_text_cursor(
        buf,
        &popup.name_input,
        cursor_vis,
        inner_x,
        oy + 4,
        inner_w,
        BACKGROUND,
    );

    buf.set_string(
        inner_x,
        oy + POP_HEIGHT - 2,
        table::truncate_str(LEFT_HINT, inner_w_usize),
        s_dim,
    );
    let rw = RIGHT_HINT.len() as u16;
    if (LEFT_HINT.len() as u16 + rw + 2) <= inner_w {
        buf.set_string(inner_x + inner_w - rw, oy + POP_HEIGHT - 2, RIGHT_HINT, s_dim);
    }
}

pub struct NecroPopupWidget<'a> {
    pub input: &'a TextInput,
    pub cursor_visible: bool,
}

impl Widget for NecroPopupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        draw_necro_tank_name_popup(buf, self.input, self.cursor_visible, area);
    }
}

pub fn draw_necro_tank_name_popup(
    buf: &mut Buffer,
    input: &TextInput,
    cursor_vis: bool,
    area: Rect,
) {
    const POP_HEIGHT: u16 = 6;
    const LEFT_HINT: &str = "ESC cancel";
    const RIGHT_HINT: &str = HINT_ENTER_SUMMON;
    const HEADER: &str = "Name your Helltank";

    let min_hints_w = (LEFT_HINT.len() + RIGHT_HINT.len() + 2) as u16;
    let inner_w = (HEADER.len() as u16).max(min_hints_w);
    let pop_w = inner_w + 4;

    let Some(layout) = table::OverlayLayout::centered(area, pop_w, POP_HEIGHT) else {
        return;
    };
    layout.clear_bg(buf, BACKGROUND);
    layout.draw_border(buf, " Necronomicon ", Color::Red, BACKGROUND);

    let (ox, oy) = (layout.ox, layout.oy);
    let s_bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(BACKGROUND);
    let s_dim = Style::default().fg(Color::DarkGray).bg(BACKGROUND);
    let inner_x = ox + 2;
    let inner_w_usize = inner_w as usize;

    buf.set_string(inner_x, oy + 1, HEADER, s_bold);
    draw_text_cursor(buf, input, cursor_vis, inner_x, oy + 2, inner_w, BACKGROUND);

    buf.set_string(
        inner_x,
        oy + POP_HEIGHT - 2,
        table::truncate_str(LEFT_HINT, inner_w_usize),
        s_dim,
    );
    let rw = RIGHT_HINT.len() as u16;
    if (LEFT_HINT.len() as u16 + rw + 2) <= inner_w {
        buf.set_string(inner_x + inner_w - rw, oy + POP_HEIGHT - 2, RIGHT_HINT, s_dim);
    }
}
