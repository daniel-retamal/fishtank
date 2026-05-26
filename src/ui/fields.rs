use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{FIELD_AIR_COLOR, FIELD_EARTH_COLOR, FIELD_FIRE_COLOR, FIELD_SPIRIT_COLOR, FIELD_WATER_COLOR, GOLD};
use crate::fishes::fish::Fish;
use crate::fishes::species::FishSpecies;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Iq,
    ZodiacSign,
    ChineseZodiac,
    Tanganana,
    HappinessLevel,
    Claustrophobic,
    AttrText,
    AttrEnumBar,
    AttrBoolean,
    PickACard,
    FavoriteColor,
    FavoriteLetter,
    FavoriteNumber,
    TarotPrediction,
    FavoriteQuote,
    FavoriteHour,
    Lonely,
    Gmi,
    ElOLaPr,
    TheOrThePr,
    Crush,
    MarriedTo,
    Hates,
    Status,
    Sin,
    HasSeenTheSky,
    Temperature,
    Delicious,
    Region,
    Dni,
    FavoriteSeason,
}

pub struct FieldValue {
    pub text: String,
    pub swatch: Option<Color>,
}

impl FieldKind {
    pub fn all() -> &'static [FieldKind] {
        use FieldKind::*;
        &[
            Iq,
            ZodiacSign,
            ChineseZodiac,
            Tanganana,
            HappinessLevel,
            Claustrophobic,
            AttrText,
            AttrEnumBar,
            AttrBoolean,
            PickACard,
            FavoriteColor,
            FavoriteLetter,
            FavoriteNumber,
            TarotPrediction,
            FavoriteQuote,
            FavoriteHour,
            Lonely,
            Gmi,
            ElOLaPr,
            TheOrThePr,
            Crush,
            MarriedTo,
            Hates,
            Status,
            Sin,
            HasSeenTheSky,
            Temperature,
            Delicious,
            Region,
            Dni,
            FavoriteSeason,
        ]
    }

    pub fn header(self) -> &'static str {
        match self {
            FieldKind::Iq => "IQ",
            FieldKind::ZodiacSign => "Zodiac Sign",
            FieldKind::ChineseZodiac => "Chinese Zodiac Sign",
            FieldKind::Tanganana => "Tangananica or Tanganana?",
            FieldKind::HappinessLevel => "Happiness Level",
            FieldKind::Claustrophobic => "Claustrophobic?",
            FieldKind::AttrText => "Attr text",
            FieldKind::AttrEnumBar => "Attr enum Bar",
            FieldKind::AttrBoolean => "Attr boolean",
            FieldKind::PickACard => "Pick a card",
            FieldKind::FavoriteColor => "Favorite Color",
            FieldKind::FavoriteLetter => "Favorite Letter",
            FieldKind::FavoriteNumber => "Favorite Number",
            FieldKind::TarotPrediction => "Tarot Prediction",
            FieldKind::FavoriteQuote => "Favorite Quote",
            FieldKind::FavoriteHour => "Favorite Hour",
            FieldKind::Lonely => "Lonely?",
            FieldKind::Gmi => "gmi?",
            FieldKind::ElOLaPr => "El o La PR?",
            FieldKind::TheOrThePr => "The or The PR?",
            FieldKind::Crush => "Crush",
            FieldKind::MarriedTo => "Married to",
            FieldKind::Hates => "Hates",
            FieldKind::Status => "Status",
            FieldKind::Sin => "Sin",
            FieldKind::HasSeenTheSky => "Has seen the sky?",
            FieldKind::Temperature => "Temperature (ºC)",
            FieldKind::Delicious => "Delicious?",
            FieldKind::Region => "Region",
            FieldKind::Dni => "DNI",
            FieldKind::FavoriteSeason => "Favorite Season",
        }
    }
}

pub fn format_weight(g: u32) -> String {
    if g >= 1000 {
        format!("{:.1}kg", g as f32 / 1000.0)
    } else {
        format!("{}g", g)
    }
}

pub fn generate_rut(rng: &mut impl RngExt) -> String {
    let body: u32 = rng.random_range(1_000_000..25_000_001);
    let s = body.to_string();
    let digits: Vec<u32> = s.chars().rev().map(|c| c as u32 - '0' as u32).collect();
    let multipliers = [2u32, 3, 4, 5, 6, 7];
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| d * multipliers[i % multipliers.len()])
        .sum();
    let rem = 11 - (sum % 11);
    let v = match rem {
        11 => '0',
        10 => 'K',
        n => char::from_digit(n, 10).unwrap_or('0'),
    };
    match s.len() {
        8 => format!("{}.{}.{}-{}", &s[..2], &s[2..5], &s[5..], v),
        _ => format!("{}.{}.{}-{}", &s[..1], &s[1..4], &s[4..], v),
    }
}

fn plain(s: impl Into<String>) -> FieldValue {
    FieldValue {
        text: s.into(),
        swatch: None,
    }
}

pub fn gen_field_value(
    kind: FieldKind,
    fish: &Fish,
    all_names: &[String],
    rng: &mut impl RngExt,
) -> FieldValue {
    match kind {
        FieldKind::Iq => {
            let v: i32 = match rng.random_range(0..100u32) {
                0 => -30,
                1 => 3000,
                _ => rng.random_range(55..=145i32),
            };
            plain(v.to_string())
        }

        FieldKind::ZodiacSign => {
            const S: &[&str] = &[
                "Aries",
                "Taurus",
                "Gemini",
                "Cancer",
                "Leo",
                "Virgo",
                "Libra",
                "Scorpio",
                "Sagittarius",
                "Capricorn",
                "Aquarius",
                "Pisces",
            ];
            plain(S[rng.random_range(0..S.len())])
        }

        FieldKind::ChineseZodiac => {
            const S: &[&str] = &[
                "鼠", "牛", "虎", "兔", "龍", "蛇", "馬", "羊", "猴", "雞", "狗", "豬",
            ];
            plain(S[rng.random_range(0..S.len())])
        }

        FieldKind::Tanganana => plain(if rng.random::<bool>() {
            "Tangananica"
        } else {
            "Tanganana"
        }),

        FieldKind::HappinessLevel => plain(format!("{}%", rng.random_range(0..=100u32))),

        FieldKind::Claustrophobic => plain(if rng.random_range(0..10u32) == 0 {
            "Yes"
        } else {
            "No"
        }),

        FieldKind::AttrText => plain("corge"),

        FieldKind::AttrEnumBar => plain(""),

        FieldKind::AttrBoolean => plain(if rng.random::<bool>() { "Sí" } else { "No" }),

        FieldKind::PickACard => {
            const RANKS: &[&str] = &[
                "Ace", "2", "3", "4", "5", "6", "7", "8", "9", "10", "Jack", "Queen", "King",
            ];
            const SUITS: &[&str] = &["Spades", "Hearts", "Diamonds", "Clubs"];
            let roll = rng.random_range(0..54u32);
            if roll >= 52 {
                plain("Joker")
            } else {
                let rank = RANKS[(roll % 13) as usize];
                let suit = SUITS[(roll / 13) as usize];
                plain(format!("{} of {}", rank, suit))
            }
        }

        FieldKind::FavoriteColor => {
            const COLORS: &[Color] = &[
                Color::Red,
                Color::Green,
                Color::Blue,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
                Color::LightRed,
                Color::LightGreen,
                Color::LightBlue,
                Color::LightYellow,
                Color::LightMagenta,
                Color::LightCyan,
                FIELD_FIRE_COLOR,
                FIELD_EARTH_COLOR,
                FIELD_WATER_COLOR,
                FIELD_SPIRIT_COLOR,
                FIELD_AIR_COLOR,
            ];
            let c = if fish.species == FishSpecies::Goldenfish {
                GOLD
            } else {
                COLORS[rng.random_range(0..COLORS.len())]
            };
            FieldValue {
                text: "      ".to_string(),
                swatch: Some(c),
            }
        }

        FieldKind::FavoriteLetter => {
            plain(char::from(b'A' + rng.random_range(0..26u8)).to_string())
        }

        FieldKind::FavoriteNumber => plain(rng.random::<i64>().to_string()),

        FieldKind::TarotPrediction => {
            const MUTANT_SPREADS: &[&str] = &[
                "The Tower + Death + Ten of Swords",
                "Three of Swords + The Devil + Nine of Swords",
                "Five of Pentacles + Ten of Wands + The Moon",
                "The Tower Reversed + Eight of Swords + The Hanged Man",
                "Death Reversed + Four of Pentacles + Judgement Reversed",
                "The Devil Reversed + Seven of Swords + Wheel of Fortune Reversed",
                "Ten of Swords Reversed + The Moon Reversed + Five of Cups",
                "Five of Cups + Hermit Reversed + Lovers Reversed",
                "Justice Reversed + The Tower + King of Pentacles Reversed",
                "Sun Reversed + Star Reversed + Nine of Wands",
            ];
            const GOLDEN_SPREADS: &[&str] = &[
                "The Sun + Ten of Cups + Ace of Pentacles",
                "The Star + Lovers + The World",
                "Wheel of Fortune + Six of Wands + The Emperor",
                "Ace of Cups + The Empress + Four of Wands",
                "The Magician + The Chariot + The Sun",
                "Death + The Star + Ace of Wands",
                "The Devil Reversed + Judgement + The Fool",
                "Nine of Pentacles + King of Pentacles + The World",
                "Two of Cups + Ten of Cups + Star Reversed",
                "Strength + The Hierophant + Sun Reversed",
            ];
            const CARDS: &[&str] = &[
                "The Fool",
                "The Magician",
                "The High Priestess",
                "The Empress",
                "The Emperor",
                "The Hierophant",
                "The Lovers",
                "The Chariot",
                "Strength",
                "The Hermit",
                "Wheel of Fortune",
                "Justice",
                "The Hanged Man",
                "Death",
                "Temperance",
                "The Devil",
                "The Tower",
                "The Star",
                "The Moon",
                "The Sun",
                "Judgement",
                "The World",
            ];
            match fish.species {
                FishSpecies::Mutantfish => {
                    plain(MUTANT_SPREADS[rng.random_range(0..MUTANT_SPREADS.len())])
                }
                FishSpecies::Goldenfish => {
                    plain(GOLDEN_SPREADS[rng.random_range(0..GOLDEN_SPREADS.len())])
                }
                _ => {
                    let mut deck: Vec<&str> = CARDS.to_vec();
                    let i1 = rng.random_range(0..deck.len());
                    let c1 = deck.remove(i1);
                    let i2 = rng.random_range(0..deck.len());
                    let c2 = deck.remove(i2);
                    let i3 = rng.random_range(0..deck.len());
                    let c3 = deck.remove(i3);
                    let r1 = if rng.random::<bool>() { " (R)" } else { "" };
                    let r2 = if rng.random::<bool>() { " (R)" } else { "" };
                    let r3 = if rng.random::<bool>() { " (R)" } else { "" };
                    plain(format!("{}{}, {}{}, {}{}", c1, r1, c2, r2, c3, r3))
                }
            }
        }

        FieldKind::FavoriteQuote => match fish.species {
            FishSpecies::Mutantfish => plain("OOGHHHHHHH"),
            FishSpecies::Goldenfish => plain("Gonna be, gonna be golden"),
            _ => {
                let count = rng.random_range(2..=8u32);
                plain((0..count).map(|_| "glub").collect::<Vec<_>>().join(" "))
            }
        },

        FieldKind::FavoriteHour => plain(format!(
            "{:02}:{:02}",
            rng.random_range(0..24u32),
            rng.random_range(0..60u32)
        )),

        FieldKind::Lonely => plain(if rng.random_range(0..10u32) == 0 {
            "Yes"
        } else {
            "No"
        }),

        FieldKind::Gmi => plain(if rng.random::<bool>() { "gmi" } else { "ngmi" }),

        FieldKind::ElOLaPr => plain("La PR"),

        FieldKind::TheOrThePr => plain("The PR"),

        FieldKind::Crush => {
            if all_names.is_empty() {
                plain("—")
            } else {
                plain(all_names[rng.random_range(0..all_names.len())].clone())
            }
        }

        FieldKind::MarriedTo => {
            if all_names.is_empty() {
                plain("—")
            } else {
                plain(all_names[rng.random_range(0..all_names.len())].clone())
            }
        }

        FieldKind::Hates => {
            if all_names.is_empty() {
                plain("No one")
            } else {
                let target = &all_names[rng.random_range(0..all_names.len())];
                if *target == fish.name {
                    plain("No one")
                } else {
                    plain(target.clone())
                }
            }
        }

        FieldKind::Status => {
            const S: &[&str] = &[
                "Swimming",
                "Pondering",
                "Breathing",
                "Prompting",
                "Prooompting",
                "Fishing",
                "Dreaming",
                "Feeling",
                "Happy",
                "Sad",
                "Nauseous",
                "Kicking Rocks",
                "Giving the Time",
                "Taking out the turn",
                "Falling",
                "Floating",
            ];
            plain(S[rng.random_range(0..S.len())])
        }

        FieldKind::Sin => {
            const SINS: &[&str] = &[
                "Lust", "Gluttony", "Greed", "Sloth", "Wrath", "Envy", "Pride",
            ];
            match fish.species {
                FishSpecies::Mutantfish => plain("[REDACTED]"),
                FishSpecies::Goldenfish => plain(""),
                _ => plain(SINS[rng.random_range(0..SINS.len())]),
            }
        }

        FieldKind::HasSeenTheSky => plain(if fish.species == FishSpecies::Goldenfish {
            "Yes"
        } else {
            "No"
        }),

        FieldKind::Temperature => {
            let t = 25.0f32 + rng.random_range(-14.0f32..14.0);
            plain(format!("{:.1}°C", t))
        }

        FieldKind::Delicious => match fish.species {
            FishSpecies::Mutantfish => plain("NOOOOOOOOOO"),
            FishSpecies::Goldenfish => plain("Yes."),
            _ => plain(match rng.random_range(0..3u32) {
                0 => "Yes",
                1 => "No",
                _ => "Maybe",
            }),
        },

        FieldKind::Region => {
            const R: &[&str] = &[
                "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "RM",
                "XIV", "XV",
            ];
            plain(R[rng.random_range(0..R.len())])
        }

        FieldKind::Dni => plain(generate_rut(rng)),

        FieldKind::FavoriteSeason => {
            const S: &[&str] = &["Winter", "Autumn", "Spring", "Summer"];
            plain(S[rng.random_range(0..S.len())])
        }
    }
}
