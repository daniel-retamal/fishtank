use ratatui::style::Color;

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
}

impl FishSpecies {
    pub fn display_name(self) -> &'static str {
        self.config().name
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
    Fixed {
        left: &'static [&'static str],
        right: &'static [&'static str],
    },
}

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

const fn standard(eye: char, tail: TailKind) -> BodyChars {
    BodyChars {
        mouth_left: '<',
        mouth_right: '>',
        eye_left: eye,
        eye_right: eye,
        body_left: '(',
        wave_left: '{',
        body_right: ')',
        wave_right: '}',
        tail,
    }
}

static PAL_MERLUZA: [Color; 2] = [Color::Gray, Color::White];

static PAL_BETTA: [Color; 3] = [Color::Blue, Color::LightBlue, Color::Cyan];
static PAL_TANG: [Color; 3] = [
    Color::Rgb(20, 60, 200),
    Color::Rgb(60, 110, 240),
    Color::Rgb(100, 160, 255),
];
static PAL_JELLYFISH: [Color; 3] = [Color::Blue, Color::Cyan, Color::LightBlue];

static PAL_CHROMIS: [Color; 3] = [Color::Cyan, Color::LightCyan, Color::White];
static PAL_ANCHOVETA: [Color; 2] = [Color::Cyan, Color::LightCyan];

static PAL_SALMON: [Color; 3] = [
    Color::Rgb(210, 60, 0),
    Color::Rgb(255, 110, 40),
    Color::Rgb(255, 170, 90),
];
static PAL_GOLDFISH: [Color; 3] = [
    Color::Rgb(255, 120, 0),
    Color::Rgb(255, 170, 50),
    Color::Rgb(255, 210, 110),
];
static PAL_SNAPPER: [Color; 3] = [Color::Red, Color::LightRed, Color::Rgb(220, 50, 50)];
static PAL_TURBOFISH: [Color; 2] = [Color::Yellow, Color::Rgb(255, 140, 0)];

static PAL_GOLDENFISH: [Color; 3] = [
    Color::Rgb(255, 210, 0),
    Color::Rgb(255, 240, 150),
    Color::Rgb(255, 255, 100),
];

static PAL_AKA: [Color; 1] = [Color::Red];

static PAL_KURO: [Color; 1] = [Color::DarkGray];

static PAL_NISHIKI: [Color; 3] = [Color::White, Color::LightRed, Color::DarkGray];

static PAL_KOI: [Color; 5] = [
    Color::White,
    Color::White,
    Color::White,
    Color::LightRed,
    Color::DarkGray,
];

static PAL_CARPIN: [Color; 3] = [Color::Yellow, Color::LightYellow, Color::Rgb(200, 130, 0)];

static PAL_DEADFISH: [Color; 2] = [Color::White, Color::DarkGray];

pub const DEADFISH_BC_SEMI: BodyChars = BodyChars {
    mouth_left: '<',
    mouth_right: '>',
    eye_left: 'Ↄ',
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
    eye_left: 'Ↄ',
    eye_right: 'C',
    body_left: '+',
    wave_left: '-',
    body_right: '+',
    wave_right: '-',
    tail: TailKind::Wide,
};

pub static PAL_MUTANT_GREEN: [Color; 3] = [
    Color::Rgb(0, 180, 60),
    Color::Rgb(0, 130, 40),
    Color::Rgb(60, 220, 100),
];
pub static PAL_MUTANT_PURPLE: [Color; 3] = [
    Color::Rgb(130, 0, 190),
    Color::Rgb(90, 0, 145),
    Color::Rgb(165, 65, 230),
];
pub static PAL_MUTANT_WHITE: [Color; 3] = [
    Color::Rgb(180, 180, 180),
    Color::Rgb(220, 220, 220),
    Color::Rgb(255, 255, 255),
];

static ANCHOVETA_L: [&str; 1] = ["<><"];
static ANCHOVETA_R: [&str; 1] = ["><>"];
static JELLYFISH_LR: [&str; 1] = ["ଳ"];

impl FishSpecies {
    pub fn config(self) -> SpeciesConfig {
        match self {
            FishSpecies::Merluza => SpeciesConfig {
                name: "Merluza",
                body: BodyTemplate::Standard(standard('º', TailKind::Wide)),
                palette: &PAL_MERLUZA,
                pattern: PatternKind::Solid,
                sway_speed: 0.10,
                speed_range: (2.5, 4.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Betta => SpeciesConfig {
                name: "Betta",
                body: BodyTemplate::Standard(standard('\'', TailKind::Wide)),
                palette: &PAL_BETTA,
                pattern: PatternKind::Striped,
                sway_speed: 0.09,
                speed_range: (2.0, 3.5),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Salmon => SpeciesConfig {
                name: "Salmon",
                body: BodyTemplate::Standard(standard('*', TailKind::Wide)),
                palette: &PAL_SALMON,
                pattern: PatternKind::Striped,
                sway_speed: 0.10,
                speed_range: (4.0, 6.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Chromis => SpeciesConfig {
                name: "Chromis",
                body: BodyTemplate::Standard(standard('º', TailKind::Short)),
                palette: &PAL_CHROMIS,
                pattern: PatternKind::Striped,
                sway_speed: 0.13,
                speed_range: (3.0, 5.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Tang => SpeciesConfig {
                name: "Tang",
                body: BodyTemplate::Standard(standard('\'', TailKind::Wide)),
                palette: &PAL_TANG,
                pattern: PatternKind::Striped,
                sway_speed: 0.09,
                speed_range: (2.5, 4.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Koi => SpeciesConfig {
                name: "Koi",
                body: BodyTemplate::Standard(BodyChars {
                    mouth_left: '>',
                    mouth_right: '<',
                    eye_left: 'º',
                    eye_right: 'º',
                    body_left: '}',
                    wave_left: ')',
                    body_right: '{',
                    wave_right: '(',
                    tail: TailKind::Swaying {
                        left: '彡',
                        right: 'ミ',
                        wave: '≡',
                    },
                }),
                palette: &PAL_KOI,
                pattern: PatternKind::Patchy,
                sway_speed: 0.07,
                speed_range: (1.5, 3.0),
                sizes: RARE_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: RARE_SELL_BASE,
                sell_cap: RARE_SELL_CAP,
            },
            FishSpecies::Carpin => SpeciesConfig {
                name: "Carpin",
                body: BodyTemplate::Standard(BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: 'ʘ',
                    eye_right: 'ʘ',
                    body_left: '}',
                    wave_left: ')',
                    body_right: '{',
                    wave_right: '(',
                    tail: TailKind::Custom {
                        left: '(',
                        right: ')',
                    },
                }),
                palette: &PAL_CARPIN,
                pattern: PatternKind::Patchy,
                sway_speed: 0.09,
                speed_range: (2.0, 3.5),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Turbofish => SpeciesConfig {
                name: "Turbofish",
                body: BodyTemplate::Standard(BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: '>',
                    eye_right: '<',
                    body_left: ':',
                    wave_left: '~',
                    body_right: ':',
                    wave_right: '~',
                    tail: TailKind::None,
                }),
                palette: &PAL_TURBOFISH,
                pattern: PatternKind::Striped,
                sway_speed: 0.18,
                speed_range: (6.0, 9.0),
                sizes: RARE_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: RARE_SELL_BASE,
                sell_cap: RARE_SELL_CAP,
            },
            FishSpecies::Deadfish => SpeciesConfig {
                name: "Deadfish",
                body: BodyTemplate::Standard(BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: 'Ↄ',
                    eye_right: 'C',
                    body_left: ';',
                    wave_left: ':',
                    body_right: ';',
                    wave_right: ':',
                    tail: TailKind::Wide,
                }),
                palette: &PAL_DEADFISH,
                pattern: PatternKind::Solid,
                sway_speed: 0.04,
                speed_range: (1.0, 2.5),
                sizes: RARE_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: RARE_SELL_BASE,
                sell_cap: RARE_SELL_CAP,
            },
            FishSpecies::Anchoveta => SpeciesConfig {
                name: "Anchoveta",
                body: BodyTemplate::Fixed {
                    left: &ANCHOVETA_L,
                    right: &ANCHOVETA_R,
                },
                palette: &PAL_ANCHOVETA,
                pattern: PatternKind::Solid,
                sway_speed: 0.0,
                speed_range: (5.0, 7.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Jellyfish => SpeciesConfig {
                name: "Jellyfish",
                body: BodyTemplate::Fixed {
                    left: &JELLYFISH_LR,
                    right: &JELLYFISH_LR,
                },
                palette: &PAL_JELLYFISH,
                pattern: PatternKind::Solid,
                sway_speed: 0.0,
                speed_range: (1.5, 3.0),
                sizes: RARE_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: RARE_SELL_BASE,
                sell_cap: RARE_SELL_CAP,
            },
            FishSpecies::Goldenfish => SpeciesConfig {
                name: "Goldenfish",
                body: BodyTemplate::Standard(BodyChars {
                    mouth_left: '<',
                    mouth_right: '>',
                    eye_left: 'º',
                    eye_right: 'º',
                    body_left: '(',
                    wave_left: '(',
                    body_right: ')',
                    wave_right: ')',
                    tail: TailKind::Wide,
                }),
                palette: &PAL_GOLDENFISH,
                pattern: PatternKind::Glistening,
                sway_speed: 0.20,
                speed_range: (2.0, 3.5),
                sizes: LEGENDARY_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: [200, 800, 2_000, 3_500],
                sell_cap: [900, 2_500, 5_000, 7_000],
            },
            FishSpecies::Goldfish => SpeciesConfig {
                name: "Goldfish",
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_GOLDFISH,
                pattern: PatternKind::Striped,
                sway_speed: 0.08,
                speed_range: (1.5, 3.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Snapper => SpeciesConfig {
                name: "Snapper",
                body: BodyTemplate::Standard(standard('º', TailKind::Wide)),
                palette: &PAL_SNAPPER,
                pattern: PatternKind::Striped,
                sway_speed: 0.10,
                speed_range: (2.5, 4.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Nishiki => SpeciesConfig {
                name: "Nishiki",
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_NISHIKI,
                pattern: PatternKind::PatchyAll,
                sway_speed: 0.08,
                speed_range: (1.5, 3.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Aka => SpeciesConfig {
                name: "Aka",
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_AKA,
                pattern: PatternKind::Solid,
                sway_speed: 0.09,
                speed_range: (2.0, 3.5),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Kuro => SpeciesConfig {
                name: "Kuro",
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_KURO,
                pattern: PatternKind::Solid,
                sway_speed: 0.07,
                speed_range: (1.5, 3.0),
                sizes: COMMON_SIZES,
                weight_base: STD_WEIGHT_BASE,
                weight_cap: STD_WEIGHT_CAP,
                sell_base: COMMON_SELL_BASE,
                sell_cap: COMMON_SELL_CAP,
            },
            FishSpecies::Mutantfish => SpeciesConfig {
                name: "Mutantfish",
                body: BodyTemplate::Standard(standard('ʘ', TailKind::Wide)),
                palette: &PAL_MUTANT_GREEN,
                pattern: PatternKind::Solid,
                sway_speed: 0.11,
                speed_range: (2.0, 5.0),
                sizes: LEGENDARY_SIZES,
                weight_base: [0, 250, 0, 0],
                weight_cap: [0; 4],
                sell_base: [0; 4],
                sell_cap: [0; 4],
            },
        }
    }

    pub fn sell_value(self, weight_g: u32, size_cat: SizeCategory, mutation_count: u32) -> u32 {
        if self == FishSpecies::Mutantfish {
            return MUTANT_SELL_BASE
                + mutation_count * MUTANT_SELL_PER_MUTATION
                + weight_g / MUTANT_SELL_WEIGHT_DIVISOR;
        }
        let cfg = self.config();
        let i = size_cat as usize;
        let (wb, wc, sb, sc) = (
            cfg.weight_base[i],
            cfg.weight_cap[i],
            cfg.sell_base[i],
            cfg.sell_cap[i],
        );
        if wc == 0 || weight_g <= wb {
            return sb;
        }
        let frac = (weight_g - wb).min(wc - wb) as f32 / (wc - wb) as f32;
        sb + (frac * (sc - sb) as f32) as u32
    }

    pub fn mutant_color_for_seed(seed: u64) -> Color {
        let family: &[Color] = match (seed >> 6) % 3 {
            0 => &PAL_MUTANT_GREEN,
            1 => &PAL_MUTANT_PURPLE,
            _ => &PAL_MUTANT_WHITE,
        };
        family[(seed >> 8) as usize % family.len()]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_ascii_lowercase();
        ALL_SPECIES
            .iter()
            .find(|&&sp| sp.config().name.to_ascii_lowercase() == lower)
            .copied()
    }
}
