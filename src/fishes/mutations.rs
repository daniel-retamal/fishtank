use rand::RngExt;

use super::fish::{Fish, compute_display_width};
use super::mutant::{EyeState, MIN_BODY_CHARS, MutantTail, MutationRecord, random_rgb};
use super::species::{BodyTemplate, FishSpecies, TailKind};

const MUTATION_MAX_BODY_SIZE: usize = 12;
pub const MUTATION_PATCH_MAX: usize = 12;
pub const MUTATION_PATCH_COUNT_MIN: usize = 2;
pub const MUTATION_PATCH_COUNT_MAX: usize = 4;
const GLISTEN_SPEED_FAST_MULT: f32 = 3.5;
const GLISTEN_SPEED_SLOW_MULT: f32 = 0.25;
const SWAY_SPEED_CLAMP_MAX: f32 = 2.5;
const DOUBLE_EYE_COUNT_MAX: usize = 3;

pub enum Mutation {
    SizeChange(i32),
    ColorPatch,
    EyeChange { delta: i32 },
    EyeColor,
    GlisteningSpeed { fast: bool },
    GlisteningMode,
    GlisteningColor,
    BodyColor,
    BodyVariant,
    TailVariant,
    MouthVariant,
    Doublefish,
    Mitosis,
    GlisteningEnable,
    GlisteningDisable,
}

impl Mutation {
    pub fn display_name(&self) -> &'static str {
        match self {
            Mutation::SizeChange(d) => {
                if *d > 0 { "size+" } else { "size-" }
            }
            Mutation::ColorPatch => "colorpatch",
            Mutation::EyeChange { delta } => {
                if *delta > 0 { "eye+" } else { "eye-" }
            }
            Mutation::EyeColor => "eyecolor",
            Mutation::GlisteningSpeed { fast } => {
                if *fast { "glistenfast" } else { "glistenslow" }
            }
            Mutation::GlisteningMode => "glistenmode",
            Mutation::GlisteningColor => "glistencolor",
            Mutation::BodyColor => "bodycolor",
            Mutation::BodyVariant => "bodyvariant",
            Mutation::TailVariant => "tailvariant",
            Mutation::MouthVariant => "mouthvariant",
            Mutation::Doublefish => "doublefish",
            Mutation::Mitosis => "mitosis",
            Mutation::GlisteningEnable => "glistenenable",
            Mutation::GlisteningDisable => "glistendisable",
        }
    }
}

pub fn tail_kind_to_mutant_tail(kind: TailKind) -> MutantTail {
    match kind {
        TailKind::WideCurly => MutantTail::Curly,
        TailKind::Swaying { .. } => MutantTail::Swaying,
        _ => MutantTail::Wide,
    }
}

pub fn pick_random_miracle_mutation(
    _fish: &Fish,
    is_double: bool,
    has_glisten: bool,
    rng: &mut impl RngExt,
) -> Mutation {
    let mut valid: Vec<u8> = (0..10).collect();
    if !is_double {
        valid.push(10);
        valid.push(11);
    }
    if is_double {
        valid.push(12);
    }
    if !has_glisten {
        valid.push(13);
    }
    if has_glisten {
        valid.push(14);
    }
    match valid[rng.random_range(0..valid.len())] {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange { delta: if rng.random::<bool>() { 1 } else { -1 } },
        3 => Mutation::EyeColor,
        4 => Mutation::GlisteningSpeed { fast: rng.random::<bool>() },
        5 => Mutation::GlisteningMode,
        6 => Mutation::GlisteningColor,
        7 => Mutation::BodyColor,
        8 => Mutation::BodyVariant,
        9 => Mutation::MouthVariant,
        10 => Mutation::TailVariant,
        11 => Mutation::Doublefish,
        12 => Mutation::Mitosis,
        13 => Mutation::GlisteningEnable,
        _ => Mutation::GlisteningDisable,
    }
}

pub fn pick_random_mutation(fish: &Fish, rng: &mut impl RngExt) -> Mutation {
    let is_double = fish.mutant.as_ref().is_some_and(|mutant| mutant.is_double);
    let has_glisten = fish.mutant.as_ref().is_some_and(|mutant| mutant.glistening_color.is_some());
    let mut valid: Vec<u8> = (0..10).collect();
    if !is_double {
        valid.push(10);
        valid.push(11);
    }
    if is_double {
        valid.push(12);
    }
    if !has_glisten {
        valid.push(13);
    }
    if has_glisten {
        valid.push(14);
    }
    match valid[rng.random_range(0..valid.len())] {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange { delta: if rng.random::<bool>() { 1 } else { -1 } },
        3 => Mutation::EyeColor,
        4 => Mutation::GlisteningSpeed { fast: rng.random::<bool>() },
        5 => Mutation::GlisteningMode,
        6 => Mutation::GlisteningColor,
        7 => Mutation::BodyColor,
        8 => Mutation::BodyVariant,
        9 => Mutation::MouthVariant,
        10 => Mutation::TailVariant,
        11 => Mutation::Doublefish,
        12 => Mutation::Mitosis,
        13 => Mutation::GlisteningEnable,
        _ => Mutation::GlisteningDisable,
    }
}

pub fn apply_mutation_to_fish(fish: &mut Fish, mutation: Mutation, rng: &mut impl RngExt) {
    let mutation_name = mutation.display_name().to_string();
    {
        let mutation_record = fish.mutations.get_or_insert_with(|| Box::new(MutationRecord::default()));
        mutation_record.count += 1;
        mutation_record.history.push(mutation_name);
    }
    let mutant = fish.mutant.as_mut().unwrap();
    match mutation {
        Mutation::SizeChange(delta) => {
            let max_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());
            let min_size = max_eyes + MIN_BODY_CHARS;
            let new_size = (fish.body_size as i32 + delta)
                .clamp(min_size as i32, MUTATION_MAX_BODY_SIZE as i32) as usize;
            fish.body_size = new_size;
        }
        Mutation::ColorPatch => {
            let count = rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
            let width = mutant.display_width(fish.body_size);
            for _ in 0..count {
                let pos = rng.random_range(0..width);
                mutant.color_patches.push((pos, random_rgb(rng)));
            }
            if mutant.color_patches.len() > MUTATION_PATCH_MAX {
                let excess = mutant.color_patches.len() - MUTATION_PATCH_MAX;
                mutant.color_patches.drain(0..excess);
            }
        }
        Mutation::EyeChange { delta } => {
            let max_eyes = fish
                .body_size
                .saturating_sub(MIN_BODY_CHARS)
                .clamp(1, 4) as i32;
            let new_left = (mutant.left_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
            while mutant.left_eyes.len() < new_left {
                mutant.left_eyes.push(EyeState::new(rng));
            }
            mutant.left_eyes.truncate(new_left);
            let new_right = (mutant.right_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
            while mutant.right_eyes.len() < new_right {
                mutant.right_eyes.push(EyeState::new(rng));
            }
            mutant.right_eyes.truncate(new_right);
        }
        Mutation::EyeColor => {
            mutant.eye_color = Some(random_rgb(rng));
        }
        Mutation::GlisteningSpeed { fast } => {
            fish.sway_speed *= if fast { GLISTEN_SPEED_FAST_MULT } else { GLISTEN_SPEED_SLOW_MULT };
            fish.sway_speed = fish.sway_speed.clamp(0.01, SWAY_SPEED_CLAMP_MAX);
        }
        Mutation::GlisteningMode => {
            mutant.glistening_mode = mutant.glistening_mode.random_other(rng);
        }
        Mutation::GlisteningColor => {
            mutant.glistening_color = if mutant.glistening_color.is_none() || rng.random::<bool>() {
                Some(random_rgb(rng))
            } else {
                None
            };
        }
        Mutation::BodyColor => {
            fish.color = random_rgb(rng);
            mutant.color_patches.clear();
            if mutant.eye_color.is_some() {
                mutant.eye_color = Some(random_rgb(rng));
            }
        }
        Mutation::BodyVariant => {
            mutant.body_variant = (mutant.body_variant + rng.random_range(1..4u8)) % 4;
        }
        Mutation::TailVariant => {
            mutant.tail_variant = match mutant.tail_variant {
                MutantTail::Wide => MutantTail::Swaying,
                MutantTail::Swaying => MutantTail::Curly,
                MutantTail::Curly => MutantTail::Wide,
            };
        }
        Mutation::MouthVariant => {
            mutant.mouth_inverted = !mutant.mouth_inverted;
        }
        Mutation::Doublefish => {
            let count = if fish.species == FishSpecies::Mutantfish {
                rng.random_range(1..=DOUBLE_EYE_COUNT_MAX)
            } else {
                match fish.species.config().body {
                    BodyTemplate::Standard(_) | BodyTemplate::Alternating(_, _) => 1,
                    BodyTemplate::Fixed { .. } => 0,
                }
            };
            mutant.double_head_eyes = (0..count).map(|_| EyeState::new(rng)).collect();
            mutant.is_double = true;
        }
        Mutation::Mitosis => unreachable!(),
        Mutation::GlisteningEnable => {
            fish.sway_speed = fish
                .species
                .config()
                .sway_speed
                .max(0.08)
                .clamp(0.01, SWAY_SPEED_CLAMP_MAX);
            mutant.glistening_color = Some(random_rgb(rng));
        }
        Mutation::GlisteningDisable => {
            fish.sway_speed = fish.species.config().sway_speed;
            mutant.glistening_color = None;
        }
    }
    let mutant = fish.mutant.as_ref().unwrap();
    fish.display_width = match fish.species.config().body {
        BodyTemplate::Fixed { left, .. } => {
            let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
            if n_chars > 1 {
                let n_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());
                if mutant.is_double {
                    2 * (1 + n_eyes + fish.body_size)
                } else {
                    2 + n_eyes + fish.body_size
                }
            } else if mutant.is_double {
                2
            } else {
                compute_display_width(fish.species, fish.body_size)
            }
        }
        _ => mutant.display_width(fish.body_size),
    };
}
