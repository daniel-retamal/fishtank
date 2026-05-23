use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::entities::fish::{Direction, Fish};
use crate::entities::species::FishSpecies;
use crate::entities::unfish::{BALL_HEIGHT, SKULL_HEIGHT, UnfishKind};
use crate::ui::{fields, render_fish_segs, scroll_list, table, tank_view};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FantasyKind {
    Iq,
    ZodiacSign,
    ChineseZodiac,
    Tanganana,
    HappinessLevel,
    Claustrophobic,
    AttrText,
    AttrEnumBar,
    AttrBoolean,
    PickACard,
    FavoriteColor,
    FavoriteLetter,
    FavoriteNumber,
    TarotPrediction,
    FavoriteQuote,
    FavoriteTime,
    Lonely,
    Gmi,
    ElOLaPr,
    TheOrThePr,
    Crush,
    MarriedTo,
    Hates,
    Status,
    Sin,
    HasSeenTheSky,
    Temperature,
    Delicious,
    Region,
    Dni,
    FavoriteSeason,
}

impl FantasyKind {
    fn all() -> &'static [FantasyKind] {
        use FantasyKind::*;
        &[
            Iq,
            ZodiacSign,
            ChineseZodiac,
            Tanganana,
            HappinessLevel,
            Claustrophobic,
            AttrText,
            AttrEnumBar,
            AttrBoolean,
            PickACard,
            FavoriteColor,
            FavoriteLetter,
            FavoriteNumber,
            TarotPrediction,
            FavoriteQuote,
            FavoriteTime,
            Lonely,
            Gmi,
            ElOLaPr,
            TheOrThePr,
            Crush,
            MarriedTo,
            Hates,
            Status,
            Sin,
            HasSeenTheSky,
            Temperature,
            Delicious,
            Region,
            Dni,
            FavoriteSeason,
        ]
    }

    fn header(self) -> &'static str {
        match self {
            FantasyKind::Iq => "IQ",
            FantasyKind::ZodiacSign => "Zodiac Sign",
            FantasyKind::ChineseZodiac => "Chinese Zodiac Sign",
            FantasyKind::Tanganana => "Tangananica or Tanganana?",
            FantasyKind::HappinessLevel => "Happiness Level",
            FantasyKind::Claustrophobic => "Claustrophobic?",
            FantasyKind::AttrText => "Attr text",
            FantasyKind::AttrEnumBar => "Attr enum Bar",
            FantasyKind::AttrBoolean => "Attr boolean",
            FantasyKind::PickACard => "Pick a card",
            FantasyKind::FavoriteColor => "Favorite Color",
            FantasyKind::FavoriteLetter => "Favorite Letter",
            FantasyKind::FavoriteNumber => "Favorite Number",
            FantasyKind::TarotPrediction => "Tarot Prediction",
            FantasyKind::FavoriteQuote => "Favorite Quote",
            FantasyKind::FavoriteTime => "Favorite Hour",
            FantasyKind::Lonely => "Lonely?",
            FantasyKind::Gmi => "gmi?",
            FantasyKind::ElOLaPr => "El o La PR?",
            FantasyKind::TheOrThePr => "The or The PR?",
            FantasyKind::Crush => "Crush",
            FantasyKind::MarriedTo => "Married to",
            FantasyKind::Hates => "Hates",
            FantasyKind::Status => "Status",
            FantasyKind::Sin => "Sin",
            FantasyKind::HasSeenTheSky => "Has seen the sky?",
            FantasyKind::Temperature => "Temperature (ºC)",
            FantasyKind::Delicious => "Delicious?",
            FantasyKind::Region => "Region",
            FantasyKind::Dni => "DNI",
            FantasyKind::FavoriteSeason => "Favorite Season",
        }
    }
}

struct FantasyCell {
    text: String,
    swatch: Option<Color>,
}

struct FantasyColumn {
    kind: FantasyKind,
    cells: Vec<FantasyCell>,
    col_width: usize,
}

pub struct FishSnapshot {
    name: String,
    species_name: &'static str,
    segments: Vec<(char, Color)>,
    display_width: usize,
    weight_g: u32,
    tank_name: Option<String>,
    display_height: u16,
    is_unfish: bool,
}

pub struct IndexState {
    pub selected: usize,
    pub scroll: usize,
    pub col_scroll: usize,
    snapshots: Vec<FishSnapshot>,
    fish_clones: Vec<Fish>,
    fixed_widths: Vec<usize>,
    show_tank_col: bool,
    fantasy_cols: Vec<FantasyColumn>,
    animated_fish: Option<Fish>,
}

fn unfish_row_height(fish: &Fish) -> u16 {
    match fish.unfish_state.as_ref().map(|us| us.kind) {
        Some(UnfishKind::Ball) => BALL_HEIGHT,
        Some(UnfishKind::Skull) => SKULL_HEIGHT,
        _ => 1,
    }
}

impl IndexState {
    pub fn new(fish_with_tanks: &[(&str, &Fish)], all: bool, show_tank_col: bool) -> Self {
        let mut rng = rand::rng();

        let filtered: Vec<(&str, &Fish)> = fish_with_tanks
            .iter()
            .filter(|(_, f)| !f.is_invisible())
            .copied()
            .collect();

        let snapshots: Vec<FishSnapshot> = filtered
            .iter()
            .map(|(tank_name, f)| FishSnapshot {
                name: f.name.clone(),
                species_name: f.species.display_name(),
                segments: f.static_left_segments(),
                display_width: f.display_width,
                weight_g: f.weight_g,
                tank_name: Some(tank_name.to_string()),
                display_height: unfish_row_height(f),
                is_unfish: f.unfish_state.is_some(),
            })
            .collect();

        let fish_clones: Vec<Fish> = filtered.iter().map(|(_, f)| (*f).clone()).collect();

        let chosen_kinds: Vec<FantasyKind> = if all {
            FantasyKind::all().to_vec()
        } else {
            let count = rng.random_range(0..=3usize);
            let mut avail: Vec<FantasyKind> = FantasyKind::all().to_vec();
            let mut chosen = Vec::new();
            for _ in 0..count.min(avail.len()) {
                let idx = rng.random_range(0..avail.len());
                chosen.push(avail.remove(idx));
            }
            chosen
        };

        let fish_names: Vec<String> = filtered.iter().map(|(_, f)| f.name.clone()).collect();
        let species_list: Vec<FishSpecies> = filtered.iter().map(|(_, f)| f.species).collect();
        let is_unfish_flags: Vec<bool> = filtered
            .iter()
            .map(|(_, f)| f.unfish_state.is_some())
            .collect();

        let fantasy_cols = chosen_kinds
            .into_iter()
            .map(|kind| {
                let mut cells = gen_fantasy(kind, &fish_names, &species_list, &mut rng);
                for (i, &is_uf) in is_unfish_flags.iter().enumerate() {
                    if is_uf && i < cells.len() {
                        cells[i] = plain("");
                    }
                }
                let max_cell_w = cells
                    .iter()
                    .map(|c| table::visual_width(&c.text))
                    .max()
                    .unwrap_or(0);
                let col_width = max_cell_w.max(table::visual_width(kind.header())).max(4);
                FantasyColumn {
                    kind,
                    cells,
                    col_width,
                }
            })
            .collect::<Vec<_>>();

        let max_name_w = snapshots
            .iter()
            .map(|s| s.name.len())
            .max()
            .unwrap_or(4)
            .max("Name".len());
        let max_species_w = snapshots
            .iter()
            .map(|s| s.species_name.len())
            .max()
            .unwrap_or(7)
            .max("Species".len());
        let max_display_w = snapshots
            .iter()
            .map(|s| s.display_width)
            .max()
            .unwrap_or(7)
            .max("Display".len());
        let max_food_w = snapshots
            .iter()
            .map(|s| fields::format_weight(s.weight_g).len())
            .max()
            .unwrap_or(1)
            .max("Weight".len());

        let mut fixed_widths = vec![max_name_w, max_species_w, max_display_w, max_food_w];
        if show_tank_col {
            let max_tank_w = snapshots
                .iter()
                .filter_map(|s| s.tank_name.as_deref())
                .map(|tn| tn.len())
                .max()
                .unwrap_or(4)
                .max("Fishtank".len());
            fixed_widths.push(max_tank_w);
        }

        let animated_fish = fish_clones.first().cloned().map(display_clone);

        Self {
            selected: 0,
            scroll: 0,
            col_scroll: 1,
            snapshots,
            fish_clones,
            fixed_widths,
            show_tank_col,
            fantasy_cols,
            animated_fish,
        }
    }

    pub fn tick_animation(&mut self, dt: f32) {
        if let Some(ref mut fish) = self.animated_fish {
            fish.tick_animation(dt);
        }
    }

    pub fn scroll_up(&mut self) {
        let prev = self.selected;
        scroll_list::scroll_up(&mut self.selected, &mut self.scroll);
        if self.selected != prev {
            self.animated_fish = self
                .fish_clones
                .get(self.selected)
                .cloned()
                .map(display_clone);
        }
    }

    pub fn scroll_down(&mut self, available_lines: usize) {
        if self.selected + 1 >= self.snapshots.len() {
            return;
        }
        self.selected += 1;
        let mut lines = 0usize;
        let mut is_visible = false;
        for i in self.scroll..self.snapshots.len() {
            let h = self.snapshots[i].display_height as usize;
            if lines + h > available_lines {
                break;
            }
            lines += h;
            if i == self.selected {
                is_visible = true;
                break;
            }
        }
        if !is_visible {
            let sel_h = self.snapshots[self.selected].display_height as usize;
            if sel_h >= available_lines {
                self.scroll = self.selected;
            } else {
                let mut lines_used = sel_h;
                let mut new_scroll = self.selected;
                while new_scroll > 0 {
                    let h = self.snapshots[new_scroll - 1].display_height as usize;
                    if lines_used + h > available_lines {
                        break;
                    }
                    lines_used += h;
                    new_scroll -= 1;
                }
                self.scroll = new_scroll;
            }
        }
        self.animated_fish = self
            .fish_clones
            .get(self.selected)
            .cloned()
            .map(display_clone);
    }

    pub fn selected_fish_name(&self) -> Option<&str> {
        self.snapshots.get(self.selected).map(|s| s.name.as_str())
    }

    pub fn selected_tank_name(&self) -> &str {
        self.snapshots
            .get(self.selected)
            .and_then(|s| s.tank_name.as_deref())
            .unwrap_or("")
    }

    pub fn scroll_left(&mut self) {
        if self.col_scroll > 1 {
            self.col_scroll -= 1;
        }
    }

    pub fn scroll_right(&mut self, terminal_width: u16) {
        let widths = self.all_col_widths();
        let total_inner_w: usize = widths.iter().sum::<usize>() + widths.len().saturating_sub(1);
        let overlay_w = ((total_inner_w + 2) as u16).min(terminal_width);
        let inner_w = overlay_w.saturating_sub(2) as usize;
        let vis = self.visible_columns(inner_w);
        let last_vis = vis.last().copied().unwrap_or(0);
        if last_vis + 1 >= widths.len() {
            return;
        }
        let target = last_vis + 1;
        let saved = self.col_scroll;
        loop {
            self.col_scroll += 1;
            if self.col_scroll >= widths.len() {
                self.col_scroll = saved;
                break;
            }
            if self.visible_columns(inner_w).contains(&target) {
                break;
            }
        }
    }

    fn all_col_widths(&self) -> Vec<usize> {
        let mut w = self.fixed_widths.clone();
        for fc in &self.fantasy_cols {
            w.push(fc.col_width);
        }
        w
    }

    pub fn visible_fish_count(&self, available_lines: usize) -> usize {
        let mut lines_used = 0usize;
        let mut count = 0usize;
        for snap in self.snapshots.iter().skip(self.scroll) {
            let h = snap.display_height as usize;
            if lines_used + h > available_lines {
                break;
            }
            lines_used += h;
            count += 1;
        }
        count.max(1)
    }

    fn visible_columns(&self, inner_w: usize) -> Vec<usize> {
        let widths = self.all_col_widths();
        if widths.is_empty() {
            return vec![];
        }
        let name_w = widths[0];
        let mut vis = vec![0usize];
        let mut used = name_w;
        for (i, &w) in widths.iter().enumerate().skip(self.col_scroll.max(1)) {
            if used + 1 + w <= inner_w {
                vis.push(i);
                used += 1 + w;
            } else {
                break;
            }
        }
        vis
    }
}

pub struct IndexOverlay<'a> {
    state: &'a IndexState,
}

impl<'a> IndexOverlay<'a> {
    pub fn new(state: &'a IndexState) -> Self {
        Self { state }
    }
}

impl Widget for IndexOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let n = state.snapshots.len();

        let widths = state.all_col_widths();
        let total_inner_w: usize = widths.iter().sum::<usize>() + widths.len().saturating_sub(1);
        let overlay_w = ((total_inner_w + 2) as u16).min(area.width);
        let available_data_lines = area.height.saturating_sub(7) as usize;
        let vis_count = state.visible_fish_count(available_data_lines);
        let visible_data_lines: usize = state
            .snapshots
            .iter()
            .skip(state.scroll)
            .take(vis_count)
            .map(|s| s.display_height as usize)
            .sum::<usize>()
            .max(1);
        let overlay_h = ((visible_data_lines + 6) as u16).min(area.height);

        let ox = area.x + area.width.saturating_sub(overlay_w) / 2;
        let oy = area.y + area.height.saturating_sub(overlay_h) / 2;
        let rect = Rect::new(ox, oy, overlay_w, overlay_h);

        let bg = Color::Reset;
        for dy in 0..overlay_h {
            for dx in 0..overlay_w {
                let x = ox + dx;
                let y = oy + dy;
                if x < area.right() && y < area.bottom() {
                    buf[(x, y)].reset();
                    buf[(x, y)].set_bg(bg);
                }
            }
        }

        let inner_x = ox + 1;
        let inner_w = overlay_w.saturating_sub(2) as usize;
        let vis_cols = state.visible_columns(inner_w);
        let fixed_count = state.fixed_widths.len();
        let total_cols = fixed_count + state.fantasy_cols.len();
        let last_vis = vis_cols.last().copied().unwrap_or(0);
        let has_right_scroll = last_vis + 1 < total_cols;

        draw_border(buf, rect, bg, has_right_scroll);
        draw_header_row(buf, state, &vis_cols, inner_x, oy + 1, inner_w, bg);
        draw_separator(buf, rect, state, &vis_cols, oy + 2, bg, has_right_scroll);

        let data_start_y = oy + 3;
        let data_end_y = oy + overlay_h - 3;

        let mut y_cursor = data_start_y;
        let mut fish_idx = state.scroll;
        while y_cursor <= data_end_y && fish_idx < n {
            let row_h = state.snapshots[fish_idx].display_height;
            if y_cursor + row_h > data_end_y + 1 {
                break;
            }
            let selected = fish_idx == state.selected;
            draw_data_row(
                buf, state, &vis_cols, fish_idx, inner_x, y_cursor, inner_w, row_h, selected, bg,
                area,
            );
            y_cursor += row_h;
            fish_idx += 1;
        }

        let footer_y = oy + overlay_h - 2;
        let scrollable = n > vis_count || state.scroll > 0;
        let h_scrollable = has_right_scroll || state.col_scroll > 1;
        let col_hint = format!("({}/{})", last_vis + 1, total_cols);
        let left_hint = match (scrollable, h_scrollable) {
            (true, true) => format!(
                " ↑↓ scroll ({}/{})   ←→ cols {}",
                state.selected + 1,
                n,
                col_hint
            ),
            (true, false) => format!(" ↑↓ scroll ({}/{})", state.selected + 1, n),
            (false, true) => format!(" ↑↓ navigate   ←→ cols {}", col_hint),
            (false, false) => " ↑↓ navigate".to_string(),
        };
        let hint_style = Style::default().fg(Color::DarkGray).bg(bg);
        let right_text = "ESC/q close";
        let center_text = if n > 0 { Some("ENTER show") } else { None };
        let left_w = table::visual_width(&left_hint) as u16;
        let right_w = table::visual_width(right_text) as u16;
        buf.set_string(
            inner_x,
            footer_y,
            table::truncate_str(&left_hint, inner_w),
            hint_style,
        );
        if right_w + 2 <= inner_w as u16 {
            buf.set_string(
                inner_x + inner_w as u16 - right_w - 1,
                footer_y,
                right_text,
                hint_style,
            );
        }
        if let Some(ct) = center_text {
            let ct_w = table::visual_width(ct) as u16;
            let center_x = inner_x + (inner_w as u16 - ct_w) / 2;
            let right_edge = inner_x + inner_w as u16 - right_w - 2;
            if center_x > inner_x + left_w + 1 && center_x + ct_w < right_edge {
                buf.set_string(center_x, footer_y, ct, hint_style);
            }
        }
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect, bg: Color, has_right_scroll: bool) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.width;
    let h = rect.height;
    let right = x + w - 1;
    let bottom = y + h - 1;
    let border_style = Style::default().fg(Color::White).bg(bg);
    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);

    buf[(x, y)].set_char('┌').set_style(border_style);
    buf[(x, bottom)].set_char('└').set_style(border_style);
    for dy in 1..h - 1 {
        buf[(x, y + dy)].set_char('│').set_style(border_style);
    }

    if has_right_scroll {
        for dx in 1..w {
            buf[(x + dx, y)].set_char('─').set_style(border_style);
            buf[(x + dx, bottom)].set_char('─').set_style(border_style);
        }
    } else {
        buf[(right, y)].set_char('┐').set_style(border_style);
        buf[(right, bottom)].set_char('┘').set_style(border_style);
        for dx in 1..w - 1 {
            buf[(x + dx, y)].set_char('─').set_style(border_style);
            buf[(x + dx, bottom)].set_char('─').set_style(border_style);
        }
        for dy in 1..h - 1 {
            buf[(right, y + dy)].set_char('│').set_style(border_style);
        }
    }

    let title = " FishResource#index ";
    if (title.len() as u16 + 4) < w {
        buf.set_string(x + 2, y, title, title_style);
    }
}

fn draw_separator(
    buf: &mut Buffer,
    rect: Rect,
    state: &IndexState,
    vis_cols: &[usize],
    sep_y: u16,
    bg: Color,
    has_right_scroll: bool,
) {
    let x = rect.x;
    let right = rect.x + rect.width - 1;
    let border_style = Style::default().fg(Color::White).bg(bg);
    let sep_style = Style::default().fg(Color::White).bg(bg);

    buf[(x, sep_y)].set_char('├').set_style(border_style);
    if has_right_scroll {
        for dx in 1..rect.width {
            buf[(x + dx, sep_y)].set_char('─').set_style(sep_style);
        }
    } else {
        buf[(right, sep_y)].set_char('┤').set_style(border_style);
        for dx in 1..rect.width - 1 {
            buf[(x + dx, sep_y)].set_char('─').set_style(sep_style);
        }
    }

    let widths = state.all_col_widths();
    let mut cx = rect.x + 1;
    for (i, &col_idx) in vis_cols.iter().enumerate() {
        cx += widths[col_idx] as u16;
        if i < vis_cols.len() - 1 {
            buf[(cx, sep_y)].set_char('┼').set_style(border_style);
            cx += 1;
        }
    }
}

fn fixed_col_header(col_idx: usize) -> &'static str {
    match col_idx {
        0 => "Name",
        1 => "Species",
        2 => "Display",
        3 => "Weight",
        4 => "Fishtank",
        _ => "",
    }
}

fn draw_header_row(
    buf: &mut Buffer,
    state: &IndexState,
    vis_cols: &[usize],
    inner_x: u16,
    row_y: u16,
    inner_w: usize,
    bg: Color,
) {
    let hdr_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let sep_style = Style::default().fg(Color::White).bg(bg);
    let widths = state.all_col_widths();
    let fixed_count = state.fixed_widths.len();

    let mut x = inner_x;
    for (order, &col_idx) in vis_cols.iter().enumerate() {
        let w = widths[col_idx];
        let header: &str = if col_idx < fixed_count {
            fixed_col_header(col_idx)
        } else {
            state.fantasy_cols[col_idx - fixed_count].kind.header()
        };
        let padded = table::pad_right(header, w);
        let clipped = table::truncate_str(
            &padded,
            (inner_x + inner_w as u16).saturating_sub(x) as usize,
        );
        buf.set_string(x, row_y, &clipped, hdr_style);
        x += w as u16;
        if order < vis_cols.len() - 1 {
            if x < inner_x + inner_w as u16 {
                buf[(x, row_y)].set_char('│').set_style(sep_style);
            }
            x += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_data_row(
    buf: &mut Buffer,
    state: &IndexState,
    vis_cols: &[usize],
    fish_idx: usize,
    inner_x: u16,
    row_y: u16,
    inner_w: usize,
    row_h: u16,
    selected: bool,
    base_bg: Color,
    area: Rect,
) {
    let sel_bg = Color::Rgb(230, 228, 220);
    let row_bg = if selected { sel_bg } else { base_bg };
    let fg = if selected {
        Color::Black
    } else {
        Color::Rgb(180, 190, 210)
    };
    let sep_style = Style::default().fg(Color::White).bg(row_bg);
    let widths = state.all_col_widths();
    let snap = &state.snapshots[fish_idx];
    let right_x = inner_x + inner_w as u16;

    for dy in 0..row_h {
        for dx in 0..inner_w as u16 {
            if inner_x + dx < right_x && row_y + dy < area.bottom() {
                buf[(inner_x + dx, row_y + dy)].set_bg(row_bg);
            }
        }
    }

    let text_y = row_y + row_h / 2;

    let mut x = inner_x;
    for (order, &col_idx) in vis_cols.iter().enumerate() {
        let w = widths[col_idx];
        let avail = right_x.saturating_sub(x) as usize;

        let fixed_count = state.fixed_widths.len();
        match col_idx {
            0 => put_text(buf, &snap.name, x, text_y, w.min(avail), fg, row_bg),
            1 => put_text(buf, snap.species_name, x, text_y, w.min(avail), fg, row_bg),
            2 if snap.is_unfish && snap.display_height > 1 => {
                let disp_w = w.min(avail) as u16;
                for dy in 0..row_h {
                    for dx in 0..disp_w {
                        if x + dx < right_x && row_y + dy < area.bottom() {
                            buf[(x + dx, row_y + dy)].reset();
                        }
                    }
                }
                let f = if selected {
                    state.animated_fish.as_ref()
                } else {
                    None
                }
                .or_else(|| state.fish_clones.get(fish_idx));
                if let Some(f) = f {
                    tank_view::render_multi_row_unfish_at(f, x as i32, row_y as i32, area, buf);
                }
            }
            2 if selected => {
                let segs = state
                    .animated_fish
                    .as_ref()
                    .map(|f| f.segments())
                    .unwrap_or_else(|| snap.segments.clone());
                render_display_cell(buf, &segs, x, text_y, w.min(avail), base_bg);
            }
            2 => render_display_cell(buf, &snap.segments, x, text_y, w.min(avail), base_bg),
            3 => {
                let s = fields::format_weight(snap.weight_g);
                put_text(buf, &s, x, text_y, w.min(avail), fg, row_bg);
            }
            4 if state.show_tank_col => {
                let tn = snap.tank_name.as_deref().unwrap_or("");
                put_text(buf, tn, x, text_y, w.min(avail), fg, row_bg);
            }
            c => {
                let fc_idx = c - fixed_count;
                if fc_idx < state.fantasy_cols.len() {
                    let col = &state.fantasy_cols[fc_idx];
                    let cell = &col.cells[fish_idx];
                    if col.kind == FantasyKind::FavoriteColor {
                        let swatch_bg = cell.swatch.unwrap_or(Color::Black);
                        for dx in 0..w.min(avail) as u16 {
                            if x + dx < right_x {
                                buf[(x + dx, text_y)].set_char(' ').set_bg(swatch_bg);
                            }
                        }
                    } else {
                        put_text(buf, &cell.text, x, text_y, w.min(avail), fg, row_bg);
                    }
                }
            }
        }

        x += w as u16;
        if order < vis_cols.len() - 1 && x < right_x {
            for dy in 0..row_h {
                if row_y + dy < area.bottom() {
                    buf[(x, row_y + dy)].set_char('│').set_style(sep_style);
                }
            }
            x += 1;
        }
    }
}

fn render_display_cell(
    buf: &mut Buffer,
    segs: &[(char, Color)],
    x: u16,
    y: u16,
    col_width: usize,
    bg: Color,
) {
    for dx in 0..col_width as u16 {
        buf[(x + dx, y)].set_char(' ').set_bg(bg);
    }
    render_fish_segs(buf, segs, x, y, col_width as u16, bg);
}

fn put_text(buf: &mut Buffer, text: &str, x: u16, y: u16, width: usize, fg: Color, bg: Color) {
    let style = Style::default().fg(fg).bg(bg);
    let blank = " ".repeat(width);
    buf.set_string(x, y, &blank, style);
    let s = table::truncate_str(text, width);
    buf.set_string(x, y, &s, style);
}

fn generate_rut(rng: &mut impl RngExt) -> String {
    let body: u32 = rng.random_range(1_000_000..25_000_001);
    let s = body.to_string();
    let digits: Vec<u32> = s.chars().rev().map(|c| c as u32 - '0' as u32).collect();
    let multipliers = [2u32, 3, 4, 5, 6, 7];
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| d * multipliers[i % multipliers.len()])
        .sum();
    let rem = 11 - (sum % 11);
    let v = match rem {
        11 => '0',
        10 => 'K',
        n => char::from_digit(n, 10).unwrap_or('0'),
    };
    match s.len() {
        8 => format!("{}.{}.{}-{}", &s[..2], &s[2..5], &s[5..], v),
        _ => format!("{}.{}.{}-{}", &s[..1], &s[1..4], &s[4..], v),
    }
}

fn gen_fantasy(
    kind: FantasyKind,
    fish_names: &[String],
    species: &[FishSpecies],
    rng: &mut impl RngExt,
) -> Vec<FantasyCell> {
    let n = fish_names.len();
    match kind {
        FantasyKind::Iq => (0..n)
            .map(|_| {
                let v: i32 = match rng.random_range(0..100u32) {
                    0 => -30,
                    1 => 3000,
                    _ => rng.random_range(55..=145i32),
                };
                plain(v.to_string())
            })
            .collect(),

        FantasyKind::ZodiacSign => {
            const S: &[&str] = &[
                "Aries",
                "Taurus",
                "Gemini",
                "Cancer",
                "Leo",
                "Virgo",
                "Libra",
                "Scorpio",
                "Sagittarius",
                "Capricorn",
                "Aquarius",
                "Pisces",
            ];
            (0..n)
                .map(|_| plain(S[rng.random_range(0..S.len())]))
                .collect()
        }

        FantasyKind::ChineseZodiac => {
            const S: &[&str] = &[
                "鼠", "牛", "虎", "兔", "龍", "蛇", "馬", "羊", "猴", "雞", "狗", "豬",
            ];
            (0..n)
                .map(|_| plain(S[rng.random_range(0..S.len())]))
                .collect()
        }

        FantasyKind::Tanganana => (0..n)
            .map(|_| {
                plain(if rng.random::<bool>() {
                    "Tangananica"
                } else {
                    "Tanganana"
                })
            })
            .collect(),

        FantasyKind::HappinessLevel => (0..n)
            .map(|_| plain(format!("{}%", rng.random_range(0..=100u32))))
            .collect(),

        FantasyKind::Claustrophobic => (0..n)
            .map(|_| {
                plain(if rng.random_range(0..10u32) == 0 {
                    "Yes"
                } else {
                    "No"
                })
            })
            .collect(),

        FantasyKind::AttrText => (0..n).map(|_| plain("corge")).collect(),

        FantasyKind::AttrEnumBar => (0..n).map(|_| plain("")).collect(),

        FantasyKind::AttrBoolean => (0..n)
            .map(|i| plain(if i % 2 == 0 { "Sí" } else { "No" }))
            .collect(),

        FantasyKind::PickACard => {
            const RANKS: &[&str] = &[
                "Ace", "2", "3", "4", "5", "6", "7", "8", "9", "10", "Jack", "Queen", "King",
            ];
            const SUITS: &[&str] = &["Spades", "Hearts", "Diamonds", "Clubs"];
            (0..n)
                .map(|_| {
                    let roll = rng.random_range(0..54u32);
                    if roll >= 52 {
                        plain("Joker")
                    } else {
                        let rank = RANKS[(roll % 13) as usize];
                        let suit = SUITS[(roll / 13) as usize];
                        plain(format!("{} of {}", rank, suit))
                    }
                })
                .collect()
        }

        FantasyKind::FavoriteColor => {
            const COLORS: &[Color] = &[
                Color::Red,
                Color::Green,
                Color::Blue,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
                Color::LightRed,
                Color::LightGreen,
                Color::LightBlue,
                Color::LightYellow,
                Color::LightMagenta,
                Color::LightCyan,
                Color::Rgb(255, 100, 0),
                Color::Rgb(100, 200, 0),
                Color::Rgb(0, 180, 150),
                Color::Rgb(180, 0, 200),
                Color::Rgb(0, 140, 255),
            ];
            (0..n)
                .map(|i| {
                    let c = if species[i] == FishSpecies::Goldenfish {
                        Color::Rgb(255, 215, 0)
                    } else {
                        COLORS[rng.random_range(0..COLORS.len())]
                    };
                    FantasyCell {
                        text: "      ".to_string(),
                        swatch: Some(c),
                    }
                })
                .collect()
        }

        FantasyKind::FavoriteLetter => (0..n)
            .map(|_| plain(char::from(b'A' + rng.random_range(0..26u8)).to_string()))
            .collect(),

        FantasyKind::FavoriteNumber => (0..n)
            .map(|_| plain(rng.random::<i64>().to_string()))
            .collect(),

        FantasyKind::TarotPrediction => {
            const MUTANT_SPREADS: &[&str] = &[
                "The Tower + Death + Ten of Swords",
                "Three of Swords + The Devil + Nine of Swords",
                "Five of Pentacles + Ten of Wands + The Moon",
                "The Tower Reversed + Eight of Swords + The Hanged Man",
                "Death Reversed + Four of Pentacles + Judgement Reversed",
                "The Devil Reversed + Seven of Swords + Wheel of Fortune Reversed",
                "Ten of Swords Reversed + The Moon Reversed + Five of Cups",
                "Five of Cups + Hermit Reversed + Lovers Reversed",
                "Justice Reversed + The Tower + King of Pentacles Reversed",
                "Sun Reversed + Star Reversed + Nine of Wands",
            ];
            const GOLDEN_SPREADS: &[&str] = &[
                "The Sun + Ten of Cups + Ace of Pentacles",
                "The Star + Lovers + The World",
                "Wheel of Fortune + Six of Wands + The Emperor",
                "Ace of Cups + The Empress + Four of Wands",
                "The Magician + The Chariot + The Sun",
                "Death + The Star + Ace of Wands",
                "The Devil Reversed + Judgement + The Fool",
                "Nine of Pentacles + King of Pentacles + The World",
                "Two of Cups + Ten of Cups + Star Reversed",
                "Strength + The Hierophant + Sun Reversed",
            ];
            const CARDS: &[&str] = &[
                "The Fool",
                "The Magician",
                "The High Priestess",
                "The Empress",
                "The Emperor",
                "The Hierophant",
                "The Lovers",
                "The Chariot",
                "Strength",
                "The Hermit",
                "Wheel of Fortune",
                "Justice",
                "The Hanged Man",
                "Death",
                "Temperance",
                "The Devil",
                "The Tower",
                "The Star",
                "The Moon",
                "The Sun",
                "Judgement",
                "The World",
            ];
            (0..n)
                .map(|i| match species[i] {
                    FishSpecies::Mutantfish => {
                        plain(MUTANT_SPREADS[rng.random_range(0..MUTANT_SPREADS.len())])
                    }
                    FishSpecies::Goldenfish => {
                        plain(GOLDEN_SPREADS[rng.random_range(0..GOLDEN_SPREADS.len())])
                    }
                    _ => {
                        let mut deck: Vec<&str> = CARDS.to_vec();
                        let i1 = rng.random_range(0..deck.len());
                        let c1 = deck.remove(i1);
                        let i2 = rng.random_range(0..deck.len());
                        let c2 = deck.remove(i2);
                        let i3 = rng.random_range(0..deck.len());
                        let c3 = deck.remove(i3);
                        let r1 = if rng.random::<bool>() { " (R)" } else { "" };
                        let r2 = if rng.random::<bool>() { " (R)" } else { "" };
                        let r3 = if rng.random::<bool>() { " (R)" } else { "" };
                        plain(format!("{}{}, {}{}, {}{}", c1, r1, c2, r2, c3, r3))
                    }
                })
                .collect()
        }

        FantasyKind::FavoriteQuote => (0..n)
            .map(|i| match species[i] {
                FishSpecies::Mutantfish => plain("OOGHHHHHHH"),
                FishSpecies::Goldenfish => plain("Gonna be, gonna be golden"),
                _ => {
                    let count = rng.random_range(2..=8u32);
                    plain((0..count).map(|_| "glub").collect::<Vec<_>>().join(" "))
                }
            })
            .collect(),

        FantasyKind::FavoriteTime => (0..n)
            .map(|_| {
                plain(format!(
                    "{:02}:{:02}",
                    rng.random_range(0..24u32),
                    rng.random_range(0..60u32)
                ))
            })
            .collect(),

        FantasyKind::Lonely => (0..n)
            .map(|_| {
                plain(if rng.random_range(0..10u32) == 0 {
                    "Yes"
                } else {
                    "No"
                })
            })
            .collect(),

        FantasyKind::Gmi => (0..n)
            .map(|_| plain(if rng.random::<bool>() { "gmi" } else { "ngmi" }))
            .collect(),

        FantasyKind::ElOLaPr => (0..n).map(|_| plain("La PR")).collect(),

        FantasyKind::TheOrThePr => (0..n).map(|_| plain("The PR")).collect(),

        FantasyKind::Crush => (0..n)
            .map(|_| plain(fish_names[rng.random_range(0..fish_names.len())].clone()))
            .collect(),

        FantasyKind::MarriedTo => (0..n)
            .map(|_| plain(fish_names[rng.random_range(0..fish_names.len())].clone()))
            .collect(),

        FantasyKind::Hates => {
            let target = rng.random_range(0..n);
            (0..n)
                .map(|i| {
                    if i == target {
                        plain("No one")
                    } else {
                        plain(fish_names[target].clone())
                    }
                })
                .collect()
        }

        FantasyKind::Status => {
            const S: &[&str] = &[
                "Swimming",
                "Pondering",
                "Breathing",
                "Prompting",
                "Prooompting",
                "Fishing",
                "Dreaming",
                "Feeling",
                "Happy",
                "Sad",
                "Nauseous",
                "Kicking Rocks",
                "Giving the Time",
                "Taking out the turn",
                "Falling",
                "Floating",
            ];
            (0..n)
                .map(|_| plain(S[rng.random_range(0..S.len())]))
                .collect()
        }

        FantasyKind::Sin => {
            const SINS: &[&str] = &[
                "Lust", "Gluttony", "Greed", "Sloth", "Wrath", "Envy", "Pride",
            ];
            (0..n)
                .map(|i| match species[i] {
                    FishSpecies::Mutantfish => plain("[REDACTED]"),
                    FishSpecies::Goldenfish => plain(""),
                    _ => plain(SINS[rng.random_range(0..SINS.len())]),
                })
                .collect()
        }

        FantasyKind::HasSeenTheSky => (0..n)
            .map(|i| {
                plain(if species[i] == FishSpecies::Goldenfish {
                    "Yes"
                } else {
                    "No"
                })
            })
            .collect(),

        FantasyKind::Temperature => (0..n)
            .map(|_| {
                let t = 25.0f32 + rng.random_range(-14.0f32..14.0);
                plain(format!("{:.1}°C", t))
            })
            .collect(),

        FantasyKind::Delicious => (0..n)
            .map(|i| match species[i] {
                FishSpecies::Mutantfish => plain("NOOOOOOOOOO"),
                FishSpecies::Goldenfish => plain("Yes."),
                _ => plain(match rng.random_range(0..3u32) {
                    0 => "Yes",
                    1 => "No",
                    _ => "Maybe",
                }),
            })
            .collect(),

        FantasyKind::Region => {
            const R: &[&str] = &[
                "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "RM",
                "XIV", "XV",
            ];
            (0..n)
                .map(|_| plain(R[rng.random_range(0..R.len())]))
                .collect()
        }

        FantasyKind::Dni => (0..n).map(|_| plain(generate_rut(rng))).collect(),

        FantasyKind::FavoriteSeason => {
            const S: &[&str] = &["Winter", "Autumn", "Spring", "Summer"];
            (0..n)
                .map(|_| plain(S[rng.random_range(0..S.len())]))
                .collect()
        }
    }
}

fn display_clone(mut fish: Fish) -> Fish {
    fish.facing = Direction::Left;
    fish
}

fn plain(s: impl Into<String>) -> FantasyCell {
    FantasyCell {
        text: s.into(),
        swatch: None,
    }
}
