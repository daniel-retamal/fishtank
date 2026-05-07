use ratatui::style::Color;

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
        match self {
            FishSpecies::Merluza => "Merluza",
            FishSpecies::Betta => "Betta",
            FishSpecies::Salmon => "Salmon",
            FishSpecies::Chromis => "Chromis",
            FishSpecies::Tang => "Tang",
            FishSpecies::Koi => "Koi",
            FishSpecies::Carpin => "Carpin",
            FishSpecies::Turbofish => "Turbofish",
            FishSpecies::Deadfish => "Deadfish",
            FishSpecies::Anchoveta => "Anchoveta",
            FishSpecies::Jellyfish => "Jellyfish",
            FishSpecies::Goldenfish => "Goldenfish",
            FishSpecies::Goldfish => "Goldfish",
            FishSpecies::Snapper => "Snapper",
            FishSpecies::Mutantfish => "Mutantfish",
            FishSpecies::Nishiki => "Nishiki",
            FishSpecies::Aka => "Aka",
            FishSpecies::Kuro => "Kuro",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpeciesConfig {
    pub body: BodyTemplate,
    pub palette: &'static [Color],
    pub pattern: PatternKind,
    pub sway_speed: f32,
    pub size_range: (usize, usize),
    pub speed_range: (f32, f32),
}

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
                body: BodyTemplate::Standard(standard('º', TailKind::Wide)),
                palette: &PAL_MERLUZA,
                pattern: PatternKind::Solid,
                sway_speed: 0.10,
                size_range: (3, 5),
                speed_range: (2.5, 4.0),
            },
            FishSpecies::Betta => SpeciesConfig {
                body: BodyTemplate::Standard(standard('\'', TailKind::Wide)),
                palette: &PAL_BETTA,
                pattern: PatternKind::Striped,
                sway_speed: 0.09,
                size_range: (4, 6),
                speed_range: (2.0, 3.5),
            },
            FishSpecies::Salmon => SpeciesConfig {
                body: BodyTemplate::Standard(standard('*', TailKind::Wide)),
                palette: &PAL_SALMON,
                pattern: PatternKind::Striped,
                sway_speed: 0.10,
                size_range: (5, 6),
                speed_range: (4.0, 6.0),
            },
            FishSpecies::Chromis => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::Short)),
                palette: &PAL_CHROMIS,
                pattern: PatternKind::Striped,
                sway_speed: 0.13,
                size_range: (2, 4),
                speed_range: (3.0, 5.0),
            },
            FishSpecies::Tang => SpeciesConfig {
                body: BodyTemplate::Standard(standard('\'', TailKind::Wide)),
                palette: &PAL_TANG,
                pattern: PatternKind::Striped,
                sway_speed: 0.09,
                size_range: (4, 6),
                speed_range: (2.5, 4.0),
            },
            FishSpecies::Koi => SpeciesConfig {
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
                size_range: (3, 5),
                speed_range: (1.5, 3.0),
            },
            FishSpecies::Carpin => SpeciesConfig {
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
                size_range: (2, 4),
                speed_range: (2.0, 3.5),
            },
            FishSpecies::Turbofish => SpeciesConfig {
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
                size_range: (2, 2),
                speed_range: (6.0, 9.0),
            },
            FishSpecies::Deadfish => SpeciesConfig {
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
                size_range: (3, 6),
                speed_range: (1.0, 2.5),
            },
            FishSpecies::Anchoveta => SpeciesConfig {
                body: BodyTemplate::Fixed {
                    left: &ANCHOVETA_L,
                    right: &ANCHOVETA_R,
                },
                palette: &PAL_ANCHOVETA,
                pattern: PatternKind::Solid,
                sway_speed: 0.0,
                size_range: (0, 0),
                speed_range: (5.0, 7.0),
            },
            FishSpecies::Jellyfish => SpeciesConfig {
                body: BodyTemplate::Fixed {
                    left: &JELLYFISH_LR,
                    right: &JELLYFISH_LR,
                },
                palette: &PAL_JELLYFISH,
                pattern: PatternKind::Solid,
                sway_speed: 0.0,
                size_range: (0, 0),
                speed_range: (1.5, 3.0),
            },
            FishSpecies::Goldenfish => SpeciesConfig {
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
                size_range: (3, 5),
                speed_range: (2.0, 3.5),
            },
            FishSpecies::Goldfish => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_GOLDFISH,
                pattern: PatternKind::Striped,
                sway_speed: 0.08,
                size_range: (2, 3),
                speed_range: (1.5, 3.0),
            },
            FishSpecies::Snapper => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::Wide)),
                palette: &PAL_SNAPPER,
                pattern: PatternKind::Striped,
                sway_speed: 0.10,
                size_range: (3, 5),
                speed_range: (2.5, 4.0),
            },
            FishSpecies::Nishiki => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_NISHIKI,
                pattern: PatternKind::PatchyAll,
                sway_speed: 0.08,
                size_range: (2, 3),
                speed_range: (1.5, 3.0),
            },
            FishSpecies::Aka => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_AKA,
                pattern: PatternKind::Solid,
                sway_speed: 0.09,
                size_range: (2, 3),
                speed_range: (2.0, 3.5),
            },
            FishSpecies::Kuro => SpeciesConfig {
                body: BodyTemplate::Standard(standard('º', TailKind::WideCurly)),
                palette: &PAL_KURO,
                pattern: PatternKind::Solid,
                sway_speed: 0.07,
                size_range: (2, 3),
                speed_range: (1.5, 3.0),
            },
            FishSpecies::Mutantfish => SpeciesConfig {
                body: BodyTemplate::Standard(standard('ʘ', TailKind::Wide)),
                palette: &PAL_MUTANT_GREEN,
                pattern: PatternKind::Solid,
                sway_speed: 0.11,
                size_range: (2, 8),
                speed_range: (2.0, 5.0),
            },
        }
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
        match s.to_ascii_lowercase().as_str() {
            "merluza" => Some(FishSpecies::Merluza),
            "betta" => Some(FishSpecies::Betta),
            "salmon" => Some(FishSpecies::Salmon),
            "chromis" => Some(FishSpecies::Chromis),
            "tang" => Some(FishSpecies::Tang),
            "koi" => Some(FishSpecies::Koi),
            "carpin" => Some(FishSpecies::Carpin),
            "turbofish" => Some(FishSpecies::Turbofish),
            "deadfish" => Some(FishSpecies::Deadfish),
            "anchoveta" => Some(FishSpecies::Anchoveta),
            "jellyfish" => Some(FishSpecies::Jellyfish),
            "goldenfish" => Some(FishSpecies::Goldenfish),
            "goldfish" => Some(FishSpecies::Goldfish),
            "snapper" => Some(FishSpecies::Snapper),
            "mutantfish" => Some(FishSpecies::Mutantfish),
            "nishiki" => Some(FishSpecies::Nishiki),
            "aka" => Some(FishSpecies::Aka),
            "kuro" => Some(FishSpecies::Kuro),
            _ => None,
        }
    }
}
