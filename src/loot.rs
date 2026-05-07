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
}

#[derive(Clone, Copy)]
enum LootEntry {
    Fish(FishSpecies),
    Cash,
    Food,
    Junk,
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
    (COMMON, LootEntry::Junk),
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

pub fn roll_loot(rng: &mut impl RngExt) -> LootKind {
    match roll_weighted(LOOT_TABLE, rng) {
        LootEntry::Fish(s) => LootKind::Fish(s),
        LootEntry::Cash => LootKind::Cash(CashValue::roll(rng)),
        LootEntry::Food => LootKind::Food(rng.random_range(50..=250u32)),
        LootEntry::Junk => LootKind::Junk(JunkSprite::new(rng)),
    }
}
