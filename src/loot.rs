use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    BLUE, BROWN, BROWN_DARK, DARK_GRAY, GRAY, GREEN, LIGHT_GREEN, LIGHT_MAGENTA, LIGHT_YELLOW,
    ORANGE, PINK, PURPLE_LIGHT, RED, SILVER, TAN, TERRACOTTA, WHITE,
};
use crate::economy::{Purchasable, Rarity, Sellable};
use crate::entities::cow::CowVariant;
use crate::fishes::species::FishSpecies;

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
const JUNK_COLORS: &[Color] = &[GRAY, BROWN, GREEN, LIGHT_GREEN];

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
    let dark = DARK_GRAY;
    vec![
        (' ', dark),
        (' ', dark),
        (' ', dark),
        ('.', dark),
        ('_', dark),
        ('_', dark),
        ('_', dark),
        ('.', dark),
    ]
}

fn junk_row_mid(rng: &mut impl RngExt) -> Vec<(char, Color)> {
    let dark = DARK_GRAY;
    vec![
        (' ', dark),
        (' ', dark),
        ('(', dark),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (junk_char(rng), junk_color(rng)),
        (')', dark),
        ('.', dark),
    ]
}

fn junk_row_bot(rng: &mut impl RngExt) -> Vec<(char, Color)> {
    let dark = DARK_GRAY;
    vec![
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MilkVariant {
    Plain,
    Chocolate,
    Strawberry,
    Vanilla,
    Alien,
}

impl MilkVariant {
    pub const ALL: &'static [MilkVariant] = &[
        MilkVariant::Plain,
        MilkVariant::Chocolate,
        MilkVariant::Strawberry,
        MilkVariant::Vanilla,
        MilkVariant::Alien,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            MilkVariant::Plain => "Milk",
            MilkVariant::Chocolate => "Chocolate Milk",
            MilkVariant::Strawberry => "Strawberry Milk",
            MilkVariant::Vanilla => "Vanilla Milk",
            MilkVariant::Alien => "Alien Milk",
        }
    }

    pub fn body_color(self) -> Color {
        match self {
            MilkVariant::Plain => WHITE,
            MilkVariant::Chocolate => BROWN_DARK,
            MilkVariant::Strawberry => PINK,
            MilkVariant::Vanilla => LIGHT_YELLOW,
            MilkVariant::Alien => LIGHT_GREEN,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MilkVariant::Plain => {
                "The Pale continues to consume. The liquid in front of you has their face. Bless yourself in the same ivory fire. Better fishing"
            }
            MilkVariant::Chocolate => {
                "Hyper-dense lipid-maximizing slurry. Overrides the baseline biological density caps for absolute mass extraction. Numbers must go up. Increase fish's weight"
            }
            MilkVariant::Strawberry => {
                "Imbues the organism with a Cursed Economic Paradigm (CEP). Compounding artificial market inflation through pastel-tier commodification. Bump sell price"
            }
            MilkVariant::Vanilla => {
                "Corporate-mandated structural amnesia? The prosaic absolute bleaches the sins in the flesh? What has it lost? Cleanses all fish mutations?"
            }
            MilkVariant::Alien => {
                "Cellular-restructuring xeno-pathway enabled by the fluid. The flesh rejects terrestrial biology, embracing the emerald hue. The sky opens up. Alien transformation"
            }
        }
    }

    pub fn cow_variant(self) -> CowVariant {
        match self {
            MilkVariant::Plain => CowVariant::WhiteBlack,
            MilkVariant::Chocolate => CowVariant::Brown,
            MilkVariant::Strawberry => CowVariant::Pink,
            MilkVariant::Vanilla => CowVariant::LightYellow,
            MilkVariant::Alien => CowVariant::LightGreen,
        }
    }
}

const MILK_SELL_PRICE: u32 = 60;
pub const NECRONOMICON_SELL_PRICE: u32 = 7_000;
const NECRONOMICON_PANEL_INNER_W: u16 = 16;
const NECRONOMICON_DESCRIPTION: &str = "An Image [or Picture] of the Law of the Dead. Image and pre-image. Summons a Gate to Hell, The Helltank. The devil has a lot of cash";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumableKind {
    Coffee,
    Bait,
    Milk(MilkVariant),
    Necronomicon,
}

impl ConsumableKind {
    pub fn all() -> Vec<ConsumableKind> {
        let mut v = vec![ConsumableKind::Coffee, ConsumableKind::Bait];
        for &m in MilkVariant::ALL {
            v.push(ConsumableKind::Milk(m));
        }
        v.push(ConsumableKind::Necronomicon);
        v
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => "Coffee",
            ConsumableKind::Bait => "Bait",
            ConsumableKind::Milk(m) => m.display_name(),
            ConsumableKind::Necronomicon => "Necronomicon",
        }
    }

    pub fn lowercase_name(self) -> String {
        self.display_name().to_ascii_lowercase()
    }

    pub fn panel_inner_w(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 8,
            ConsumableKind::Bait => 9,
            ConsumableKind::Milk(_) => MILK_PANEL_INNER_W_LOCAL,
            ConsumableKind::Necronomicon => NECRONOMICON_PANEL_INNER_W,
        }
    }

    pub fn hook_col(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 4,
            ConsumableKind::Bait => 3,
            ConsumableKind::Milk(_) => MILK_HOOK_COL,
            ConsumableKind::Necronomicon => 12,
        }
    }

    pub fn hook_row(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 2,
            ConsumableKind::Bait => 0,
            ConsumableKind::Milk(_) => MILK_HOOK_ROW,
            ConsumableKind::Necronomicon => 0,
        }
    }

    pub fn active_label(self) -> Option<&'static str> {
        match self {
            ConsumableKind::Coffee => Some("caffeinated"),
            ConsumableKind::Bait => Some("baiting"),
            ConsumableKind::Milk(_) | ConsumableKind::Necronomicon => None,
        }
    }

    pub fn active_duration_secs(self) -> Option<f32> {
        match self {
            ConsumableKind::Coffee => Some(crate::consumable::COFFEE_DURATION),
            ConsumableKind::Bait => Some(crate::consumable::BAIT_DURATION),
            ConsumableKind::Milk(_) | ConsumableKind::Necronomicon => None,
        }
    }

    pub fn buy_price(self) -> u32 {
        match self {
            ConsumableKind::Coffee => 10,
            ConsumableKind::Bait => 15,
            ConsumableKind::Milk(_) | ConsumableKind::Necronomicon => 0,
        }
    }

    pub fn sell_price(self) -> u32 {
        match self {
            ConsumableKind::Coffee => 8,
            ConsumableKind::Bait => 12,
            ConsumableKind::Milk(_) => MILK_SELL_PRICE,
            ConsumableKind::Necronomicon => NECRONOMICON_SELL_PRICE,
        }
    }

    pub fn is_buyable(self) -> bool {
        matches!(self, ConsumableKind::Coffee | ConsumableKind::Bait)
    }

    pub fn description(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => {
                "Work-communion-enabling percolated Breverage. Allows terminal-humanii connection. Gives fishes something to believe in. Faster reeling"
            }
            ConsumableKind::Bait => {
                "Lesser-blood sacrifice for higher-entropy lifeforms. Bait Mindset. Even they wish for the Heavens. Get better fishes"
            }
            ConsumableKind::Milk(m) => m.description(),
            ConsumableKind::Necronomicon => NECRONOMICON_DESCRIPTION,
        }
    }

    pub fn rarity(self) -> Rarity {
        Rarity::Common
    }
}

impl Purchasable for ConsumableKind {
    fn buy_price(&self) -> u32 {
        ConsumableKind::buy_price(*self)
    }
    fn display_name(&self) -> &str {
        ConsumableKind::display_name(*self)
    }
}

impl Sellable for ConsumableKind {
    fn sell_price(&self) -> u32 {
        ConsumableKind::sell_price(*self)
    }
    fn display_name(&self) -> &str {
        ConsumableKind::display_name(*self)
    }
}

pub trait Catchable {
    fn rarity(&self) -> Rarity;
    fn catch_weight(&self) -> u32 {
        self.rarity().catch_weight()
    }
    fn on_catch(&self, rng: &mut impl RngExt) -> LootKind;
}

impl Catchable for FishSpecies {
    fn rarity(&self) -> Rarity {
        self.config().rarity
    }
    fn on_catch(&self, _rng: &mut impl RngExt) -> LootKind {
        LootKind::Fish(*self)
    }
}

pub fn coffee_sprite_rows(anim_phase: bool) -> Vec<Vec<(char, Color)>> {
    let steam = SILVER;
    let cup = GRAY;
    let liquid = BROWN_DARK;
    let label = WHITE;

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

const MILK_SPRITE_LINES: &[&str] = &[
    "  _____  ",
    " j_____j ",
    "/_____/_\\",
    "|_____|,|",
    "|     | |",
    "|     | |",
    "|_____|,'",
];

pub fn milk_sprite_rows(variant: MilkVariant) -> Vec<Vec<(char, Color)>> {
    let color = variant.body_color();
    MILK_SPRITE_LINES
        .iter()
        .map(|line| line.chars().map(|c| (c, color)).collect())
        .collect()
}

pub const MILK_SPRITE_W: u16 = 9;
pub const MILK_SPRITE_H: u16 = 7;
const MILK_HOOK_COL: u16 = 9;
const MILK_HOOK_ROW: u16 = 2;
const MILK_PANEL_INNER_W_LOCAL: u16 = MILK_HOOK_COL + 3;

pub fn bait_sprite_rows() -> Vec<Vec<(char, Color)>> {
    let dirt = BROWN_DARK;
    let body = TERRACOTTA;
    let eye = TAN;

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
            CashValue::One => GREEN,
            CashValue::Two => PURPLE_LIGHT,
            CashValue::Five => LIGHT_MAGENTA,
            CashValue::Ten => BLUE,
            CashValue::Twenty => ORANGE,
            CashValue::Hundred => RED,
            CashValue::Thousand => LIGHT_YELLOW,
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
    Junk(JunkSprite),
    Consumable(ConsumableKind),
}

impl ItemKind {
    pub fn display_name(&self) -> &str {
        match self {
            ItemKind::GoldBar => "Gold Bar",
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
}

pub struct LootPool {
    slots: Vec<(u32, PoolSlot)>,
    devils_luck: u32,
}

impl LootPool {
    pub fn default_pool() -> Self {
        let mut slots: Vec<(u32, PoolSlot)> = FishSpecies::all_buyable()
            .iter()
            .map(|&s| (s.config().rarity.catch_weight(), PoolSlot::Species(s)))
            .collect();
        slots.push((COMMON, PoolSlot::Cash));
        slots.push((COMMON, PoolSlot::Food));
        slots.push((JUNK_WEIGHT, PoolSlot::Junk));
        slots.push((COFFEE_WEIGHT, PoolSlot::Consumable(ConsumableKind::Coffee)));
        slots.push((BAIT_WEIGHT, PoolSlot::Consumable(ConsumableKind::Bait)));
        slots.push((
            LEGENDARY,
            PoolSlot::Consumable(ConsumableKind::Necronomicon),
        ));
        slots.push((ULTRA_LEGENDARY, PoolSlot::GoldBar));
        Self {
            slots,
            devils_luck: 0,
        }
    }

    pub fn with_candyfish(mut self) -> Self {
        self.slots.push((
            FishSpecies::Candyfish.config().rarity.catch_weight(),
            PoolSlot::Species(FishSpecies::Candyfish),
        ));
        self
    }

    pub fn fish_excluded() -> Self {
        let mut pool = Self::default_pool();
        pool.slots
            .retain(|(_, s)| !matches!(s, PoolSlot::Species(_)));
        pool
    }

    pub fn with_bait(mut self, stacks: u32) -> Self {
        if stacks == 0 {
            return self;
        }
        let mult = 1u32 + stacks;
        for (w, slot) in &mut self.slots {
            let boostable = match slot {
                PoolSlot::Species(s) => s.config().rarity != Rarity::Common,
                PoolSlot::Consumable(ConsumableKind::Necronomicon) | PoolSlot::GoldBar => true,
                _ => false,
            };
            if boostable {
                *w *= mult;
            }
        }
        self
    }

    pub fn with_cows(mut self, cow_counts: &CowCounts) -> Self {
        let other_total: u32 = self.slots.iter().map(|(w, _)| *w).sum();
        for &variant in MilkVariant::ALL {
            let n = cow_counts.of(variant);
            if n == 0 {
                continue;
            }
            let weight = milk_weight_for_cow_count(other_total, n);
            self.slots
                .push((weight, PoolSlot::Consumable(ConsumableKind::Milk(variant))));
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
                    PoolSlot::Food => {
                        LootKind::Food(rng.random_range(FOOD_AMOUNT_MIN..=FOOD_AMOUNT_MAX))
                    }
                    PoolSlot::Junk => LootKind::Item(ItemKind::Junk(JunkSprite::new(rng))),
                    PoolSlot::Consumable(kind) => LootKind::Item(ItemKind::Consumable(*kind)),
                    PoolSlot::GoldBar => LootKind::Item(ItemKind::GoldBar),
                };
            }
            v -= w;
        }
        LootKind::Item(ItemKind::Junk(JunkSprite::new(rng)))
    }
}

const MILK_DROP_ALPHA: f32 = 0.143;

fn milk_drop_probability(cow_count: u32) -> f32 {
    let n = cow_count as f32;
    let an = MILK_DROP_ALPHA * n;
    an / (1.0 + an)
}

fn milk_weight_for_cow_count(other_total: u32, cow_count: u32) -> u32 {
    let p = milk_drop_probability(cow_count);
    ((other_total as f32) * p / (1.0 - p)).round().max(1.0) as u32
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

pub struct CowCounts {
    pub plain: u32,
    pub chocolate: u32,
    pub strawberry: u32,
    pub vanilla: u32,
    pub alien: u32,
}

impl CowCounts {
    pub fn of(&self, variant: MilkVariant) -> u32 {
        match variant {
            MilkVariant::Plain => self.plain,
            MilkVariant::Chocolate => self.chocolate,
            MilkVariant::Strawberry => self.strawberry,
            MilkVariant::Vanilla => self.vanilla,
            MilkVariant::Alien => self.alien,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.plain + self.chocolate + self.strawberry + self.vanilla + self.alien == 0
    }
}

pub fn roll_loot(
    rng: &mut impl RngExt,
    bait_stacks: u32,
    devils_luck: u32,
    cow_counts: &CowCounts,
) -> LootKind {
    LootPool::default_pool()
        .with_bait(bait_stacks)
        .with_devils_luck(devils_luck)
        .with_cows(cow_counts)
        .roll(rng)
}

pub fn roll_loot_no_fish(
    rng: &mut impl RngExt,
    devils_luck: u32,
    cow_counts: &CowCounts,
) -> LootKind {
    LootPool::fish_excluded()
        .with_devils_luck(devils_luck)
        .with_cows(cow_counts)
        .roll(rng)
}
