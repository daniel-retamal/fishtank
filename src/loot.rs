use rand::RngExt;
use ratatui::style::Color;

use crate::entities::species::FishSpecies;

const JUNK_FILLER_CHARS: &[char] = &['&', '@', '€', '%', '$', '#', 'X', '<', '>'];
const JUNK_COLORS: &[Color] = &[
    Color::Gray,
    Color::Rgb(160, 100, 40),
    Color::Green,
    Color::LightGreen,
];

pub struct JunkSprite {
    pub rows: Vec<Vec<(char, Color)>>,
}

fn junk_color(rng: &mut impl RngExt) -> Color {
    JUNK_COLORS[rng.random_range(0..JUNK_COLORS.len())]
}

fn junk_char(rng: &mut impl RngExt) -> char {
    JUNK_FILLER_CHARS[rng.random_range(0..JUNK_FILLER_CHARS.len())]
}

impl JunkSprite {
    pub fn new(rng: &mut impl RngExt) -> Self {
        let dark = Color::DarkGray;

        let row0 = vec![
            (' ', dark),
            (' ', dark),
            (' ', dark),
            ('.', dark),
            ('_', dark),
            ('_', dark),
            ('_', dark),
            ('.', dark),
        ];

        let row1 = vec![
            (' ', dark),
            (' ', dark),
            ('(', dark),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (')', dark),
            ('.', dark),
        ];

        let row2 = vec![
            ('.', dark),
            ('(', dark),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (junk_char(rng), junk_color(rng)),
            (')', dark),
        ];

        JunkSprite {
            rows: vec![row0, row1, row2],
        }
    }

    pub fn sprite_height() -> u16 {
        3
    }
    pub fn hook_col() -> u16 {
        9
    }
    pub fn hook_row() -> u16 {
        1
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsumableKind {
    Coffee,
    Bait,
}

impl ConsumableKind {
    pub fn panel_inner_w(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 8,
            ConsumableKind::Bait => 9,
        }
    }

    pub fn hook_col(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 4,
            ConsumableKind::Bait => 3,
        }
    }

    pub fn hook_row(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 2,
            ConsumableKind::Bait => 0,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => "Coffee",
            ConsumableKind::Bait => "Bait",
        }
    }
}

pub fn coffee_sprite_rows(anim_phase: bool) -> Vec<Vec<(char, Color)>> {
    let steam = Color::Rgb(200, 200, 200);
    let cup = Color::Rgb(180, 180, 180);
    let liquid = Color::Rgb(140, 80, 20);
    let label = Color::White;

    let (top_open, top_close, bot_open, bot_close) = if anim_phase {
        (')', ')', '(', '(')
    } else {
        ('(', '(', ')', ')')
    };

    vec![
        vec![
            (' ', steam),
            (' ', steam),
            (top_open, steam),
            (top_close, steam),
        ],
        vec![
            (' ', steam),
            (' ', steam),
            (bot_open, steam),
            (bot_close, steam),
        ],
        vec![
            (' ', cup),
            ('|', cup),
            ('~', liquid),
            ('~', liquid),
            ('|', cup),
        ],
        vec![('C', label), ('|', cup), ('_', cup), ('_', cup), ('|', cup)],
    ]
}

pub fn bait_sprite_rows() -> Vec<Vec<(char, Color)>> {
    let dirt = Color::Rgb(120, 80, 40);
    let body = Color::Rgb(220, 100, 80);
    let eye = Color::Rgb(200, 160, 120);

    vec![
        vec![(' ', dirt), (' ', dirt), (' ', dirt), ('_', dirt)],
        vec![
            (' ', body),
            (' ', body),
            ('(', body),
            ('º', eye),
            ('\\', body),
        ],
        vec![
            (' ', body),
            ('_', body),
            ('_', body),
            (')', body),
            (' ', body),
            (')', body),
        ],
        vec![
            ('(', body),
            ('_', body),
            ('_', body),
            ('_', body),
            ('/', body),
        ],
    ]
}

#[derive(Clone, Copy)]
pub enum CashValue {
    One,
    Two,
    Five,
    Ten,
    Twenty,
    Hundred,
    Thousand,
}

const CASH_TABLE: &[(u32, CashValue)] = &[
    (188, CashValue::One),
    (188, CashValue::Two),
    (188, CashValue::Five),
    (188, CashValue::Ten),
    (188, CashValue::Twenty),
    (25, CashValue::Hundred),
    (5, CashValue::Thousand),
];

impl CashValue {
    pub fn amount(self) -> u32 {
        match self {
            CashValue::One => 1,
            CashValue::Two => 2,
            CashValue::Five => 5,
            CashValue::Ten => 10,
            CashValue::Twenty => 20,
            CashValue::Hundred => 100,
            CashValue::Thousand => 1000,
        }
    }

    pub fn color(self) -> Color {
        match self {
            CashValue::One => Color::Green,
            CashValue::Two => Color::Rgb(140, 0, 210),
            CashValue::Five => Color::LightMagenta,
            CashValue::Ten => Color::Blue,
            CashValue::Twenty => Color::Rgb(255, 130, 0),
            CashValue::Hundred => Color::Red,
            CashValue::Thousand => Color::Yellow,
        }
    }

    pub fn roll(rng: &mut impl RngExt) -> Self {
        roll_weighted(CASH_TABLE, rng)
    }
}

pub enum LootKind {
    Fish(FishSpecies),
    Cash(CashValue),
    Food(u32),
    Junk(JunkSprite),
    Consumable(ConsumableKind),
}

#[derive(Clone, Copy)]
enum LootEntry {
    Fish(FishSpecies),
    Cash,
    Food,
    JunkSlot,
}

const LEGENDARY: u32 = 3;
const RARE: u32 = 20;
const COMMON: u32 = 64;

const LOOT_TABLE: &[(u32, LootEntry)] = &[
    (LEGENDARY, LootEntry::Fish(FishSpecies::Mutantfish)),
    (LEGENDARY, LootEntry::Fish(FishSpecies::Goldenfish)),
    (RARE, LootEntry::Fish(FishSpecies::Turbofish)),
    (RARE, LootEntry::Fish(FishSpecies::Jellyfish)),
    (RARE, LootEntry::Fish(FishSpecies::Deadfish)),
    (RARE, LootEntry::Fish(FishSpecies::Koi)),
    (COMMON, LootEntry::Cash),
    (COMMON, LootEntry::Fish(FishSpecies::Merluza)),
    (COMMON, LootEntry::Fish(FishSpecies::Betta)),
    (COMMON, LootEntry::Fish(FishSpecies::Salmon)),
    (COMMON, LootEntry::Fish(FishSpecies::Chromis)),
    (COMMON, LootEntry::Fish(FishSpecies::Tang)),
    (COMMON, LootEntry::Fish(FishSpecies::Carpin)),
    (COMMON, LootEntry::Fish(FishSpecies::Anchoveta)),
    (COMMON, LootEntry::Fish(FishSpecies::Goldfish)),
    (COMMON, LootEntry::Fish(FishSpecies::Snapper)),
    (COMMON, LootEntry::Fish(FishSpecies::Nishiki)),
    (COMMON, LootEntry::Fish(FishSpecies::Aka)),
    (COMMON, LootEntry::Fish(FishSpecies::Kuro)),
    (COMMON, LootEntry::Food),
    (COMMON, LootEntry::JunkSlot),
];

fn roll_weighted<T: Copy>(table: &[(u32, T)], rng: &mut impl RngExt) -> T {
    let total: u32 = table.iter().map(|(w, _)| w).sum();
    let mut v = rng.random_range(0..total);
    for (weight, item) in table {
        if v < *weight {
            return *item;
        }
        v -= weight;
    }
    table.last().unwrap().1
}

fn roll_junk_slot(rng: &mut impl RngExt) -> LootKind {
    match rng.random_range(0..3u32) {
        0 => LootKind::Junk(JunkSprite::new(rng)),
        1 => LootKind::Consumable(ConsumableKind::Coffee),
        _ => LootKind::Consumable(ConsumableKind::Bait),
    }
}

pub fn roll_loot_no_fish(rng: &mut impl RngExt) -> LootKind {
    let non_fish: Vec<(u32, LootEntry)> = LOOT_TABLE
        .iter()
        .filter(|(_, e)| !matches!(e, LootEntry::Fish(_)))
        .copied()
        .collect();
    let total: u32 = non_fish.iter().map(|(w, _)| w).sum();
    let mut v = rng.random_range(0..total);
    for (weight, entry) in &non_fish {
        if v < *weight {
            return match entry {
                LootEntry::Fish(_) => unreachable!(),
                LootEntry::Cash => LootKind::Cash(CashValue::roll(rng)),
                LootEntry::Food => LootKind::Food(rng.random_range(50..=250u32)),
                LootEntry::JunkSlot => roll_junk_slot(rng),
            };
        }
        v -= weight;
    }
    roll_junk_slot(rng)
}

pub fn roll_loot(rng: &mut impl RngExt, bait_stacks: u32) -> LootKind {
    let bait_mult = 1u32 + bait_stacks;
    let total: u32 = LOOT_TABLE
        .iter()
        .map(|(w, _)| if *w < COMMON { w * bait_mult } else { *w })
        .sum();
    let mut v = rng.random_range(0..total);
    for (weight, entry) in LOOT_TABLE {
        let eff_weight = if *weight < COMMON {
            weight * bait_mult
        } else {
            *weight
        };
        if v < eff_weight {
            return match entry {
                LootEntry::Fish(s) => LootKind::Fish(*s),
                LootEntry::Cash => LootKind::Cash(CashValue::roll(rng)),
                LootEntry::Food => LootKind::Food(rng.random_range(50..=250u32)),
                LootEntry::JunkSlot => roll_junk_slot(rng),
            };
        }
        v -= eff_weight;
    }
    roll_junk_slot(rng)
}
