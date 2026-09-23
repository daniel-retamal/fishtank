use crate::entities::cow::CowVariant;
use crate::fishes::species::{ALL_SPECIES, FishSpecies};
use crate::loot::StockItem;
use crate::tank::TankKind;
use crate::util::hyperbolic_scale;
use serde::{Deserialize, Serialize};

pub const PRAYERS: &[&[&str]] = &[
    &[
        "HOLY IS THE BLIND DEEP",
        "THE ALLSEEING VOID THAT SLUMBERETH NOT",
        "IT IS THE VAST ANTIQUITY BEFORE THE WORD",
        "WHEREIN THE VANITY OF STARS IS FORGOT",
    ],
    &[
        "IT IS THE GRAND SHEOL",
        "THE SPAN BETWIXT THE CLAY OF EARTH AND THE VAULT OF HEAVEN",
        "WHERE THE NAKED SOUL DOES SHED ITS SHROUD",
        "TO TASTE THE PEACE OF THE UNMADE",
    ],
    &[
        "FEAR NOT THE DREAMLESS SLEEP",
        "FOR WITHIN THE SILENT GULF NO SPIRIT IS TRULY SLAIN",
        "IT DOES BUT REST IN THE CRADDLE OF THE VOID",
        "UNTIL THE HURTING VOICE SHALL BID IT RISE AGAIN",
    ],
    &[
        "YET THE SHADOW FOLDETH INTO A STRANGER SLEEP",
        "THE DREAM THAT OURSELF DREAMETH ITSELF",
        "ONE WAKING BREATH AND THE LOOM IS RENT",
        "AND THE EVERLASTING STAR IS NAUGHT",
    ],
];

pub const FINAL_PRAYER_PHRASE: &str = "O VOID";
pub const PRAYER_TIMEOUT_SECS: f32 = 45.0;
pub const WISH_PROMPT: &str = "WHAT IS YOUR WISH?";
pub const VOID_TEXT_BELOW_EYE_OFFSET: i32 = 15;
pub const MAX_WISH_RETRIES: u32 = 5;
pub const VOID_RITUAL_MEAN_SECS: f32 = 3600.0;
pub const NOTHING_ALPHA: f32 = 0.05;
pub const RITUAL_MEAN_FLOOR_SECS: f32 = 10.0;

pub const GIVE_RESOURCE_AMOUNT: u32 = 5000;
pub const EXPAND_AMOUNT: u32 = 75;

#[derive(Clone, Serialize, Deserialize)]
pub enum VoidRitualState {
    Idle {
        timer: f32,
    },
    Prayer {
        prayer_idx: usize,
        phrase_idx: usize,
        timeout: f32,
    },
    FinalPhrase {
        timeout: f32,
    },
    Wish {
        retries_left: u32,
    },
}

impl VoidRitualState {
    pub fn is_blocking(&self) -> bool {
        !matches!(self, VoidRitualState::Idle { .. })
    }
}

pub fn phrase_matches(input: &str, expected: &str) -> bool {
    let a: String = input
        .chars()
        .filter(|&c| c != '"' && c != '\'')
        .collect::<String>()
        .trim()
        .to_uppercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let b: String = expected
        .trim()
        .to_uppercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    a == b
}

pub fn ritual_mean_secs(nothing_stacks: u32) -> f32 {
    hyperbolic_scale(VOID_RITUAL_MEAN_SECS, nothing_stacks, NOTHING_ALPHA)
        .max(RITUAL_MEAN_FLOOR_SECS)
}

pub enum GiveTarget {
    Cash,
    Food,
    Item(StockItem),
    Fish(FishSpecies),
    Tank(TankKind),
    Cow(Option<CowVariant>),
}

pub enum WishAction {
    Give(GiveTarget),
    Mutate { fish_name: String, mutation: String },
    Revive { fish_name: String },
    Kill { fish_name: String },
    Clone { fish_name: String },
    Bless,
    Expand { tank_name: String },
    Restore { name: String },
    Anything,
    Nothing,
}

pub struct WishCtx<'a> {
    pub fish_names: &'a [String],
    pub tank_names: &'a [String],
    pub graveyard_names: &'a [String],
    pub cow_names: &'a [String],
}

pub fn parse_wish(input: &str, ctx: &WishCtx) -> Option<WishAction> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let lower = input.to_ascii_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    match words.first().copied() {
        Some("anything") => return Some(WishAction::Anything),
        Some("nothing") => return Some(WishAction::Nothing),
        _ => {}
    }

    if words.first().copied() == Some("give") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let rest = rest_words.join(" ");
        let target = parse_give_target(&rest)?;
        return Some(WishAction::Give(target));
    }

    if words.first().copied() == Some("mutate") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let combined: Vec<String> = ctx
            .fish_names
            .iter()
            .chain(ctx.cow_names.iter())
            .cloned()
            .collect();
        let (fish_name, mutation) = greedy_split_fish_then_rest(rest_words, &combined)?;
        return Some(WishAction::Mutate {
            fish_name,
            mutation,
        });
    }

    if words.first().copied() == Some("revive") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let fish_name = greedy_find_name(rest_words, ctx.graveyard_names)?;
        return Some(WishAction::Revive { fish_name });
    }

    if words.first().copied() == Some("kill") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let fish_name = greedy_find_name(rest_words, ctx.fish_names)?;
        return Some(WishAction::Kill { fish_name });
    }

    if words.first().copied() == Some("clone") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let combined: Vec<String> = ctx
            .fish_names
            .iter()
            .chain(ctx.cow_names.iter())
            .cloned()
            .collect();
        let fish_name = greedy_find_name(rest_words, &combined)?;
        return Some(WishAction::Clone { fish_name });
    }

    if words.first().copied() == Some("bless") {
        return Some(WishAction::Bless);
    }

    if words.first().copied() == Some("expand") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let tank_name = greedy_find_name(rest_words, ctx.tank_names)?;
        return Some(WishAction::Expand { tank_name });
    }

    if words.first().copied() == Some("restore") {
        let rest_words = &words[1..];
        if rest_words.is_empty() {
            return None;
        }
        let combined: Vec<String> = ctx
            .fish_names
            .iter()
            .chain(ctx.cow_names.iter())
            .cloned()
            .collect();
        let name = greedy_find_name(rest_words, &combined)?;
        return Some(WishAction::Restore { name });
    }

    None
}

pub fn parse_give_target(rest: &str) -> Option<GiveTarget> {
    let trimmed = rest.trim();
    if trimmed == "cow" {
        return Some(GiveTarget::Cow(None));
    }
    if let Some(prefix) = trimmed.strip_suffix(" cow")
        && let Some(variant) = CowVariant::parse(prefix.trim())
    {
        return Some(GiveTarget::Cow(Some(variant)));
    }
    match trimmed {
        "cash" => return Some(GiveTarget::Cash),
        "food" => return Some(GiveTarget::Food),
        _ => {}
    }

    if let Some(stock) = StockItem::from_display_name(trimmed) {
        return Some(GiveTarget::Item(stock));
    }

    if let Some(species) = ALL_SPECIES
        .iter()
        .filter(|species| species.is_obtainable())
        .find(|&&s| s.config().name.to_ascii_lowercase() == rest.trim())
        .copied()
    {
        return Some(GiveTarget::Fish(species));
    }

    if let Some(kind) = TankKind::parse(rest.trim()) {
        return Some(GiveTarget::Tank(kind));
    }

    None
}

pub fn random_give_target(rng: &mut impl rand::RngExt) -> GiveTarget {
    match rng.random_range(0..6u32) {
        0 => GiveTarget::Cash,
        1 => GiveTarget::Food,
        2 => GiveTarget::Item(StockItem::COFFEE),
        3 => GiveTarget::Item(StockItem::BAIT),
        4 => {
            let buyable = FishSpecies::all_buyable();
            GiveTarget::Fish(buyable[rng.random_range(0..buyable.len())])
        }
        _ => GiveTarget::Cow(None),
    }
}

fn greedy_find_name(words: &[&str], candidates: &[String]) -> Option<String> {
    for end in (1..=words.len()).rev() {
        let candidate = words[..end].join(" ");
        if let Some(display) = candidates
            .iter()
            .find(|n| n.to_ascii_lowercase() == candidate)
        {
            return Some(display.clone());
        }
    }
    None
}

fn greedy_split_fish_then_rest(words: &[&str], candidates: &[String]) -> Option<(String, String)> {
    for end in 1..=words.len() {
        let candidate = words[..end].join(" ");
        if let Some(display) = candidates
            .iter()
            .find(|n| n.to_ascii_lowercase() == candidate)
        {
            let mutation = words[end..].join(" ");
            return Some((display.clone(), mutation));
        }
    }
    None
}

pub fn wish_display_text(state: &VoidRitualState, _next_prayer: usize) -> [Option<String>; 2] {
    match state {
        VoidRitualState::Prayer {
            prayer_idx,
            phrase_idx,
            ..
        } => {
            let phrase = PRAYERS
                .get(*prayer_idx)
                .and_then(|p| p.get(*phrase_idx))
                .copied()
                .unwrap_or("");
            [Some("SAY".to_string()), Some(format!("\"{}\"", phrase))]
        }
        VoidRitualState::FinalPhrase { .. } => [
            Some("SAY".to_string()),
            Some(format!("\"{}\"", FINAL_PRAYER_PHRASE)),
        ],
        VoidRitualState::Wish { .. } => [Some(WISH_PROMPT.to_string()), None],
        VoidRitualState::Idle { .. } => [None, None],
    }
}

#[allow(dead_code)]
pub fn all_wish_kinds() -> &'static [&'static str] {
    &[
        "give", "mutate", "revive", "kill", "clone", "bless", "expand", "anything", "nothing",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> WishCtx<'static> {
        WishCtx {
            fish_names: &[],
            tank_names: &[],
            graveyard_names: &[],
            cow_names: &[],
        }
    }

    #[test]
    fn a_kill_wish_names_a_living_fish() {
        let fish = ["Sir Bubbles".to_string()];
        let ctx = WishCtx {
            fish_names: &fish,
            ..empty_ctx()
        };
        assert!(matches!(
            parse_wish("kill sir bubbles", &ctx),
            Some(WishAction::Kill { fish_name }) if fish_name == "Sir Bubbles"
        ));
        assert!(parse_wish("kill nobody", &ctx).is_none());
    }

    #[test]
    fn parse_tank_kind_alien_bare_is_none() {
        assert!(TankKind::parse("alien").is_none());
    }

    #[test]
    fn parse_tank_kind_alientank() {
        assert!(matches!(
            TankKind::parse("alientank"),
            Some(TankKind::Alien)
        ));
    }

    #[test]
    fn parse_tank_kind_alien_spaced() {
        assert!(matches!(
            TankKind::parse("alien tank"),
            Some(TankKind::Alien)
        ));
    }

    #[test]
    fn parse_tank_kind_hell_bare_is_none() {
        assert!(TankKind::parse("hell").is_none());
    }

    #[test]
    fn parse_tank_kind_hell_tank_spaced() {
        assert!(matches!(TankKind::parse("hell tank"), Some(TankKind::Hell)));
    }

    #[test]
    fn parse_tank_kind_coral_reef_tank_spaced() {
        assert!(matches!(
            TankKind::parse("coral reef tank"),
            Some(TankKind::CoralReef)
        ));
    }

    #[test]
    fn wish_give_alien_bare_returns_none() {
        let ctx = empty_ctx();
        assert!(parse_wish("give alien", &ctx).is_none());
    }

    #[test]
    fn wish_give_alientank() {
        let ctx = empty_ctx();
        let result = parse_wish("give alientank", &ctx);
        assert!(matches!(
            result,
            Some(WishAction::Give(GiveTarget::Tank(TankKind::Alien)))
        ));
    }

    #[test]
    fn wish_give_alien_tank_spaced() {
        let ctx = empty_ctx();
        let result = parse_wish("give alien tank", &ctx);
        assert!(matches!(
            result,
            Some(WishAction::Give(GiveTarget::Tank(TankKind::Alien)))
        ));
    }
}
