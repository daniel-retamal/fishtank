use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    BLUE, BROWN, BROWN_DARK, CREAM, DARK_GRAY, GRAY, GREEN, LIGHT_GREEN, LIGHT_MAGENTA,
    LIGHT_YELLOW, ORANGE, PINK, PURPLE_LIGHT, RED, SILVER, STEEL, TAN, TERRACOTTA, WHITE,
};
use crate::economy::{Purchasable, Rarity, Sellable};
use crate::entities::cow::CowVariant;
use crate::entities::glistening::GlisteningMode;
use crate::fishes::parts::{Part, PartTier};
use crate::fishes::species::FishSpecies;
use crate::sprite::apply_glisten;
use crate::tank::TankKind;
use crate::ui::hints::{
    HINT_ENTER_CAPTURE, HINT_ENTER_CONNECT, HINT_ENTER_IRRADIATE, HINT_ENTER_SUMMON,
};

pub const FOOD_AMOUNT_MIN: u32 = 20;
pub const FOOD_AMOUNT_MAX: u32 = 80;
const DEVILS_LUCK_CASH_BONUS: u32 = 30;
const CASH_TIER_SHIFT: u32 = 12;
const CASH_CENTER_IDX: usize = 3;

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
    Irradiated,
}

impl MilkVariant {
    pub const ALL: &'static [MilkVariant] = &[
        MilkVariant::Plain,
        MilkVariant::Chocolate,
        MilkVariant::Strawberry,
        MilkVariant::Vanilla,
        MilkVariant::Alien,
        MilkVariant::Irradiated,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            MilkVariant::Plain => "Milk",
            MilkVariant::Chocolate => "Chocolate Milk",
            MilkVariant::Strawberry => "Strawberry Milk",
            MilkVariant::Vanilla => "Vanilla Milk",
            MilkVariant::Alien => "Alien Milk",
            MilkVariant::Irradiated => "Irradiated Milk",
        }
    }

    pub fn body_color(self) -> Color {
        match self {
            MilkVariant::Plain => WHITE,
            MilkVariant::Chocolate => BROWN_DARK,
            MilkVariant::Strawberry => PINK,
            MilkVariant::Vanilla => LIGHT_YELLOW,
            MilkVariant::Alien => LIGHT_GREEN,
            MilkVariant::Irradiated => GREEN,
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
                "Cellular-restructuring xeno-pathway fluid enabler. The flesh rejects terrestrial biology, embracing the emerald hue. The sky opens up. Alien transformation"
            }
            MilkVariant::Irradiated => {
                "Rage against the carcase. Entfesselt Sein Fleisch. Let the self dissolve, for a brief moment, in the unnverving experience of letting go. Random mutations"
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
            MilkVariant::Irradiated => CowVariant::LightGreen,
        }
    }
}

const MILK_SELL_PRICE: u32 = 60;
const NECRONOMICON_PANEL_INNER_W: u16 = 16;
const NECRONOMICON_DESCRIPTION: &str = "An Image [or Picture] of the Law of the Dead. Image and pre-image. Summons a Gate to Hell, The Helltank. The devil has a lot of cash";
const DEMON_CORE_PANEL_INNER_W: u16 = 18;
const DEMON_CORE_HOOK_COL: u16 = 13;
const DEMON_CORE_HOOK_ROW: u16 = 1;
const DEMON_CORE_DESCRIPTION: &str = "A heavy metal heart quietly rotting with anger. Your <player_species> <species_main_appendage> yearns for its burn. Bring forth its shimmering nightmare in the Radioactivetank. Unchain your biology";
const COMPUTER_HOOK_ROW: u16 = 2;
const COMPUTER_DESCRIPTION: &str = "The monster whose influence has changed history, leaving none of it left. The Angelii are speaking, listen. Connect to the terminal and see the Matrixtank.";
const BLANK_WAFER_BUY_PRICE: u32 = 50;
pub const BLANK_WAFER_SELL_PRICE: u32 = resale_price(BLANK_WAFER_BUY_PRICE);
const FABRICATOR_BUY_PRICE: u32 = 400;
const FABRICATOR_SELL_PRICE: u32 = resale_price(FABRICATOR_BUY_PRICE);
const FABRICATOR_DESCRIPTION: &str = "A single-use fabricator. Prints a Circuit Blueprint into fresh botfish at the Foundry, spending a Blank Wafer per fish and every part the design uses. Foundry stock.";
const BLANK_WAFER_DESCRIPTION: &str = "Uncut silence, ground until it holds your face. It has never been told what it is and it is waiting. Every circuit you will ever love begins as this nothing. Foundry stock.";
const RESALE_PERCENT: u32 = 80;
const PERCENT: u32 = 100;
const BLANK_BLUEPRINT_BUY_PRICE: u32 = 60;
const BLANK_BLUEPRINT_SELL_PRICE: u32 = resale_price(BLANK_BLUEPRINT_BUY_PRICE);
const BLANK_BLUEPRINT_DESCRIPTION: &str = "A blank circuit blueprint. Captures the wired botfish of this tank into a named blueprint. Foundry stock.";
pub const CIRCUIT_BLUEPRINT_NAME: &str = "Circuit Blueprint";
pub const CIRCUIT_BLUEPRINT_SELL_PRICE: u32 = BLANK_BLUEPRINT_SELL_PRICE;
pub const CIRCUIT_BLUEPRINT_DESCRIPTION: &str =
    "A captured circuit. Print or etch it at the Foundry.";

const fn resale_price(buy_price: u32) -> u32 {
    buy_price * RESALE_PERCENT / PERCENT
}
const BLUEPRINT_COLOR: Color = BLUE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NameTarget {
    Tank(TankKind),
    Blueprint,
}

impl NameTarget {
    pub fn header(self) -> String {
        match self {
            NameTarget::Tank(kind) => format!("Name your {}", kind.display_name()),
            NameTarget::Blueprint => format!("Name your {CIRCUIT_BLUEPRINT_NAME}"),
        }
    }

    pub fn border(self) -> Color {
        match self {
            NameTarget::Tank(kind) => kind.config().bubble_color,
            NameTarget::Blueprint => BLUEPRINT_COLOR,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumableKind {
    Coffee,
    Bait,
    Milk(MilkVariant),
    Necronomicon,
    DemonCore,
    Computer,
    VoidSeed,
    BlankWafer,
    Part(Part),
    Fabricator,
    BlankBlueprint,
}

impl ConsumableKind {
    pub fn all() -> Vec<ConsumableKind> {
        let mut v = vec![ConsumableKind::Coffee, ConsumableKind::Bait];
        for &m in MilkVariant::ALL {
            v.push(ConsumableKind::Milk(m));
        }
        v.push(ConsumableKind::Necronomicon);
        v.push(ConsumableKind::DemonCore);
        v.push(ConsumableKind::Computer);
        v.push(ConsumableKind::VoidSeed);
        for &part in Part::ALL {
            v.push(ConsumableKind::Part(part));
        }
        v.push(ConsumableKind::BlankWafer);
        v.push(ConsumableKind::Fabricator);
        v.push(ConsumableKind::BlankBlueprint);
        v
    }

    pub fn bench_stock() -> Vec<ConsumableKind> {
        let mut stock: Vec<ConsumableKind> = Self::all()
            .into_iter()
            .filter(|kind| kind.sold_at_bench())
            .collect();
        stock.sort_by_key(|kind| kind.bench_tier().index());
        stock
    }

    pub fn bench_tier(self) -> PartTier {
        match self {
            ConsumableKind::Part(part) => part.tier(),
            _ => PartTier::Materials,
        }
    }

    fn sold_at_bench(self) -> bool {
        self.is_robotics_stock() && self.buy_price() > 0
    }

    pub fn is_robotics_stock(self) -> bool {
        matches!(
            self,
            ConsumableKind::BlankWafer
                | ConsumableKind::Part(_)
                | ConsumableKind::Fabricator
                | ConsumableKind::BlankBlueprint
        )
    }

    pub fn robotics_stock() -> Vec<ConsumableKind> {
        Self::all()
            .into_iter()
            .filter(|kind| kind.is_robotics_stock())
            .collect()
    }

    pub fn seeds() -> Vec<ConsumableKind> {
        Self::all()
            .into_iter()
            .filter(|kind| kind.summons_tank().is_some())
            .collect()
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ConsumableKind::Coffee => "Coffee",
            ConsumableKind::Bait => "Bait",
            ConsumableKind::Milk(m) => m.display_name(),
            ConsumableKind::Necronomicon => "Necronomicon",
            ConsumableKind::DemonCore => "Demon Core",
            ConsumableKind::Computer => "Computer",
            ConsumableKind::VoidSeed => VOID_SEED_NAME,
            ConsumableKind::BlankWafer => "Blank Wafer",
            ConsumableKind::Part(part) => part.display_name(),
            ConsumableKind::Fabricator => "Fabricator",
            ConsumableKind::BlankBlueprint => "Blank Circuit Blueprint",
        }
    }

    pub fn summons_tank(self) -> Option<TankKind> {
        match self {
            ConsumableKind::Necronomicon => Some(TankKind::Hell),
            ConsumableKind::DemonCore => Some(TankKind::Rad),
            ConsumableKind::Computer => Some(TankKind::Matrix),
            ConsumableKind::VoidSeed => Some(TankKind::Void),
            _ => None,
        }
    }

    pub fn name_target(self) -> Option<NameTarget> {
        if let Some(kind) = self.summons_tank() {
            return Some(NameTarget::Tank(kind));
        }
        (self == ConsumableKind::BlankBlueprint).then_some(NameTarget::Blueprint)
    }

    pub fn summon_hint(self) -> &'static str {
        match self {
            ConsumableKind::DemonCore => HINT_ENTER_IRRADIATE,
            ConsumableKind::Computer => HINT_ENTER_CONNECT,
            ConsumableKind::BlankBlueprint => HINT_ENTER_CAPTURE,
            _ => HINT_ENTER_SUMMON,
        }
    }

    pub fn summon_border_override(self) -> Option<Color> {
        match self {
            ConsumableKind::Computer => Some(DARK_GRAY),
            _ => None,
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
            ConsumableKind::DemonCore => DEMON_CORE_PANEL_INNER_W,
            ConsumableKind::Computer => computer_art().panel_inner_w(),
            ConsumableKind::VoidSeed => void_seed_art().panel_inner_w(),
            ConsumableKind::BlankWafer => blank_wafer_art().panel_inner_w(),
            ConsumableKind::Part(part) => part_art(part).panel_inner_w(),
            ConsumableKind::Fabricator => fabricator_art().panel_inner_w(),
            ConsumableKind::BlankBlueprint => blank_blueprint_art().panel_inner_w(),
        }
    }

    pub fn hook_col(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 4,
            ConsumableKind::Bait => 3,
            ConsumableKind::Milk(_) => MILK_HOOK_COL,
            ConsumableKind::Necronomicon => 12,
            ConsumableKind::DemonCore => DEMON_CORE_HOOK_COL,
            ConsumableKind::Computer => computer_art().hook_col(),
            ConsumableKind::VoidSeed => void_seed_art().hook_col(),
            ConsumableKind::BlankWafer => blank_wafer_art().hook_col(),
            ConsumableKind::Part(part) => part_art(part).hook_col(),
            ConsumableKind::Fabricator => fabricator_art().hook_col(),
            ConsumableKind::BlankBlueprint => blank_blueprint_art().hook_col(),
        }
    }

    pub fn hook_row(self) -> u16 {
        match self {
            ConsumableKind::Coffee => 2,
            ConsumableKind::Bait => 0,
            ConsumableKind::Milk(_) => MILK_HOOK_ROW,
            ConsumableKind::Necronomicon => 0,
            ConsumableKind::DemonCore => DEMON_CORE_HOOK_ROW,
            ConsumableKind::Computer => computer_art().hook_row(),
            ConsumableKind::VoidSeed => void_seed_art().hook_row(),
            ConsumableKind::BlankWafer => blank_wafer_art().hook_row(),
            ConsumableKind::Part(part) => part_art(part).hook_row(),
            ConsumableKind::Fabricator => fabricator_art().hook_row(),
            ConsumableKind::BlankBlueprint => blank_blueprint_art().hook_row(),
        }
    }

    pub fn active_label(self) -> Option<&'static str> {
        match self {
            ConsumableKind::Coffee => Some("caffeinated"),
            ConsumableKind::Bait => Some("baiting"),
            ConsumableKind::Milk(_)
            | ConsumableKind::Necronomicon
            | ConsumableKind::DemonCore
            | ConsumableKind::Computer
            | ConsumableKind::VoidSeed
            | ConsumableKind::BlankWafer
            | ConsumableKind::Part(_)
            | ConsumableKind::Fabricator
            | ConsumableKind::BlankBlueprint => None,
        }
    }

    pub fn active_duration_secs(self) -> Option<f32> {
        match self {
            ConsumableKind::Coffee => Some(crate::consumable::COFFEE_DURATION),
            ConsumableKind::Bait => Some(crate::consumable::BAIT_DURATION),
            ConsumableKind::Milk(_)
            | ConsumableKind::Necronomicon
            | ConsumableKind::DemonCore
            | ConsumableKind::Computer
            | ConsumableKind::VoidSeed
            | ConsumableKind::BlankWafer
            | ConsumableKind::Part(_)
            | ConsumableKind::Fabricator
            | ConsumableKind::BlankBlueprint => None,
        }
    }

    pub fn buy_price(self) -> u32 {
        if let Some(kind) = self.summons_tank() {
            return kind.buy_price();
        }
        match self {
            ConsumableKind::Coffee => 10,
            ConsumableKind::Bait => 15,
            ConsumableKind::Part(part) => part.price(),
            ConsumableKind::BlankWafer => BLANK_WAFER_BUY_PRICE,
            ConsumableKind::Fabricator => FABRICATOR_BUY_PRICE,
            ConsumableKind::BlankBlueprint => BLANK_BLUEPRINT_BUY_PRICE,
            ConsumableKind::Milk(_)
            | ConsumableKind::Necronomicon
            | ConsumableKind::DemonCore
            | ConsumableKind::Computer
            | ConsumableKind::VoidSeed => 0,
        }
    }

    pub fn sell_price(self) -> u32 {
        if self.summons_tank().is_some() {
            return resale_price(self.buy_price());
        }
        match self {
            ConsumableKind::Coffee => 8,
            ConsumableKind::Bait => 12,
            ConsumableKind::Milk(_) => MILK_SELL_PRICE,
            ConsumableKind::Part(part) => resale_price(part.price()),
            ConsumableKind::BlankWafer => BLANK_WAFER_SELL_PRICE,
            ConsumableKind::Fabricator => FABRICATOR_SELL_PRICE,
            ConsumableKind::BlankBlueprint => BLANK_BLUEPRINT_SELL_PRICE,
            ConsumableKind::Necronomicon
            | ConsumableKind::DemonCore
            | ConsumableKind::Computer
            | ConsumableKind::VoidSeed => 0,
        }
    }

    pub fn can_be_consumed(self) -> bool {
        !matches!(self, ConsumableKind::BlankWafer)
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
            ConsumableKind::DemonCore => DEMON_CORE_DESCRIPTION,
            ConsumableKind::Computer => COMPUTER_DESCRIPTION,
            ConsumableKind::VoidSeed => VOID_SEED_DESCRIPTION,
            ConsumableKind::BlankWafer => BLANK_WAFER_DESCRIPTION,
            ConsumableKind::Part(part) => part.description(),
            ConsumableKind::Fabricator => FABRICATOR_DESCRIPTION,
            ConsumableKind::BlankBlueprint => BLANK_BLUEPRINT_DESCRIPTION,
        }
    }

    pub fn rarity(self) -> Rarity {
        match self {
            ConsumableKind::Necronomicon
            | ConsumableKind::DemonCore
            | ConsumableKind::Computer
            | ConsumableKind::VoidSeed => Rarity::Legendary,
            ConsumableKind::Part(part) => part.rarity(),
            ConsumableKind::Fabricator => Rarity::Rare,
            _ => Rarity::Common,
        }
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
    if variant == MilkVariant::Irradiated {
        return irradiated_milk_sprite();
    }
    let color = variant.body_color();
    MILK_SPRITE_LINES
        .iter()
        .map(|line| line.chars().map(|c| (c, color)).collect())
        .collect()
}

fn irradiated_milk_sprite() -> Vec<Vec<(char, Color)>> {
    let mut rows: Vec<Vec<(char, Color)>> = MILK_SPRITE_LINES
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let color = if i % 2 == 0 { GREEN } else { LIGHT_GREEN };
            line.chars().map(|c| (c, color)).collect()
        })
        .collect();
    apply_glisten(
        &mut rows,
        std::f32::consts::FRAC_PI_4,
        GlisteningMode::Wave,
        GREEN,
        PURPLE_LIGHT,
    );
    rows
}

const DEMON_CORE_SPRITE_LINES: &[&str] = &[
    "       ,__,",
    r#"     _|    |_"#,
    r#"  .'` \,__,/ ``."#,
    " :              :",
    r#"  \`..,____,..'/"#,
    r#"   `.       _.'"#,
    r#"     `"----"'"#,
];

pub fn demoncore_sprite_rows(glisten_phase: f32) -> Vec<Vec<(char, Color)>> {
    let mut rows: Vec<Vec<(char, Color)>> = DEMON_CORE_SPRITE_LINES
        .iter()
        .map(|line| line.chars().map(|c| (c, DARK_GRAY)).collect())
        .collect();
    apply_glisten(
        &mut rows,
        glisten_phase,
        GlisteningMode::Wave,
        DARK_GRAY,
        WHITE,
    );
    rows
}

const COMPUTER_SPRITE_LINES: &[&str] = &[
    " ________",
    "| ______o|",
    "||__---_||",
    "| ______ |",
    "||______||",
    "|--------|",
    "|      O |",
    "|      | |",
    "|      | |",
    "|      | |",
    "|::::::::|",
];

const HOOK_LINE_CLEARANCE: u16 = 1;
const HOOKED_PANEL_MARGIN: u16 = 3;

pub struct HookedArt {
    lines: Vec<&'static str>,
    hook_row: u16,
}

impl HookedArt {
    pub fn new(lines: &'static [&'static str], hook_row: u16) -> Self {
        let indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        Self {
            lines: lines
                .iter()
                .map(|line| line.get(indent..).unwrap_or_default().trim_end())
                .collect(),
            hook_row,
        }
    }

    fn width_of(line: &str) -> u16 {
        line.chars().count() as u16
    }

    pub fn hook_row(&self) -> u16 {
        self.hook_row
    }

    pub fn hook_col(&self) -> u16 {
        let on_hook = self
            .lines
            .get(self.hook_row as usize)
            .map_or(0, |line| Self::width_of(line));
        self.lines
            .iter()
            .take(self.hook_row as usize)
            .map(|line| Self::width_of(line) + HOOK_LINE_CLEARANCE)
            .fold(on_hook, u16::max)
    }

    pub fn panel_inner_w(&self) -> u16 {
        self.hook_col() + HOOKED_PANEL_MARGIN
    }

    pub fn lines(&self) -> &[&'static str] {
        &self.lines
    }

    pub fn rows(&self, color: Color) -> Vec<Vec<(char, Color)>> {
        self.lines
            .iter()
            .map(|line| line.chars().map(|ch| (ch, color)).collect())
            .collect()
    }
}

fn computer_art() -> HookedArt {
    HookedArt::new(COMPUTER_SPRITE_LINES, COMPUTER_HOOK_ROW)
}

pub fn computer_sprite_rows() -> Vec<Vec<(char, Color)>> {
    computer_art().rows(CREAM)
}

const BLANK_WAFER_SPRITE_LINES: &[&str] = &[
    "   ,-----.   ",
    " ,'  : : :`. ",
    "/ : : : : : \\",
    "|: : : : : :|",
    "\\ : : : : : /",
    " `. : : :  ,'",
    "   `-----'   ",
];

const BLANK_WAFER_HOOK_ROW: u16 = (BLANK_WAFER_SPRITE_LINES.len() / 2) as u16;

fn blank_wafer_art() -> HookedArt {
    HookedArt::new(BLANK_WAFER_SPRITE_LINES, BLANK_WAFER_HOOK_ROW)
}

pub fn blank_wafer_sprite_rows() -> Vec<Vec<(char, Color)>> {
    blank_wafer_art().rows(SILVER)
}

const VOID_SEED_NAME: &str = " ";
const VOID_SEED_DESCRIPTION: &str = "";
const VOID_SEED_SIDE: usize = 7;
const VOID_SEED_SPRITE_LINES: &[&str] = &[
    "       ", "       ", "       ", "       ", "       ", "       ", "       ",
];

fn void_seed_art() -> HookedArt {
    HookedArt::new(VOID_SEED_SPRITE_LINES, (VOID_SEED_SIDE / 2) as u16)
}

pub fn void_seed_sprite_rows() -> Vec<Vec<(char, Color)>> {
    void_seed_art().rows(WHITE)
}

const BLANK_BLUEPRINT_SPRITE_LINES: &[&str] = &[
    r"   ,--------.",
    r"  / .  .  . /|",
    r" /________ / |",
    r" |  .  .  | /",
    r" |________|/",
];

const BLANK_BLUEPRINT_HOOK_ROW: u16 = (BLANK_BLUEPRINT_SPRITE_LINES.len() / 2) as u16;

fn blank_blueprint_art() -> HookedArt {
    HookedArt::new(BLANK_BLUEPRINT_SPRITE_LINES, BLANK_BLUEPRINT_HOOK_ROW)
}

pub fn blank_blueprint_sprite_rows() -> Vec<Vec<(char, Color)>> {
    blank_blueprint_art().rows(BLUEPRINT_COLOR)
}

const FABRICATOR_SPRITE_LINES: &[&str] = &[
    r" _________",
    r"|[=======]|",
    r"|  _____  |",
    r"| |_____| |",
    r"|_________|",
];

const FABRICATOR_HOOK_ROW: u16 = (FABRICATOR_SPRITE_LINES.len() / 2) as u16;

fn fabricator_art() -> HookedArt {
    HookedArt::new(FABRICATOR_SPRITE_LINES, FABRICATOR_HOOK_ROW)
}

pub fn fabricator_sprite_rows() -> Vec<Vec<(char, Color)>> {
    fabricator_art().rows(ORANGE)
}

const PART_HOOK_ROW: u16 = 2;

pub fn part_art(part: Part) -> HookedArt {
    HookedArt::new(part.sprite_lines(), PART_HOOK_ROW)
}

pub fn part_sprite_rows(part: Part) -> Vec<Vec<(char, Color)>> {
    part_art(part).rows(STEEL)
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
    Junk(JunkSprite),
    Consumable(ConsumableKind),
}

impl ItemKind {
    pub fn display_name(&self) -> &str {
        match self {
            ItemKind::Junk(_) => "Junk",
            ItemKind::Consumable(kind) => ConsumableKind::display_name(*kind),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum StockItem {
    Consumable(ConsumableKind),
    Junk,
}

impl StockItem {
    pub const COFFEE: StockItem = StockItem::Consumable(ConsumableKind::Coffee);
    pub const BAIT: StockItem = StockItem::Consumable(ConsumableKind::Bait);
    pub const NECRONOMICON: StockItem = StockItem::Consumable(ConsumableKind::Necronomicon);
    pub const DEMON_CORE: StockItem = StockItem::Consumable(ConsumableKind::DemonCore);
    pub const COMPUTER: StockItem = StockItem::Consumable(ConsumableKind::Computer);

    pub fn display_name(self) -> &'static str {
        match self {
            StockItem::Consumable(kind) => ConsumableKind::display_name(kind),
            StockItem::Junk => "Junk",
        }
    }

    pub fn is_consumable(self) -> bool {
        matches!(self, StockItem::Consumable(kind) if kind.can_be_consumed())
    }

    pub fn rarity(self) -> Rarity {
        match self {
            StockItem::Consumable(kind) => kind.rarity(),
            StockItem::Junk => Rarity::Common,
        }
    }

    pub fn gift_quantity(self) -> u32 {
        self.rarity().gift_quantity()
    }

    pub fn from_display_name(name: &str) -> Option<StockItem> {
        let lower = name.to_ascii_lowercase();
        if lower == "junk" {
            return Some(StockItem::Junk);
        }
        ConsumableKind::all()
            .into_iter()
            .find(|kind| ConsumableKind::display_name(*kind).to_ascii_lowercase() == lower)
            .map(StockItem::Consumable)
    }

    pub fn from_item(item: &ItemKind) -> Option<StockItem> {
        match item {
            ItemKind::Junk(_) => Some(StockItem::Junk),
            ItemKind::Consumable(kind) => Some(StockItem::Consumable(*kind)),
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
}

pub struct LootPool {
    slots: Vec<(u32, PoolSlot)>,
    devils_luck: u32,
}

impl LootPool {
    pub fn default_pool() -> Self {
        let mut slots: Vec<(u32, PoolSlot)> = FishSpecies::all_wild()
            .iter()
            .map(|&s| (s.config().rarity.catch_weight(), PoolSlot::Species(s)))
            .collect();
        slots.push((Rarity::Common.catch_weight(), PoolSlot::Cash));
        slots.push((Rarity::Common.catch_weight(), PoolSlot::Food));
        slots.push((JUNK_WEIGHT, PoolSlot::Junk));
        slots.push((COFFEE_WEIGHT, PoolSlot::Consumable(ConsumableKind::Coffee)));
        slots.push((BAIT_WEIGHT, PoolSlot::Consumable(ConsumableKind::Bait)));
        let mut pool = Self {
            slots,
            devils_luck: 0,
        };
        for seed in ConsumableKind::seeds() {
            pool.push_consumable(seed);
        }
        pool
    }

    pub fn with_native(mut self, kind: TankKind) -> Self {
        for species in FishSpecies::native_to(kind) {
            self = self.with_species(species);
        }
        if kind.config().robotics_loot {
            self = self.with_robotics();
        }
        self
    }

    fn with_species(mut self, species: FishSpecies) -> Self {
        self.slots.push((
            species.config().rarity.catch_weight(),
            PoolSlot::Species(species),
        ));
        self
    }

    fn push_consumable(&mut self, kind: ConsumableKind) {
        self.slots
            .push((kind.rarity().catch_weight(), PoolSlot::Consumable(kind)));
    }

    pub fn with_robotics(mut self) -> Self {
        for &part in Part::ALL {
            self.push_consumable(ConsumableKind::Part(part));
        }
        self.push_consumable(ConsumableKind::BlankWafer);
        self.push_consumable(ConsumableKind::Fabricator);
        self.push_consumable(ConsumableKind::BlankBlueprint);
        self
    }

    pub fn with_grace(mut self, stacks: u32) -> Self {
        if stacks == 0 {
            return self;
        }
        let mult = 1u32 + stacks;
        for (w, slot) in &mut self.slots {
            let legendary = match slot {
                PoolSlot::Species(s) => s.config().rarity == Rarity::Legendary,
                PoolSlot::Consumable(kind) => kind.rarity() == Rarity::Legendary,
                _ => false,
            };
            if legendary {
                *w *= mult;
            }
        }
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
                PoolSlot::Consumable(kind) => kind.rarity() != Rarity::Common,
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
    pub irradiated: u32,
}

impl CowCounts {
    pub fn of(&self, variant: MilkVariant) -> u32 {
        match variant {
            MilkVariant::Plain => self.plain,
            MilkVariant::Chocolate => self.chocolate,
            MilkVariant::Strawberry => self.strawberry,
            MilkVariant::Vanilla => self.vanilla,
            MilkVariant::Alien => self.alien,
            MilkVariant::Irradiated => self.irradiated,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.plain + self.chocolate + self.strawberry + self.vanilla + self.alien + self.irradiated
            == 0
    }
}

pub fn roll_loot_no_fish(
    rng: &mut impl RngExt,
    devils_luck: u32,
    grace_stacks: u32,
    cow_counts: &CowCounts,
) -> LootKind {
    LootPool::fish_excluded()
        .with_devils_luck(devils_luck)
        .with_grace(grace_stacks)
        .with_cows(cow_counts)
        .roll(rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::species::{ALL_SPECIES, Habitat};

    const ROLLS: usize = 20_000;

    fn rolled_consumables(pool: &LootPool) -> Vec<ConsumableKind> {
        let mut rng = rand::rng();
        (0..ROLLS)
            .filter_map(|_| match pool.roll(&mut rng) {
                LootKind::Item(ItemKind::Consumable(kind)) => Some(kind),
                _ => None,
            })
            .collect()
    }

    fn species_in(pool: &LootPool) -> Vec<FishSpecies> {
        pool.slots
            .iter()
            .filter_map(|(_, slot)| match slot {
                PoolSlot::Species(species) => Some(*species),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_tank_adds_exactly_its_native_species_to_the_wild_pool() {
        for &kind in TankKind::all() {
            let pool = LootPool::default_pool().with_native(kind);
            let mut expected = FishSpecies::all_wild().to_vec();
            expected.extend(FishSpecies::native_to(kind));
            assert_eq!(species_in(&pool), expected, "{}", kind.display_name());
            let drops_parts = pool
                .slots
                .iter()
                .any(|(_, slot)| matches!(slot, PoolSlot::Consumable(ConsumableKind::Part(_))));
            assert_eq!(
                drops_parts,
                kind.config().robotics_loot,
                "{} drops parts only if its config says so",
                kind.display_name()
            );
        }
    }

    #[test]
    fn a_legendary_is_caught_only_on_its_banner_and_everything_else_everywhere() {
        for &species in ALL_SPECIES {
            let config = species.config();
            let native = matches!(config.habitat, Habitat::Native(_));
            assert_eq!(
                native,
                config.rarity == Rarity::Legendary,
                "{} has the wrong habitat for its rarity",
                species.display_name()
            );
            assert!(
                config.habitat != Habitat::Nowhere,
                "{}",
                species.display_name()
            );
        }
        assert_eq!(FishSpecies::Unfish.config().habitat, Habitat::Nowhere);
    }

    #[test]
    fn each_banner_features_the_fish_the_game_already_ties_to_it() {
        let banners = [
            (TankKind::Candy, FishSpecies::Candyfish),
            (TankKind::Hell, FishSpecies::Cashfish),
            (TankKind::Rad, FishSpecies::Mutantfish),
            (TankKind::Heaven, FishSpecies::Holyfish),
            (TankKind::Matrix, FishSpecies::Botfish),
        ];
        for &kind in TankKind::all() {
            let expected: Vec<FishSpecies> = banners
                .iter()
                .filter(|(banner, _)| *banner == kind)
                .map(|(_, species)| *species)
                .collect();
            assert_eq!(
                FishSpecies::native_to(kind),
                expected,
                "{}",
                kind.display_name()
            );
        }
    }

    #[test]
    fn a_species_the_shop_does_not_sell_lives_in_a_tank_the_shop_does() {
        for &species in ALL_SPECIES {
            if species.config().buyable {
                continue;
            }
            let Habitat::Native(kind) = species.config().habitat else {
                panic!(
                    "{} can be neither bought nor caught",
                    species.display_name()
                );
            };
            let grown = ConsumableKind::seeds()
                .into_iter()
                .any(|seed| seed.summons_tank() == Some(kind));
            assert!(
                kind.config().buyable || grown || kind.config().unique || kind == TankKind::Alien,
                "{} lives in a tank nobody can get",
                species.display_name()
            );
        }
    }

    #[test]
    fn junk_coffee_and_bait_share_one_common_slot_so_each_is_common() {
        assert_eq!(
            JUNK_WEIGHT + COFFEE_WEIGHT + BAIT_WEIGHT,
            Rarity::Common.catch_weight(),
            "the sundries split one Common catch between them"
        );
        for sundry in [StockItem::Junk, StockItem::COFFEE, StockItem::BAIT] {
            assert_eq!(sundry.rarity(), Rarity::Common, "{}", sundry.display_name());
        }
    }

    #[test]
    fn the_robotics_pool_drops_every_part_and_the_wafers_to_print_with() {
        let rolled = rolled_consumables(&LootPool::default_pool().with_robotics());
        for &part in Part::ALL {
            assert!(
                rolled.contains(&ConsumableKind::Part(part)),
                "{} never dropped in {ROLLS} rolls",
                part.display_name()
            );
        }
        assert!(rolled.contains(&ConsumableKind::BlankWafer));
    }

    #[test]
    fn an_ordinary_pool_never_drops_a_part_or_a_wafer() {
        let rolled = rolled_consumables(&LootPool::default_pool());
        assert!(
            !rolled
                .iter()
                .any(|kind| matches!(kind, ConsumableKind::Part(_) | ConsumableKind::BlankWafer)),
            "robotics is Matrixtank loot, not something the sea coughs up"
        );
    }

    #[test]
    fn robotics_stock_resells_at_the_house_ratio_and_never_above_the_bench() {
        for kind in ConsumableKind::bench_stock() {
            assert_eq!(
                kind.sell_price(),
                kind.buy_price() * RESALE_PERCENT / PERCENT,
                "{} resells off the house ratio",
                kind.display_name()
            );
            assert!(kind.sell_price() < kind.buy_price());
        }
    }

    #[test]
    fn every_piece_of_robotics_stock_is_sold_at_the_bench() {
        let bench = ConsumableKind::bench_stock();
        let stock = ConsumableKind::robotics_stock();
        assert_eq!(bench.len(), stock.len());
        for kind in stock {
            assert!(
                bench.contains(&kind),
                "{} is robotics stock the bench never shows",
                kind.display_name()
            );
        }
    }

    #[test]
    fn the_bench_is_laid_out_in_tier_order_and_ends_in_its_materials() {
        let bench = ConsumableKind::bench_stock();
        let tiers: Vec<usize> = bench.iter().map(|kind| kind.bench_tier().index()).collect();
        let mut sorted = tiers.clone();
        sorted.sort_unstable();
        assert_eq!(tiers, sorted, "the bench mixes its tiers");
        assert_eq!(
            bench.last().map(|kind| kind.bench_tier()),
            Some(PartTier::Materials)
        );
    }

    #[test]
    fn everything_on_the_bench_that_is_not_a_part_is_material() {
        for kind in ConsumableKind::bench_stock() {
            let expected = match kind {
                ConsumableKind::Part(part) => part.tier(),
                _ => PartTier::Materials,
            };
            assert_eq!(kind.bench_tier(), expected, "{}", kind.display_name());
        }
    }

    #[test]
    fn the_matrixtank_drops_the_fabricator_as_a_rare_catch() {
        assert_eq!(ConsumableKind::Fabricator.rarity(), Rarity::Rare);
        let rolled = rolled_consumables(&LootPool::default_pool().with_robotics());
        assert!(rolled.contains(&ConsumableKind::Fabricator));
        assert!(
            !rolled_consumables(&LootPool::default_pool()).contains(&ConsumableKind::Fabricator)
        );
    }

    #[test]
    fn a_wafer_is_bulk_stock_and_a_part_is_not() {
        let wafer_weight = ConsumableKind::BlankWafer.rarity().catch_weight();
        for &part in Part::ALL {
            assert!(
                ConsumableKind::Part(part).rarity().catch_weight() <= wafer_weight,
                "{} must not out-drop the raw material",
                part.display_name()
            );
        }
    }

    #[test]
    fn a_wafer_is_on_the_bench_and_resells_at_the_house_ratio() {
        assert_eq!(
            ConsumableKind::BlankWafer.buy_price(),
            BLANK_WAFER_BUY_PRICE
        );
        assert_eq!(ConsumableKind::BlankWafer.sell_price(), 40);
        assert!(ConsumableKind::bench_stock().contains(&ConsumableKind::BlankWafer));
    }

    #[test]
    fn raw_foundry_material_is_blank_and_never_empty() {
        for kind in ConsumableKind::all() {
            assert!(
                !kind.lowercase_name().contains("empty"),
                "{} names blankness with a second word",
                kind.display_name()
            );
        }
        for blank in [ConsumableKind::BlankWafer, ConsumableKind::BlankBlueprint] {
            assert!(
                blank.display_name().starts_with("Blank"),
                "{} is uncut material and must say so",
                blank.display_name()
            );
        }
    }

    #[test]
    fn two_blanks_never_answer_to_one_name() {
        let wafer = ConsumableKind::BlankWafer.lowercase_name();
        let blueprint = ConsumableKind::BlankBlueprint.lowercase_name();
        assert_ne!(wafer, blueprint);
        for name in [&wafer, &blueprint] {
            let matched: Vec<ConsumableKind> = ConsumableKind::all()
                .into_iter()
                .filter(|kind| kind.lowercase_name() == *name)
                .collect();
            assert_eq!(matched.len(), 1, "{name:?} reaches two items");
        }
    }

    #[test]
    fn the_bench_ends_with_foundry_stock_after_the_parts() {
        let stock = ConsumableKind::bench_stock();
        let tail = &stock[Part::ALL.len()..];
        assert!(
            tail == [
                ConsumableKind::BlankWafer,
                ConsumableKind::Fabricator,
                ConsumableKind::BlankBlueprint
            ]
        );
        assert_eq!(ConsumableKind::Fabricator.buy_price(), 400);
        assert_eq!(ConsumableKind::Fabricator.sell_price(), 320);
        assert!(ConsumableKind::Fabricator.can_be_consumed());
    }

    #[test]
    fn a_blank_wafer_is_stock_the_inventory_never_offers_to_consume() {
        assert!(!ConsumableKind::BlankWafer.can_be_consumed());
        assert!(!StockItem::Consumable(ConsumableKind::BlankWafer).is_consumable());
        assert!(StockItem::Consumable(ConsumableKind::Part(Part::InverterCoil)).is_consumable());
        assert!(StockItem::COFFEE.is_consumable());
    }

    fn hooked_arts() -> Vec<(String, HookedArt)> {
        let mut arts: Vec<(String, HookedArt)> = Part::ALL
            .iter()
            .map(|&part| (part.display_name().to_string(), part_art(part)))
            .collect();
        arts.push(("Blank Wafer".to_string(), blank_wafer_art()));
        arts.push(("Computer".to_string(), computer_art()));
        arts.push(("Fabricator".to_string(), fabricator_art()));
        arts
    }

    #[test]
    fn a_fishing_line_never_touches_the_art_it_hangs_beside() {
        for (name, art) in hooked_arts() {
            let hook_col = art.hook_col() as usize;
            for (row, line) in art.lines().iter().enumerate() {
                let width = line.chars().count();
                if row < art.hook_row() as usize {
                    assert!(
                        width < hook_col,
                        "{name} row {row} runs into the line at column {hook_col}"
                    );
                } else if row == art.hook_row() as usize {
                    assert!(width <= hook_col, "{name}'s hook lands inside its art");
                }
            }
        }
    }

    #[test]
    fn a_catch_sprite_is_drawn_from_its_leftmost_ink() {
        for (name, art) in hooked_arts() {
            let indent = art
                .lines()
                .iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.len() - line.trim_start().len())
                .min();
            assert_eq!(indent, Some(0), "{name} keeps a blank margin on its left");
        }
    }

    #[test]
    fn a_trimmed_sprite_keeps_its_shape() {
        let art = part_art(Part::LedgerNerve);
        assert_eq!(art.lines()[0], "$ $  $");
        assert_eq!(art.lines()[1], r" \| / $");
        assert_eq!(
            art.hook_col(),
            8,
            "one clear column after the widest row above the hook"
        );
    }

    #[test]
    fn every_consumable_is_listed_exactly_once() {
        let all = ConsumableKind::all();
        for &part in Part::ALL {
            assert!(all.contains(&ConsumableKind::Part(part)));
        }
        assert!(all.contains(&ConsumableKind::BlankWafer));
        let mut names: Vec<&str> = all
            .iter()
            .map(|k| ConsumableKind::display_name(*k))
            .collect();
        let listed = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), listed, "two consumables share a display name");
    }

    #[test]
    fn a_tank_seed_costs_exactly_the_tank_it_grows_and_resells_at_the_house_rate() {
        let seeds = ConsumableKind::seeds();
        assert!(
            !seeds.is_empty(),
            "the seeds are the only way to a Legendary tank"
        );

        for seed in seeds {
            let tank = seed.summons_tank().expect("filtered on it");
            assert_eq!(
                seed.buy_price(),
                tank.buy_price(),
                "{} must cost what a {} costs, or the seed is a discount on the tank",
                seed.display_name(),
                tank.display_name()
            );
            assert_eq!(
                seed.sell_price(),
                resale_price(seed.buy_price()),
                "{} must resell at the house rate like every other item",
                seed.display_name()
            );
        }
    }
}
