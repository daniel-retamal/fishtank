use std::cell::Cell;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    BROWN, INDIGO, LIGHT_BLUE, LIGHT_CYAN, LIGHT_GREEN, LIGHT_MAGENTA, LIGHT_RED, LIGHT_YELLOW,
    MAGENTA, ORANGE, PINK, PURPLE, PURPLE_DARK, PURPLE_LIGHT, VIOLET, WHITE,
};

use crate::sprite::{TRANSPARENT, mirror_grid, opaque_line};

const LOLLIPOP_HEAD: &[&str] = &[
    "    _....._   ",
    "  .' _..._ '. ",
    r#" / /`  __ `\ \"#,
    r#"; ;  /`  \  | ;"#,
    "| | |  (_/  ; |",
    r#"; ;  \_.._.' .;"#,
    r#" \ '. ,_,. .'/  "#,
    "  '._ ,_, _.'  ",
];
const LOLLIPOP_HEAD_ROWS: usize = 8;
const LOLLIPOP_WIDTH: i32 = 15;
const LOLLIPOP_STICK_COL: usize = 7;

const KOYAK_HEAD: &[&str] = &[r#"  ,-"-. "#, " :=====:", " :=====;", "  `-,-' "];
const KOYAK_HEAD_ROWS: usize = 4;
const KOYAK_WIDTH: i32 = 8;
const KOYAK_STICK_COL: usize = 4;

const CANE_TOP: &[&str] = &[
    "   ___   ",
    " .' | '.",
    r#"/\./"\,/|"#,
    "|_|   | |",
    "|_|   |/|",
];
const CANE_MIDDLE_A: &str = "      | |";
const CANE_MIDDLE_B: &str = "      |/|";
const CANE_BOTTOM: &str = "      |_|";
const CANE_TOP_ROWS: usize = 5;
const CANE_WIDTH: i32 = 9;
const CANE_POLE_LEFT_COL: i32 = 6;
const CANE_POLE_RIGHT_LOCAL: i32 = 2;
const CANE_POLE_RIGHT_DROP: i32 = 1;
const CANE_STRIPE_BAND: i32 = 3;
const CANE_STRIPE_FULL_PERIOD: usize = 6;

pub const HOUSE_LINES: &[&str] = &[
    "               [ ]             ",
    "       ________[_]________     ",
    r#"      /\  /   / __L___/ O \    "#,
    r#"     //O\/ O O  \    /\  / \   "#,
    r#"    //O__\  / O /\__/  \/   \  "#,
    r#"   //O____\/   /  \ |[]| O / \ "#,
    r#"  //O______\  /   /\|__|  /   \"#,
    r#" //O________\/   O   /   / O O \"#,
    r#"/_I________I_\__L___L___L___L___\"#,
    "  I   __   I    O[]_|_[]O    I  ",
    "  I O|  |O I    O[]_|_[]O    I  ",
    "  I_O|  |O_I_________________I  ",
];
pub const HOUSE_ROWS: usize = 12;
pub const HOUSE_WIDTH: i32 = 33;

const HOUSE_WHITE_MASK: &[&str] = &[
    "       ________   ________",
    r#"      /\  /   /   L   /   \"#,
    r#"     /  \/               / \"#,
    r#"    /    \  /   /       /   \"#,
    r#"   /      \/   /           / \"#,
    r#"  /        \  /   /       /   \"#,
    r#" /          \/       /   /     \"#,
    r#"/            \__L___L___L___L___\"#,
];

pub const HOUSE2_LINES: &[&str] = &[
    r#"                      [ ]        "#,
    r#"    __________________[_]____    "#,
    r#"   /^^^O^^,-.^^O^^^^O^^^^^^O^\   "#,
    r#"  / O   ,',-.`. O O      O   O\  "#,
    r#" /O^^^,','   `.`.^^^O^,-----.^^\ "#,
    r#"/___,','__   __`.`.__/_,"T"._\__\"#,
    r#" | '-'||  |O|  ||`-`O[_|_|_|_]O| "#,
    r#" | O  ||__|O|__|| O O[_|_|_|_]O| "#,
    r#" |    |         |   ,____,  _  | "#,
    r#" |  O | __   __ | O |    |O| |O| "#,
    r#" |    ||  |O|  ||   |    |O|_|O| "#,
    r#" | O  ||__|O|__|| O |    |     | "#,
    r#" |____|_________|___|    |_____| "#,
];
pub const HOUSE2_ROWS: usize = 13;
pub const HOUSE2_WIDTH: i32 = 33;

const HOUSE2_WHITE_MASK: &[&str] = &[
    r#"    __________________ _ ____"#,
    r#"   /^^^ ^^,-.^^ ^^^^ ^^^^^^ ^\"#,
    r#"  /     ,',-.`.               \"#,
    r#" / ^^^,','   `.`.^^^ ^,-----.^^\"#,
    r#"/___,','       `.`.__/       \__\"#,
    r#"   '-'           `-`"#,
];

const HOUSE2_GREEN_MASK: &[&str] = &[
    "",
    "",
    "",
    "",
    "",
    r#"        __   __       _,"T"._"#,
    r#"       |  | |  |     [_|_|_|_]"#,
    r#"       |__| |__|     [_|_|_|_]"#,
    r#"                    ,____,  _"#,
    r#"        __   __     |    | | |"#,
    r#"       |  | |  |    |    | |_|"#,
    r#"       |__| |__|    |    |"#,
    r#"                    |    |"#,
];

const MAN_LINES: &[&str] = &[
    "    .-.    ",
    "  _( \" )_  ",
    " (_  :  _) ",
    "   / ' \\   ",
    "  (_/^\\_)  ",
];
const MAN_WIDTH: i32 = 11;
const MAN_SWAY_ROWS: usize = 3;
const MAN_SWAY_STEP_TICKS: u32 = 12;
const MAN_SWAY_SEQUENCE: &[i32] = &[0, -1, 0, 1];

const CANDY_HEIGHT_MIN: usize = 8;
const CANDY_HEIGHT_MAX: usize = 22;
const LOLLIPOP_HEIGHT_MAX: usize = 40;
const CANDY_GAP_MIN: i32 = 3;
const CANDY_GAP_MAX: i32 = 6;
const CANDY_LEFT_MIN: i32 = 1;
const CANDY_LEFT_MAX: i32 = 8;

const HOUSE_GAP_MIN: i32 = 55;
const HOUSE_GAP_MAX: i32 = 90;
const DECO_GAP_MIN: i32 = 4;
const DECO_GAP_MAX: i32 = 8;
const DECO_LEFT_MIN: i32 = 5;
const DECO_LEFT_MAX: i32 = 15;

const CANDY_PINKS: &[Color] = &[
    PINK,
    MAGENTA,
    LIGHT_MAGENTA,
    PURPLE_DARK,
    PURPLE,
    PURPLE_LIGHT,
    VIOLET,
    INDIGO,
];
const HOUSE_ACCENT_COLORS: &[Color] = &[
    LIGHT_YELLOW,
    LIGHT_CYAN,
    LIGHT_BLUE,
    LIGHT_MAGENTA,
    LIGHT_RED,
    LIGHT_GREEN,
    ORANGE,
];

fn random_candy_color(rng: &mut impl RngExt) -> Color {
    let total = CANDY_PINKS.len() + HOUSE_ACCENT_COLORS.len();
    let idx = rng.random_range(0..total);
    if idx < CANDY_PINKS.len() {
        CANDY_PINKS[idx]
    } else {
        HOUSE_ACCENT_COLORS[idx - CANDY_PINKS.len()]
    }
}

pub fn man_sway_offset(tick: u32) -> i32 {
    let step = (tick / MAN_SWAY_STEP_TICKS) % MAN_SWAY_SEQUENCE.len() as u32;
    MAN_SWAY_SEQUENCE[step as usize]
}

#[derive(Clone, Copy)]
pub enum CandyVariant {
    Lollipop,
    Koyak,
    CandyCane,
}

impl CandyVariant {
    fn random(rng: &mut impl RngExt) -> Self {
        match rng.random_range(0..3u8) {
            0 => CandyVariant::Lollipop,
            1 => CandyVariant::Koyak,
            _ => CandyVariant::CandyCane,
        }
    }

    fn sprite_width(self) -> i32 {
        match self {
            CandyVariant::Lollipop => LOLLIPOP_WIDTH,
            CandyVariant::Koyak => KOYAK_WIDTH,
            CandyVariant::CandyCane => CANE_WIDTH,
        }
    }
}

pub struct CandyPlant {
    pub x: i32,
    pub height: usize,
    pub variant: CandyVariant,
    pub color_a: Color,
}

impl CandyPlant {
    fn new(x: i32, variant: CandyVariant, rng: &mut impl RngExt) -> Self {
        let height_max = match variant {
            CandyVariant::Lollipop => LOLLIPOP_HEIGHT_MAX,
            _ => CANDY_HEIGHT_MAX,
        };
        let min_h = match variant {
            CandyVariant::Lollipop => LOLLIPOP_HEAD_ROWS + 2,
            CandyVariant::Koyak => KOYAK_HEAD_ROWS + 2,
            CandyVariant::CandyCane => CANE_TOP_ROWS + 2,
        };
        let height = rng.random_range(CANDY_HEIGHT_MIN..=height_max).max(min_h);
        let color_a = random_candy_color(rng);
        Self {
            x,
            height,
            variant,
            color_a,
        }
    }

    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        match self.variant {
            CandyVariant::Lollipop => lollipop_rows(self),
            CandyVariant::Koyak => koyak_rows(self),
            CandyVariant::CandyCane => candy_cane_rows(self),
        }
    }

    pub fn sprite_width(&self) -> i32 {
        self.variant.sprite_width()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GingerbreadVariant {
    House,
    House2,
    Man,
}

pub struct GingerbreadDecoration {
    pub x: i32,
    pub variant: GingerbreadVariant,
    pub mirrored: bool,
    pub accent: Color,
    pub o_colors: Vec<Color>,
}

impl GingerbreadDecoration {
    fn new_house(x: i32, rng: &mut impl RngExt) -> Self {
        let (variant, lines) = if rng.random::<bool>() {
            (GingerbreadVariant::House, HOUSE_LINES)
        } else {
            (GingerbreadVariant::House2, HOUSE2_LINES)
        };
        let o_count = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == 'O').count())
            .sum();
        let o_colors = (0..o_count)
            .map(|_| HOUSE_ACCENT_COLORS[rng.random_range(0..HOUSE_ACCENT_COLORS.len())])
            .collect();
        Self {
            x,
            variant,
            mirrored: rng.random::<bool>(),
            accent: PINK,
            o_colors,
        }
    }

    fn new_man(x: i32, rng: &mut impl RngExt) -> Self {
        let man_accents = [LIGHT_GREEN, LIGHT_RED];
        let accent = man_accents[rng.random_range(0..2)];
        Self {
            x,
            variant: GingerbreadVariant::Man,
            mirrored: false,
            accent,
            o_colors: Vec::new(),
        }
    }

    pub fn rows(&self, sway_offset: i32) -> Vec<Vec<(char, Color)>> {
        match self.variant {
            GingerbreadVariant::House => self.maybe_mirror(house_rows(&self.o_colors)),
            GingerbreadVariant::House2 => self.maybe_mirror(house2_rows(&self.o_colors)),
            GingerbreadVariant::Man => man_rows(self.accent, sway_offset),
        }
    }

    fn maybe_mirror(&self, grid: Vec<Vec<(char, Color)>>) -> Vec<Vec<(char, Color)>> {
        if self.mirrored {
            mirror_grid(&grid)
        } else {
            grid
        }
    }

    pub fn sprite_width(&self) -> i32 {
        match self.variant {
            GingerbreadVariant::House => HOUSE_WIDTH,
            GingerbreadVariant::House2 => HOUSE2_WIDTH,
            GingerbreadVariant::Man => MAN_WIDTH,
        }
    }
}

pub struct CandyBackground {
    pub plants: Vec<CandyPlant>,
    pub decos: Vec<GingerbreadDecoration>,
}

impl Default for CandyBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl CandyBackground {
    pub fn new() -> Self {
        Self {
            plants: Vec::new(),
            decos: Vec::new(),
        }
    }
}

pub fn extend_candy_plants(plants: &mut Vec<CandyPlant>, to_width: i32, rng: &mut impl RngExt) {
    let frontier = plants.iter().map(|p| p.x + p.sprite_width()).max();
    let mut next_x = match frontier {
        Some(edge) => edge + rng.random_range(CANDY_GAP_MIN..=CANDY_GAP_MAX),
        None => rng.random_range(CANDY_LEFT_MIN..=CANDY_LEFT_MAX),
    };
    while next_x < to_width {
        let variant = CandyVariant::random(rng);
        let plant = CandyPlant::new(next_x, variant, rng);
        let effective_w = (plant.sprite_width() - 2).max(1);
        let gap = rng.random_range(CANDY_GAP_MIN..=CANDY_GAP_MAX);
        next_x = plant.x + effective_w + gap;
        plants.push(plant);
    }
}

pub fn extend_candy_decos(
    decos: &mut Vec<GingerbreadDecoration>,
    to_width: i32,
    rng: &mut impl RngExt,
) {
    let frontier = decos.iter().map(|d| d.x + d.sprite_width()).max();
    let mut next_house_x = match frontier {
        Some(edge) => edge + rng.random_range(HOUSE_GAP_MIN..=HOUSE_GAP_MAX),
        None => rng.random_range(DECO_LEFT_MIN..=DECO_LEFT_MAX),
    };
    while next_house_x < to_width {
        let house = GingerbreadDecoration::new_house(next_house_x, rng);
        let house_end = house.x + house.sprite_width();
        decos.push(house);
        let next_house = house_end + rng.random_range(HOUSE_GAP_MIN..=HOUSE_GAP_MAX);
        let mut sx = house_end + rng.random_range(DECO_GAP_MIN..=DECO_GAP_MAX);
        loop {
            let deco = GingerbreadDecoration::new_man(sx, rng);
            let deco_end = deco.x + deco.sprite_width();
            if deco_end + DECO_GAP_MIN > next_house {
                break;
            }
            sx = deco_end + rng.random_range(DECO_GAP_MIN..=DECO_GAP_MAX);
            decos.push(deco);
        }
        next_house_x = next_house;
    }
}

fn stick_row(width: usize, stick_col: usize) -> Vec<(char, Color)> {
    (0..width)
        .map(|i| {
            if i == stick_col {
                ('|', WHITE)
            } else {
                (TRANSPARENT, WHITE)
            }
        })
        .collect()
}

fn lollipop_rows(plant: &CandyPlant) -> Vec<Vec<(char, Color)>> {
    let stick_count = plant.height.saturating_sub(LOLLIPOP_HEAD_ROWS).max(1);
    let mut rows: Vec<Vec<(char, Color)>> = LOLLIPOP_HEAD
        .iter()
        .map(|&line| opaque_line(line, Color::Reset, |_, _| plant.color_a))
        .collect();
    for _ in 0..stick_count {
        rows.push(stick_row(LOLLIPOP_WIDTH as usize, LOLLIPOP_STICK_COL));
    }
    rows
}

fn koyak_rows(plant: &CandyPlant) -> Vec<Vec<(char, Color)>> {
    let stick_count = plant.height.saturating_sub(KOYAK_HEAD_ROWS).max(1);
    let mut rows: Vec<Vec<(char, Color)>> = KOYAK_HEAD
        .iter()
        .map(|&line| opaque_line(line, Color::Reset, |_, _| plant.color_a))
        .collect();
    for _ in 0..stick_count {
        rows.push(stick_row(KOYAK_WIDTH as usize, KOYAK_STICK_COL));
    }
    rows
}

fn cane_hook_color(row: usize, col: usize, pink: Color) -> Option<Color> {
    match (row, col) {
        (0, 4) | (0, 5) => Some(pink),
        (0, 3) => Some(WHITE),
        (1, 4) | (1, 6) | (1, 7) => Some(pink),
        (1, 1) | (1, 2) => Some(WHITE),
        (2, 0) | (2, 4) | (2, 5) | (2, 6) => Some(pink),
        (2, 1) | (2, 2) | (2, 3) => Some(WHITE),
        (3, 0) | (3, 1) | (3, 2) => Some(pink),
        (4, 0) | (4, 1) | (4, 2) => Some(WHITE),
        _ => None,
    }
}

fn cane_pole_color(rows_from_bottom: i32, col: usize, pink: Color) -> Color {
    let local = col as i32 - CANE_POLE_LEFT_COL;
    let adjusted_rfb = if local == CANE_POLE_RIGHT_LOCAL {
        rows_from_bottom + CANE_POLE_RIGHT_DROP
    } else {
        rows_from_bottom
    };
    let diagonal = adjusted_rfb - local;
    let band = (diagonal + CANE_STRIPE_BAND - 1).div_euclid(CANE_STRIPE_BAND);
    if band.rem_euclid(2) == 0 { pink } else { WHITE }
}

fn cane_effective_height(height: usize) -> usize {
    let min = CANE_TOP_ROWS + 2;
    let h = height.max(min);
    h.div_ceil(CANE_STRIPE_FULL_PERIOD) * CANE_STRIPE_FULL_PERIOD
}

fn candy_cane_rows(plant: &CandyPlant) -> Vec<Vec<(char, Color)>> {
    let pink = plant.color_a;
    let height = cane_effective_height(plant.height);
    let middle_count = height - CANE_TOP_ROWS - 1;
    let mut glyph_rows: Vec<&str> = CANE_TOP.to_vec();
    for j in 0..middle_count {
        let rfb = middle_count - j;
        glyph_rows.push(if rfb % 3 == 1 {
            CANE_MIDDLE_B
        } else {
            CANE_MIDDLE_A
        });
    }
    glyph_rows.push(CANE_BOTTOM);
    let total = glyph_rows.len();
    glyph_rows
        .iter()
        .enumerate()
        .map(|(i, &line)| {
            let rfb = (total - 1 - i) as i32;
            opaque_line(line, Color::Reset, |_, col| {
                if i < CANE_TOP_ROWS
                    && let Some(color) = cane_hook_color(i, col, pink)
                {
                    return color;
                }
                cane_pole_color(rfb, col, pink)
            })
        })
        .collect()
}

fn house_white_at(row: usize, col: usize) -> bool {
    if row < 1 || row > HOUSE_WHITE_MASK.len() {
        return false;
    }
    HOUSE_WHITE_MASK[row - 1]
        .chars()
        .nth(col)
        .is_some_and(|c| c != ' ')
}

fn house_window_cell(row: usize, col: usize) -> bool {
    (row == 9 || row == 10) && (17..=23).contains(&col)
}

fn house_door_cell(row: usize, col: usize) -> bool {
    matches!(
        (row, col),
        (9, 6) | (9, 7) | (10, 5) | (10, 8) | (11, 5) | (11, 8)
    )
}

fn house_char_color(
    ch: char,
    row: usize,
    col: usize,
    o_idx: &Cell<usize>,
    o_colors: &[Color],
) -> Color {
    if ch == 'O' {
        let i = o_idx.get();
        o_idx.set(i + 1);
        return o_colors.get(i).copied().unwrap_or(LIGHT_RED);
    }
    if ch == 'I' {
        return BROWN;
    }
    if house_white_at(row, col) {
        return WHITE;
    }
    if row == 5 && (ch == '[' || ch == ']') {
        return LIGHT_GREEN;
    }
    if house_window_cell(row, col) {
        return LIGHT_GREEN;
    }
    if house_door_cell(row, col) {
        return LIGHT_GREEN;
    }
    BROWN
}

fn mask_marks(mask: &[&str], offset: usize, row: usize, col: usize) -> bool {
    if row < offset {
        return false;
    }
    mask.get(row - offset)
        .and_then(|line| line.chars().nth(col))
        .is_some_and(|c| c != ' ')
}

fn house2_char_color(
    ch: char,
    row: usize,
    col: usize,
    o_idx: &Cell<usize>,
    o_colors: &[Color],
) -> Color {
    if ch == 'O' {
        let i = o_idx.get();
        o_idx.set(i + 1);
        return o_colors.get(i).copied().unwrap_or(LIGHT_RED);
    }
    if mask_marks(HOUSE2_WHITE_MASK, 1, row, col) {
        return WHITE;
    }
    if mask_marks(HOUSE2_GREEN_MASK, 0, row, col) {
        return LIGHT_GREEN;
    }
    BROWN
}

fn build_house_rows(
    lines: &[&str],
    o_colors: &[Color],
    color_fn: impl Fn(char, usize, usize, &Cell<usize>, &[Color]) -> Color,
) -> Vec<Vec<(char, Color)>> {
    let o_idx = Cell::new(0usize);
    lines
        .iter()
        .enumerate()
        .map(|(row_idx, &line)| {
            opaque_line(line, Color::Reset, |c, col| {
                color_fn(c, row_idx, col, &o_idx, o_colors)
            })
        })
        .collect()
}

fn house_rows(o_colors: &[Color]) -> Vec<Vec<(char, Color)>> {
    build_house_rows(HOUSE_LINES, o_colors, house_char_color)
}

fn house2_rows(o_colors: &[Color]) -> Vec<Vec<(char, Color)>> {
    build_house_rows(HOUSE2_LINES, o_colors, house2_char_color)
}

fn shift_row(row: &[(char, Color)], offset: i32) -> Vec<(char, Color)> {
    let len = row.len() as i32;
    (0..len)
        .map(|i| {
            let src = i - offset;
            if (0..len).contains(&src) {
                row[src as usize]
            } else {
                (TRANSPARENT, Color::Reset)
            }
        })
        .collect()
}

fn man_rows(accent: Color, sway_offset: i32) -> Vec<Vec<(char, Color)>> {
    MAN_LINES
        .iter()
        .enumerate()
        .map(|(row_idx, &line)| {
            let row = opaque_line(line, Color::Reset, |c, _| match c {
                '"' => WHITE,
                ':' | '\'' => accent,
                _ => BROWN,
            });
            if row_idx < MAN_SWAY_ROWS {
                shift_row(&row, sway_offset)
            } else {
                row
            }
        })
        .collect()
}
