use rand::RngExt;
use ratatui::style::Color;

use crate::entities::species::FishSpecies;

pub const GOLD_BAR_VALUE: u32 = 5_000;
pub const FOOD_AMOUNT_MIN: u32 = 20;
pub const FOOD_AMOUNT_MAX: u32 = 80;
const JUNK_SLOT_OUTCOMES: u32 = 3;
const DEVILS_LUCK_CASH_BONUS: u32 = 30;
const CASH_TIER_SHIFT: u32 = 12;
const CASH_CENTER_IDX: usize = 3;

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
    pub const fn all() -> &'static [ConsumableKind] {
        &[ConsumableKind::Coffee, ConsumableKind::Bait]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => "Coffee",
            ConsumableKind::Bait => "Bait",
        }
    }

    pub fn lowercase_name(self) -> String {
        self.display_name().to_ascii_lowercase()
    }

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
    (55, CashValue::One),
    (75, CashValue::Two),
    (95, CashValue::Five),
    (140, CashValue::Ten),
    (195, CashValue::Twenty),
    (75, CashValue::Hundred),
    (20, CashValue::Thousand),
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
    GoldBar,
    Necronomicon,
}

#[derive(Clone, Copy)]
enum LootEntry {
    Fish(FishSpecies),
    Cash,
    Food,
    JunkSlot,
    GoldBar,
    Necronomicon,
}

const ULTRA_LEGENDARY: u32 = 2;
const LEGENDARY: u32 = 4;
const RARE: u32 = 22;
const COMMON: u32 = 62;

const LOOT_TABLE: &[(u32, LootEntry)] = &[
    (LEGENDARY, LootEntry::Fish(FishSpecies::Mutantfish)),
    (LEGENDARY, LootEntry::Fish(FishSpecies::Goldenfish)),
    (LEGENDARY, LootEntry::Necronomicon),
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
    (ULTRA_LEGENDARY, LootEntry::GoldBar),
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

fn roll_cash_with_luck(rng: &mut impl RngExt, devils_luck: u32) -> CashValue {
    if devils_luck == 0 {
        return CashValue::roll(rng);
    }
    let shifted: Vec<(u32, CashValue)> = CASH_TABLE
        .iter()
        .enumerate()
        .map(|(i, (w, v))| {
            let rank_dist = (i as i32 - CASH_CENTER_IDX as i32).unsigned_abs();
            let shift = CASH_TIER_SHIFT * devils_luck * rank_dist;
            let new_w = if i < CASH_CENTER_IDX {
                w.saturating_sub(shift).max(1)
            } else {
                w + shift
            };
            (new_w, *v)
        })
        .collect();
    roll_weighted(&shifted, rng)
}

fn roll_junk_slot(rng: &mut impl RngExt) -> LootKind {
    match rng.random_range(0..JUNK_SLOT_OUTCOMES) {
        0 => LootKind::Junk(JunkSprite::new(rng)),
        1 => LootKind::Consumable(ConsumableKind::Coffee),
        _ => LootKind::Consumable(ConsumableKind::Bait),
    }
}

pub fn roll_loot_no_fish(rng: &mut impl RngExt, devils_luck: u32) -> LootKind {
    let non_fish: Vec<(u32, LootEntry)> = LOOT_TABLE
        .iter()
        .filter(|(_, e)| !matches!(e, LootEntry::Fish(_)))
        .map(|(w, e)| {
            let eff_w = if matches!(e, LootEntry::Cash) {
                w + DEVILS_LUCK_CASH_BONUS * devils_luck
            } else {
                *w
            };
            (eff_w, *e)
        })
        .collect();
    let total: u32 = non_fish.iter().map(|(w, _)| w).sum();
    let mut v = rng.random_range(0..total);
    for (weight, entry) in &non_fish {
        if v < *weight {
            return match entry {
                LootEntry::Fish(_) => unreachable!(),
                LootEntry::Cash => LootKind::Cash(roll_cash_with_luck(rng, devils_luck)),
                LootEntry::Food => {
                    LootKind::Food(rng.random_range(FOOD_AMOUNT_MIN..=FOOD_AMOUNT_MAX))
                }
                LootEntry::JunkSlot => roll_junk_slot(rng),
                LootEntry::GoldBar => LootKind::GoldBar,
                LootEntry::Necronomicon => LootKind::Necronomicon,
            };
        }
        v -= weight;
    }
    roll_junk_slot(rng)
}

pub fn roll_loot(rng: &mut impl RngExt, bait_stacks: u32, devils_luck: u32) -> LootKind {
    let bait_mult = 1u32 + bait_stacks;
    let total: u32 = LOOT_TABLE
        .iter()
        .map(|(w, e)| {
            let base = if *w < COMMON { w * bait_mult } else { *w };
            if matches!(e, LootEntry::Cash) {
                base + DEVILS_LUCK_CASH_BONUS * devils_luck
            } else {
                base
            }
        })
        .sum();
    let mut v = rng.random_range(0..total);
    for (weight, entry) in LOOT_TABLE {
        let base_eff = if *weight < COMMON {
            weight * bait_mult
        } else {
            *weight
        };
        let eff_weight = if matches!(entry, LootEntry::Cash) {
            base_eff + DEVILS_LUCK_CASH_BONUS * devils_luck
        } else {
            base_eff
        };
        if v < eff_weight {
            return match entry {
                LootEntry::Fish(s) => LootKind::Fish(*s),
                LootEntry::Cash => LootKind::Cash(roll_cash_with_luck(rng, devils_luck)),
                LootEntry::Food => {
                    LootKind::Food(rng.random_range(FOOD_AMOUNT_MIN..=FOOD_AMOUNT_MAX))
                }
                LootEntry::JunkSlot => roll_junk_slot(rng),
                LootEntry::GoldBar => LootKind::GoldBar,
                LootEntry::Necronomicon => LootKind::Necronomicon,
            };
        }
        v -= eff_weight;
    }
    roll_junk_slot(rng)
}
