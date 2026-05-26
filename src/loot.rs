use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{BROWN, CASH_ORANGE, CASH_PURPLE, COFFEE_LIQUID, LIGHT_GRAY, OFF_WHITE, WORM_BODY, WORM_DIRT, WORM_EYE};
use crate::economy::{Purchasable, Rarity, Sellable};
use crate::fishes::species::{ALL_SPECIES, FishSpecies};

pub const GOLD_BAR_VALUE: u32 = 5_000;
pub const FOOD_AMOUNT_MIN: u32 = 20;
pub const FOOD_AMOUNT_MAX: u32 = 80;
const DEVILS_LUCK_CASH_BONUS: u32 = 30;
const CASH_TIER_SHIFT: u32 = 12;
const CASH_CENTER_IDX: usize = 3;

const ULTRA_LEGENDARY: u32 = 2;
const LEGENDARY: u32 = 4;
const COMMON: u32 = 62;

const JUNK_WEIGHT: u32 = 21;
const COFFEE_WEIGHT: u32 = 21;
const BAIT_WEIGHT: u32 = 20;

const JUNK_FILLER_CHARS: &[char] = &['&', '@', '€', '%', '$', '#', 'X', '<', '>'];
const JUNK_COLORS: &[Color] = &[Color::Gray, BROWN, Color::Green, Color::LightGreen];

pub struct JunkSprite {
    pub rows: Vec<Vec<(char, Color)>>,
}

fn junk_color(rng: &mut impl RngExt) -> Color {
    JUNK_COLORS[rng.random_range(0..JUNK_COLORS.len())]
}

fn junk_char(rng: &mut impl RngExt) -> char {
    JUNK_FILLER_CHARS[rng.random_range(0..JUNK_FILLER_CHARS.len())]
}

fn junk_row_top() -> Vec<(char, Color)> {
    let dark = Color::DarkGray;
    vec![
        (' ', dark), (' ', dark), (' ', dark), ('.', dark),
        ('_', dark), ('_', dark), ('_', dark), ('.', dark),
    ]
}

fn junk_row_mid(rng: &mut impl RngExt) -> Vec<(char, Color)> {
    let dark = Color::DarkGray;
    vec![
        (' ', dark), (' ', dark), ('(', dark),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (')', dark), ('.', dark),
    ]
}

fn junk_row_bot(rng: &mut impl RngExt) -> Vec<(char, Color)> {
    let dark = Color::DarkGray;
    vec![
        ('.', dark), ('(', dark),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (')', dark),
    ]
}

impl JunkSprite {
    pub fn new(rng: &mut impl RngExt) -> Self {
        JunkSprite {
            rows: vec![junk_row_top(), junk_row_mid(rng), junk_row_bot(rng)],
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

    pub fn buy_price(self) -> u32 {
        match self {
            ConsumableKind::Coffee => 10,
            ConsumableKind::Bait => 15,
        }
    }

    pub fn sell_price(self) -> u32 {
        match self {
            ConsumableKind::Coffee => 8,
            ConsumableKind::Bait => 12,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => "Work-communion-enabling percolated Breverage. Allows terminal-humanii connection. Gives fishes something to believe in. Faster reeling",
            ConsumableKind::Bait => "Lesser-blood sacrifice for higher-entropy lifeforms. Bait Mindset. Even they wish for the Heavens. Get better fishes",
        }
    }

    pub fn rarity(self) -> Rarity {
        Rarity::Common
    }
}

impl Purchasable for ConsumableKind {
    fn buy_price(&self) -> u32 { ConsumableKind::buy_price(*self) }
    fn display_name(&self) -> &str { ConsumableKind::display_name(*self) }
}

impl Sellable for ConsumableKind {
    fn sell_price(&self) -> u32 { ConsumableKind::sell_price(*self) }
    fn display_name(&self) -> &str { ConsumableKind::display_name(*self) }
}

pub trait Catchable {
    fn rarity(&self) -> Rarity;
    fn catch_weight(&self) -> u32 { self.rarity().catch_weight() }
    fn on_catch(&self, rng: &mut impl RngExt) -> LootKind;
}

impl Catchable for FishSpecies {
    fn rarity(&self) -> Rarity { self.config().rarity }
    fn on_catch(&self, _rng: &mut impl RngExt) -> LootKind { LootKind::Fish(*self) }
}

pub fn coffee_sprite_rows(anim_phase: bool) -> Vec<Vec<(char, Color)>> {
    let steam = OFF_WHITE;
    let cup = LIGHT_GRAY;
    let liquid = COFFEE_LIQUID;
    let label = Color::White;

    let (top_open, top_close, bot_open, bot_close) = if anim_phase {
        (')', ')', '(', '(')
    } else {
        ('(', '(', ')', ')')
    };

    vec![
        vec![(' ', steam), (' ', steam), (top_open, steam), (top_close, steam)],
        vec![(' ', steam), (' ', steam), (bot_open, steam), (bot_close, steam)],
        vec![(' ', cup), ('|', cup), ('~', liquid), ('~', liquid), ('|', cup)],
        vec![('C', label), ('|', cup), ('_', cup), ('_', cup), ('|', cup)],
    ]
}

pub fn bait_sprite_rows() -> Vec<Vec<(char, Color)>> {
    let dirt = WORM_DIRT;
    let body = WORM_BODY;
    let eye = WORM_EYE;

    vec![
        vec![(' ', dirt), (' ', dirt), (' ', dirt), ('_', dirt)],
        vec![(' ', body), (' ', body), ('(', body), ('º', eye), ('\\', body)],
        vec![(' ', body), ('_', body), ('_', body), (')', body), (' ', body), (')', body)],
        vec![('(', body), ('_', body), ('_', body), ('_', body), ('/', body)],
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
            CashValue::Two => CASH_PURPLE,
            CashValue::Five => Color::LightMagenta,
            CashValue::Ten => Color::Blue,
            CashValue::Twenty => CASH_ORANGE,
            CashValue::Hundred => Color::Red,
            CashValue::Thousand => Color::Yellow,
        }
    }

    pub fn roll(rng: &mut impl RngExt) -> Self {
        roll_weighted(CASH_TABLE, rng)
    }

    pub fn rarity(self) -> Rarity {
        match self {
            CashValue::One | CashValue::Two => Rarity::Common,
            CashValue::Five | CashValue::Ten | CashValue::Twenty => Rarity::Rare,
            CashValue::Hundred | CashValue::Thousand => Rarity::Legendary,
        }
    }
}

pub enum ItemKind {
    GoldBar,
    Necronomicon,
    Junk(JunkSprite),
    Consumable(ConsumableKind),
}

impl ItemKind {
    pub fn display_name(&self) -> &str {
        match self {
            ItemKind::GoldBar => "Gold Bar",
            ItemKind::Necronomicon => "Necronomicon",
            ItemKind::Junk(_) => "Junk",
            ItemKind::Consumable(kind) => ConsumableKind::display_name(*kind),
        }
    }
}

pub enum LootKind {
    Fish(FishSpecies),
    Cash(CashValue),
    Food(u32),
    Item(ItemKind),
}

#[derive(Clone, Copy)]
enum PoolSlot {
    Species(FishSpecies),
    Cash,
    Food,
    Junk,
    Consumable(ConsumableKind),
    GoldBar,
    Necronomicon,
}

pub struct LootPool {
    slots: Vec<(u32, PoolSlot)>,
    devils_luck: u32,
}

impl LootPool {
    pub fn default_pool() -> Self {
        let mut slots: Vec<(u32, PoolSlot)> = ALL_SPECIES
            .iter()
            .map(|&s| (s.config().rarity.catch_weight(), PoolSlot::Species(s)))
            .collect();
        slots.push((COMMON, PoolSlot::Cash));
        slots.push((COMMON, PoolSlot::Food));
        slots.push((JUNK_WEIGHT, PoolSlot::Junk));
        slots.push((COFFEE_WEIGHT, PoolSlot::Consumable(ConsumableKind::Coffee)));
        slots.push((BAIT_WEIGHT, PoolSlot::Consumable(ConsumableKind::Bait)));
        slots.push((LEGENDARY, PoolSlot::Necronomicon));
        slots.push((ULTRA_LEGENDARY, PoolSlot::GoldBar));
        Self { slots, devils_luck: 0 }
    }

    pub fn fish_excluded() -> Self {
        let mut pool = Self::default_pool();
        pool.slots.retain(|(_, s)| !matches!(s, PoolSlot::Species(_)));
        pool
    }

    pub fn with_bait(mut self, stacks: u32) -> Self {
        if stacks == 0 { return self; }
        let mult = 1u32 + stacks;
        for (w, slot) in &mut self.slots {
            let boostable = match slot {
                PoolSlot::Species(s) => s.config().rarity != Rarity::Common,
                PoolSlot::Necronomicon | PoolSlot::GoldBar => true,
                _ => false,
            };
            if boostable { *w *= mult; }
        }
        self
    }

    pub fn with_devils_luck(mut self, level: u32) -> Self {
        self.devils_luck = level;
        if level > 0 {
            for (w, slot) in &mut self.slots {
                if matches!(slot, PoolSlot::Cash) {
                    *w += DEVILS_LUCK_CASH_BONUS * level;
                }
            }
        }
        self
    }

    pub fn roll(&self, rng: &mut impl RngExt) -> LootKind {
        let total: u32 = self.slots.iter().map(|(w, _)| w).sum();
        let mut v = rng.random_range(0..total);
        for (w, slot) in &self.slots {
            if v < *w {
                return match slot {
                    PoolSlot::Species(s) => LootKind::Fish(*s),
                    PoolSlot::Cash => LootKind::Cash(roll_cash_with_luck(rng, self.devils_luck)),
                    PoolSlot::Food => LootKind::Food(rng.random_range(FOOD_AMOUNT_MIN..=FOOD_AMOUNT_MAX)),
                    PoolSlot::Junk => LootKind::Item(ItemKind::Junk(JunkSprite::new(rng))),
                    PoolSlot::Consumable(kind) => LootKind::Item(ItemKind::Consumable(*kind)),
                    PoolSlot::GoldBar => LootKind::Item(ItemKind::GoldBar),
                    PoolSlot::Necronomicon => LootKind::Item(ItemKind::Necronomicon),
                };
            }
            v -= w;
        }
        LootKind::Item(ItemKind::Junk(JunkSprite::new(rng)))
    }
}

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

pub fn roll_loot(rng: &mut impl RngExt, bait_stacks: u32, devils_luck: u32) -> LootKind {
    LootPool::default_pool().with_bait(bait_stacks).with_devils_luck(devils_luck).roll(rng)
}

pub fn roll_loot_no_fish(rng: &mut impl RngExt, devils_luck: u32) -> LootKind {
    LootPool::fish_excluded().with_devils_luck(devils_luck).roll(rng)
}
