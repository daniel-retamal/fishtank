use ratatui::style::Color;

use crate::economy::{Purchasable, Rarity, Sellable};

pub const EYE_ROUND: char = 'º';
pub const EYE_CIRCLE: char = 'ʘ';
pub const EYE_DEAD: char = 'Ↄ';
pub const TAIL_WAVE_LEFT: char = '彡';
pub const TAIL_WAVE_RIGHT: char = 'ミ';
pub const TAIL_EQUAL: char = '≡';
use crate::colors::{
    AMBER, AMBER_DARK, AMBER_LIGHT, BLUE, CYAN, DARK_GRAY, FOREST, GOLD, GOLD_BRIGHT, GOLD_PALE,
    GRAY, GREEN_BRIGHT, GREEN_LIGHT, LIGHT_BLUE, LIGHT_CYAN, LIGHT_RED, LIGHT_YELLOW, NAVY,
    NAVY_DARK, NAVY_LIGHT, ORANGE, ORANGE_DARK, ORANGE_LIGHT, PURPLE, PURPLE_LIGHT, RED, RED_DARK,
    SILVER, VIOLET, WHITE, YELLOW,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SizeCategory {
    S = 0,
    M = 1,
    L = 2,
    XL = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FishSpecies {
    Merluza,
    Betta,
    Salmon,
    Chromis,
    Tang,
    Koi,
    Carpin,
    Turbofish,
    Deadfish,
    Anchoveta,
    Jellyfish,
    Goldenfish,
    Goldfish,
    Snapper,
    Mutantfish,
    Nishiki,
    Aka,
    Kuro,
    Unfish,
}

impl FishSpecies {
    pub fn display_name(self) -> &'static str {
        self.config().name
    }

    pub fn all_buyable() -> &'static [FishSpecies] {
        ALL_SPECIES
    }

    pub fn buy_price(self) -> u32 {
        self.config().rarity.fish_buy_price()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpeciesConfig {
    pub name: &'static str,
    pub body: BodyTemplate,
    pub palette: &'static [Color],
    pub pattern: PatternKind,
    pub sway_speed: f32,
    pub speed_range: (f32, f32),
    pub rarity: Rarity,
    pub auto_glisten: bool,
    pub auto_mutate: bool,
    pub can_zoomie: bool,
    pub zoomie_vertical: bool,
    pub sizes: [usize; 4],
    pub weight_base: [u32; 4],
    pub weight_cap: [u32; 4],
    pub sell_base: [u32; 4],
    pub sell_cap: [u32; 4],
}

pub const MUTANT_SELL_BASE: u32 = 500;
pub const MUTANT_SELL_PER_MUTATION: u32 = 100;
pub const MUTANT_SELL_WEIGHT_DIVISOR: u32 = 10;

const COMMON_SIZES: [usize; 4] = [3, 5, 7, 9];
const RARE_SIZES: [usize; 4] = [4, 6, 8, 10];
const LEGENDARY_SIZES: [usize; 4] = [5, 7, 9, 11];

const STD_WEIGHT_BASE: [u32; 4] = [100, 250, 500, 1_000];
const STD_WEIGHT_CAP: [u32; 4] = [2_500, 7_500, 20_000, 40_000];

const COMMON_SELL_BASE: [u32; 4] = [12, 27, 65, 175];
const COMMON_SELL_CAP: [u32; 4] = [40, 115, 280, 610];
const RARE_SELL_BASE: [u32; 4] = [30, 75, 175, 500];
const RARE_SELL_CAP: [u32; 4] = [110, 310, 750, 1_850];
const LEGENDARY_SELL_BASE: [u32; 4] = [200, 800, 2_000, 3_500];
const LEGENDARY_SELL_CAP: [u32; 4] = [900, 2_500, 5_000, 7_000];

pub const ALL_SPECIES: &[FishSpecies] = &[
    FishSpecies::Merluza,
    FishSpecies::Betta,
    FishSpecies::Salmon,
    FishSpecies::Chromis,
    FishSpecies::Tang,
    FishSpecies::Koi,
    FishSpecies::Carpin,
    FishSpecies::Turbofish,
    FishSpecies::Deadfish,
    FishSpecies::Anchoveta,
    FishSpecies::Jellyfish,
    FishSpecies::Goldenfish,
    FishSpecies::Goldfish,
    FishSpecies::Snapper,
    FishSpecies::Mutantfish,
    FishSpecies::Nishiki,
    FishSpecies::Aka,
    FishSpecies::Kuro,
];

#[derive(Debug, Clone, Copy)]
pub enum BodyTemplate {
    Standard(BodyChars),
    Alternating(BodyChars, BodyChars),
    Fixed {
        left: &'static [&'static str],
        right: &'static [&'static str],
    },
}

#[derive(Debug, Clone, Copy)]
pub struct BodyPair {
    pub left_body: char,
    pub left_wave: char,
    pub right_body: char,
    pub right_wave: char,
}

pub const BODY_ROUND: BodyPair = BodyPair {
    left_body: '(',
    left_wave: '{',
    right_body: ')',
    right_wave: '}',
};
pub const BODY_CURLY: BodyPair = BodyPair {
    left_body: '}',
    left_wave: ')',
    right_body: '{',
    right_wave: '(',
};

#[derive(Debug, Clone, Copy)]
pub struct BodyChars {
    pub mouth_left: char,
    pub mouth_right: char,
    pub eye_left: char,
    pub eye_right: char,
    pub body_left: char,
    pub wave_left: char,
    pub body_right: char,
    pub wave_right: char,
    pub tail: TailKind,
}

#[derive(Debug, Clone, Copy)]
pub enum TailKind {
    Wide,
    Short,
    Custom { left: char, right: char },
    Swaying { left: char, right: char, wave: char },
    WideCurly,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum PatternKind {
    Solid,
    Striped,
    Patchy,
    PatchyAll,
    Glistening,
}

const fn standard_with(eye: char, tail: TailKind, body: BodyPair) -> BodyChars {
    BodyChars {
        mouth_left: '<',
        mouth_right: '>',
        eye_left: eye,
        eye_right: eye,
        body_left: body.left_body,
        wave_left: body.left_wave,
        body_right: body.right_body,
        wave_right: body.right_wave,
        tail,
    }
}

const fn standard(eye: char, tail: TailKind) -> BodyChars {
    standard_with(eye, tail, BODY_ROUND)
}

static MERLUZA_PALETTE: [Color; 2] = [GRAY, WHITE];

static BETTA_PALETTE: [Color; 3] = [BLUE, LIGHT_BLUE, CYAN];
static TANG_PALETTE: [Color; 3] = [NAVY_DARK, NAVY, NAVY_LIGHT];
static JELLYFISH_PALETTE: [Color; 3] = [BLUE, CYAN, LIGHT_BLUE];

static CHROMIS_PALETTE: [Color; 3] = [CYAN, LIGHT_CYAN, WHITE];
static ANCHOVETA_PALETTE: [Color; 2] = [CYAN, LIGHT_CYAN];

static SALMON_PALETTE: [Color; 3] = [ORANGE_DARK, ORANGE, ORANGE_LIGHT];
static GOLDFISH_PALETTE: [Color; 3] = [ORANGE, AMBER, AMBER_LIGHT];
static SNAPPER_PALETTE: [Color; 3] = [RED_DARK, RED, LIGHT_RED];
static TURBOFISH_PALETTE: [Color; 2] = [YELLOW, ORANGE];

static GOLDENFISH_PALETTE: [Color; 3] = [GOLD, GOLD_PALE, GOLD_BRIGHT];

static AKA_PALETTE: [Color; 1] = [RED];
static KURO_PALETTE: [Color; 1] = [DARK_GRAY];
static NISHIKI_PALETTE: [Color; 3] = [WHITE, LIGHT_RED, DARK_GRAY];

static KOI_PALETTE: [Color; 5] = [WHITE, WHITE, WHITE, LIGHT_RED, DARK_GRAY];

static CARPIN_PALETTE: [Color; 3] = [YELLOW, LIGHT_YELLOW, AMBER_DARK];

static DEADFISH_PALETTE: [Color; 2] = [WHITE, DARK_GRAY];
static UNFISH_PALETTE: [Color; 1] = [WHITE];

pub const DEADFISH_BC_SEMI: BodyChars = BodyChars {
    mouth_left: '<',
    mouth_right: '>',
    eye_left: EYE_DEAD,
    eye_right: 'C',
    body_left: ';',
    wave_left: ':',
    body_right: ';',
    wave_right: ':',
    tail: TailKind::Wide,
};

pub const DEADFISH_BC_PLUS: BodyChars = BodyChars {
    mouth_left: '<',
    mouth_right: '>',
    eye_left: EYE_DEAD,
    eye_right: 'C',
    body_left: '+',
    wave_left: '-',
    body_right: '+',
    wave_right: '-',
    tail: TailKind::Wide,
};

pub static MUTANT_GREEN_PALETTE: [Color; 3] = [FOREST, GREEN_LIGHT, GREEN_BRIGHT];
pub static MUTANT_PURPLE_PALETTE: [Color; 3] = [PURPLE_LIGHT, PURPLE, VIOLET];
pub static MUTANT_WHITE_PALETTE: [Color; 3] = [GRAY, SILVER, WHITE];

static ANCHOVETA_L: [&str; 1] = ["<><"];
static ANCHOVETA_R: [&str; 1] = ["><>"];
static JELLYFISH_LR: [&str; 1] = ["ള"];

fn rarity_arrays(rarity: Rarity) -> ([usize; 4], [u32; 4], [u32; 4]) {
    match rarity {
        Rarity::Common => (COMMON_SIZES, COMMON_SELL_BASE, COMMON_SELL_CAP),
        Rarity::Rare => (RARE_SIZES, RARE_SELL_BASE, RARE_SELL_CAP),
        Rarity::Legendary => (LEGENDARY_SIZES, LEGENDARY_SELL_BASE, LEGENDARY_SELL_CAP),
    }
}

fn standard_config(
    name: &'static str,
    body: BodyChars,
    palette: &'static [Color],
    pattern: PatternKind,
    sway_speed: f32,
    speed_range: (f32, f32),
    rarity: Rarity,
) -> SpeciesConfig {
    let (sizes, sell_base, sell_cap) = rarity_arrays(rarity);
    SpeciesConfig {
        name,
        body: BodyTemplate::Standard(body),
        palette,
        pattern,
        sway_speed,
        speed_range,
        rarity,
        auto_glisten: false,
        auto_mutate: false,
        can_zoomie: true,
        zoomie_vertical: false,
        sizes,
        weight_base: STD_WEIGHT_BASE,
        weight_cap: STD_WEIGHT_CAP,
        sell_base,
        sell_cap,
    }
}

fn fixed_config(
    name: &'static str,
    left: &'static [&'static str],
    right: &'static [&'static str],
    palette: &'static [Color],
    pattern: PatternKind,
    speed_range: (f32, f32),
    rarity: Rarity,
) -> SpeciesConfig {
    let (sizes, sell_base, sell_cap) = rarity_arrays(rarity);
    SpeciesConfig {
        name,
        body: BodyTemplate::Fixed { left, right },
        palette,
        pattern,
        sway_speed: 0.0,
        speed_range,
        rarity,
        auto_glisten: false,
        auto_mutate: false,
        can_zoomie: true,
        zoomie_vertical: false,
        sizes,
        weight_base: STD_WEIGHT_BASE,
        weight_cap: STD_WEIGHT_CAP,
        sell_base,
        sell_cap,
    }
}

impl FishSpecies {
    pub fn config(self) -> SpeciesConfig {
        use FishSpecies::*;
        use PatternKind::*;
        use Rarity::*;
        match self {
            Merluza => standard_config(
                "Merluza",
                standard(EYE_ROUND, TailKind::Wide),
                &MERLUZA_PALETTE,
                Solid,
                0.10,
                (2.5, 4.0),
                Common,
            ),
            Betta => standard_config(
                "Betta",
                standard('\'', TailKind::Wide),
                &BETTA_PALETTE,
                Striped,
                0.09,
                (2.0, 3.5),
                Common,
            ),
            Salmon => standard_config(
                "Salmon",
                standard('*', TailKind::Wide),
                &SALMON_PALETTE,
                Striped,
                0.10,
                (4.0, 6.0),
                Common,
            ),
            Chromis => standard_config(
                "Chromis",
                standard(EYE_ROUND, TailKind::Short),
                &CHROMIS_PALETTE,
                Striped,
                0.13,
                (3.0, 5.0),
                Common,
            ),
            Tang => standard_config(
                "Tang",
                standard('\'', TailKind::Wide),
                &TANG_PALETTE,
                Striped,
                0.09,
                (2.5, 4.0),
                Common,
            ),
            Goldfish => standard_config(
                "Goldfish",
                standard(EYE_ROUND, TailKind::WideCurly),
                &GOLDFISH_PALETTE,
                Striped,
                0.08,
                (1.5, 3.0),
                Common,
            ),
            Snapper => standard_config(
                "Snapper",
                standard(EYE_ROUND, TailKind::Wide),
                &SNAPPER_PALETTE,
                Striped,
                0.10,
                (2.5, 4.0),
                Common,
            ),
            Nishiki => standard_config(
                "Nishiki",
                standard(EYE_ROUND, TailKind::WideCurly),
                &NISHIKI_PALETTE,
                PatchyAll,
                0.08,
                (1.5, 3.0),
                Common,
            ),
            Aka => standard_config(
                "Aka",
                standard(EYE_ROUND, TailKind::WideCurly),
                &AKA_PALETTE,
                Solid,
                0.09,
                (2.0, 3.5),
                Common,
            ),
            Kuro => standard_config(
                "Kuro",
                standard(EYE_ROUND, TailKind::WideCurly),
                &KURO_PALETTE,
                Solid,
                0.07,
                (1.5, 3.0),
                Common,
            ),
            Deadfish => {
                let (sizes, sell_base, sell_cap) = rarity_arrays(Rare);
                SpeciesConfig {
                    name: "Deadfish",
                    body: BodyTemplate::Alternating(DEADFISH_BC_SEMI, DEADFISH_BC_PLUS),
                    palette: &DEADFISH_PALETTE,
                    pattern: Solid,
                    sway_speed: 0.04,
                    speed_range: (1.0, 2.5),
                    rarity: Rare,
                    auto_glisten: false,
                    auto_mutate: false,
                    can_zoomie: true,
                    zoomie_vertical: false,
                    sizes,
                    weight_base: STD_WEIGHT_BASE,
                    weight_cap: STD_WEIGHT_CAP,
                    sell_base,
                    sell_cap,
                }
            }
            Anchoveta => fixed_config(
                "Anchoveta",
                &ANCHOVETA_L,
                &ANCHOVETA_R,
                &ANCHOVETA_PALETTE,
                Solid,
                (5.0, 7.0),
                Common,
            ),
            Jellyfish => {
                let mut config = fixed_config(
                    "Jellyfish",
                    &JELLYFISH_LR,
                    &JELLYFISH_LR,
                    &JELLYFISH_PALETTE,
                    Solid,
                    (1.5, 3.0),
                    Rare,
                );
                config.zoomie_vertical = true;
                config
            }
            Turbofish => standard_config(
                "Turbofish",
                BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: '>',
                    eye_right: '<',
                    body_left: ':',
                    wave_left: '~',
                    body_right: ':',
                    wave_right: '~',
                    tail: TailKind::None,
                },
                &TURBOFISH_PALETTE,
                Striped,
                0.18,
                (6.0, 9.0),
                Rare,
            ),
            Koi => standard_config(
                "Koi",
                BodyChars {
                    mouth_left: '>',
                    mouth_right: '<',
                    eye_left: EYE_ROUND,
                    eye_right: EYE_ROUND,
                    body_left: BODY_CURLY.left_body,
                    wave_left: BODY_CURLY.left_wave,
                    body_right: BODY_CURLY.right_body,
                    wave_right: BODY_CURLY.right_wave,
                    tail: TailKind::Swaying {
                        left: TAIL_WAVE_LEFT,
                        right: TAIL_WAVE_RIGHT,
                        wave: TAIL_EQUAL,
                    },
                },
                &KOI_PALETTE,
                Patchy,
                0.07,
                (1.5, 3.0),
                Rare,
            ),
            Carpin => standard_config(
                "Carpin",
                standard_with(
                    EYE_CIRCLE,
                    TailKind::Custom {
                        left: '(',
                        right: ')',
                    },
                    BODY_CURLY,
                ),
                &CARPIN_PALETTE,
                Patchy,
                0.09,
                (2.0, 3.5),
                Common,
            ),
            Goldenfish => standard_config(
                "Goldenfish",
                BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: EYE_ROUND,
                    eye_right: EYE_ROUND,
                    body_left: '(',
                    wave_left: '(',
                    body_right: ')',
                    wave_right: ')',
                    tail: TailKind::Wide,
                },
                &GOLDENFISH_PALETTE,
                Glistening,
                0.20,
                (2.0, 3.5),
                Legendary,
            ),
            Mutantfish => SpeciesConfig {
                name: "Mutantfish",
                body: BodyTemplate::Standard(standard(EYE_CIRCLE, TailKind::Wide)),
                palette: &MUTANT_GREEN_PALETTE,
                pattern: Solid,
                sway_speed: 0.11,
                speed_range: (2.0, 5.0),
                rarity: Legendary,
                auto_glisten: true,
                auto_mutate: true,
                can_zoomie: true,
                zoomie_vertical: false,
                sizes: LEGENDARY_SIZES,
                weight_base: [0, 250, 0, 0],
                weight_cap: [0; 4],
                sell_base: [0; 4],
                sell_cap: [0; 4],
            },
            Unfish => SpeciesConfig {
                name: "Unfish",
                body: BodyTemplate::Standard(standard(EYE_ROUND, TailKind::Wide)),
                palette: &UNFISH_PALETTE,
                pattern: Solid,
                sway_speed: 0.10,
                speed_range: (2.0, 4.0),
                rarity: Common,
                auto_glisten: false,
                auto_mutate: false,
                can_zoomie: false,
                zoomie_vertical: false,
                sizes: COMMON_SIZES,
                weight_base: [1; 4],
                weight_cap: [0; 4],
                sell_base: [0; 4],
                sell_cap: [0; 4],
            },
        }
    }

    pub fn sell_value(self, weight_g: u32, size_cat: SizeCategory, mutation_count: u32) -> u32 {
        if self == FishSpecies::Unfish {
            return 0;
        }
        if self == FishSpecies::Mutantfish {
            return MUTANT_SELL_BASE
                + mutation_count * MUTANT_SELL_PER_MUTATION
                + weight_g / MUTANT_SELL_WEIGHT_DIVISOR;
        }
        let config = self.config();
        let i = size_cat as usize;
        let (weight_base, weight_cap, sell_base, sell_cap) = (
            config.weight_base[i],
            config.weight_cap[i],
            config.sell_base[i],
            config.sell_cap[i],
        );
        if weight_cap == 0 || weight_g <= weight_base {
            return sell_base;
        }
        let frac = (weight_g - weight_base).min(weight_cap - weight_base) as f32
            / (weight_cap - weight_base) as f32;
        sell_base + (frac * (sell_cap - sell_base) as f32) as u32
    }

    pub fn mutant_color_for_seed(seed: u64) -> Color {
        let family: &[Color] = match (seed >> 6) % 3 {
            0 => &MUTANT_GREEN_PALETTE,
            1 => &MUTANT_PURPLE_PALETTE,
            _ => &MUTANT_WHITE_PALETTE,
        };
        family[(seed >> 8) as usize % family.len()]
    }

    pub fn parse(s: &str) -> Option<Self> {
        let lower = s.to_ascii_lowercase();
        ALL_SPECIES
            .iter()
            .find(|&&sp| sp.config().name.to_ascii_lowercase() == lower)
            .copied()
    }
}

impl Purchasable for FishSpecies {
    fn buy_price(&self) -> u32 {
        self.config().rarity.fish_buy_price()
    }
    fn display_name(&self) -> &str {
        self.config().name
    }
}

impl Sellable for FishSpecies {
    fn sell_price(&self) -> u32 {
        self.config().sell_base[0]
    }
    fn display_name(&self) -> &str {
        self.config().name
    }
}
