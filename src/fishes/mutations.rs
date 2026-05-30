use rand::RngExt;
use ratatui::style::Color;

use super::fish::{Fish, compute_display_width};
use super::mutant::{
    EyeState, MIN_BODY_CHARS, MutantState, MutantTail, MutationRecord, random_rgb,
};
use super::species::{BodyTemplate, FishSpecies, TailKind};
use crate::colors::{DARK_GRAY, LIGHT_GREEN, PINK, WHITE};

const MUTATION_MAX_BODY_SIZE: usize = 12;
pub const MUTATION_PATCH_MAX: usize = 12;
pub const MUTATION_PATCH_COUNT_MIN: usize = 2;
pub const MUTATION_PATCH_COUNT_MAX: usize = 4;
const GLISTEN_SPEED_FAST_MULT: f32 = 3.5;
const GLISTEN_SPEED_SLOW_MULT: f32 = 0.25;
const SWAY_SPEED_CLAMP_MAX: f32 = 2.5;
const SWAY_SPEED_GLISTEN_FLOOR: f32 = 0.08;
const SWAY_SPEED_CLAMP_MIN: f32 = 0.01;

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
    Alienation,
    Strawberry,
}

impl Mutation {
    pub fn display_name(&self) -> &'static str {
        match self {
            Mutation::SizeChange(d) => {
                if *d > 0 {
                    "size+"
                } else {
                    "size-"
                }
            }
            Mutation::ColorPatch => "colorpatch",
            Mutation::EyeChange { delta } => {
                if *delta > 0 {
                    "eye+"
                } else {
                    "eye-"
                }
            }
            Mutation::EyeColor => "eyecolor",
            Mutation::GlisteningSpeed { fast } => {
                if *fast {
                    "glistenfast"
                } else {
                    "glistenslow"
                }
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
            Mutation::Alienation => "alienation",
            Mutation::Strawberry => "strawberry",
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

pub trait Mutatable {
    fn body_size(&self) -> usize;
    fn set_body_size(&mut self, n: usize);
    fn color(&self) -> Color;
    fn set_color(&mut self, c: Color);
    fn sway_speed(&self) -> f32;
    fn set_sway_speed(&mut self, s: f32);
    fn default_sway_speed(&self) -> f32;
    fn has_mutant(&self) -> bool {
        true
    }
    fn ensure_mutant<R: RngExt>(&mut self, _rng: &mut R) {}
    fn mutant(&self) -> &MutantState;
    fn mutant_mut(&mut self) -> &mut MutantState;
    fn mutations_record_mut(&mut self) -> &mut MutationRecord;
    fn doublefish_eye_count<R: RngExt>(&self, rng: &mut R) -> usize;
    fn allows_tail_variant(&self) -> bool {
        true
    }
    fn recompute_display_width(&mut self);
    fn color_patch_range(&self) -> usize {
        self.mutant().display_width(self.body_size())
    }
}

const DOUBLE_EYE_COUNT_MAX: usize = 3;

impl Mutatable for Fish {
    fn body_size(&self) -> usize {
        self.body_size
    }
    fn set_body_size(&mut self, n: usize) {
        self.body_size = n;
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, c: Color) {
        self.color = c;
    }
    fn sway_speed(&self) -> f32 {
        self.sway_speed
    }
    fn set_sway_speed(&mut self, s: f32) {
        self.sway_speed = s;
    }
    fn default_sway_speed(&self) -> f32 {
        self.species.config().sway_speed
    }
    fn has_mutant(&self) -> bool {
        self.mutant.is_some()
    }
    fn ensure_mutant<R: RngExt>(&mut self, rng: &mut R) {
        if self.mutant.is_none() {
            let tail = match self.species.config().body {
                BodyTemplate::Standard(bc) | BodyTemplate::Alternating(bc, _) => {
                    tail_kind_to_mutant_tail(bc.tail)
                }
                BodyTemplate::Fixed { .. } => MutantTail::Wide,
            };
            self.mutant = Some(Box::new(MutantState::new_for_standard(tail, rng)));
        }
    }
    fn mutant(&self) -> &MutantState {
        self.mutant.as_ref().unwrap()
    }
    fn mutant_mut(&mut self) -> &mut MutantState {
        self.mutant.as_mut().unwrap()
    }
    fn mutations_record_mut(&mut self) -> &mut MutationRecord {
        self.mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
    }
    fn doublefish_eye_count<R: RngExt>(&self, rng: &mut R) -> usize {
        if self.species == FishSpecies::Mutantfish {
            rng.random_range(1..=DOUBLE_EYE_COUNT_MAX)
        } else {
            match self.species.config().body {
                BodyTemplate::Standard(_) | BodyTemplate::Alternating(_, _) => 1,
                BodyTemplate::Fixed { .. } => 0,
            }
        }
    }
    fn allows_tail_variant(&self) -> bool {
        !matches!(self.species.config().body, BodyTemplate::Fixed { .. })
    }
    fn recompute_display_width(&mut self) {
        let Some(mutant) = self.mutant.as_ref() else {
            return;
        };
        self.display_width = match self.species.config().body {
            BodyTemplate::Fixed { left, .. } => {
                let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                if n_chars > 1 {
                    let n_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());
                    if mutant.is_double {
                        2 * (1 + n_eyes + self.body_size)
                    } else {
                        2 + n_eyes + self.body_size
                    }
                } else if mutant.is_double {
                    2
                } else {
                    compute_display_width(self.species, self.body_size)
                }
            }
            _ => mutant.display_width(self.body_size),
        };
    }
}

pub fn pick_random_miracle_mutation<M: Mutatable>(
    target: &M,
    is_double: bool,
    has_glisten: bool,
    rng: &mut impl RngExt,
) -> Mutation {
    let mut valid: Vec<u8> = (0..10).collect();
    if !is_double {
        if target.allows_tail_variant() {
            valid.push(10);
        }
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
    valid.push(15);
    let pick = valid[rng.random_range(0..valid.len())];
    let _ = target;
    decode_mutation_choice(pick, rng)
}

pub fn pick_random_mutation<M: Mutatable>(target: &M, rng: &mut impl RngExt) -> Mutation {
    let is_double = target.mutant().is_double;
    let has_glisten = target.mutant().glistening_color.is_some();
    pick_random_miracle_mutation(target, is_double, has_glisten, rng)
}

fn decode_mutation_choice(pick: u8, rng: &mut impl RngExt) -> Mutation {
    match pick {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange {
            delta: if rng.random::<bool>() { 1 } else { -1 },
        },
        3 => Mutation::EyeColor,
        4 => Mutation::GlisteningSpeed {
            fast: rng.random::<bool>(),
        },
        5 => Mutation::GlisteningMode,
        6 => Mutation::GlisteningColor,
        7 => Mutation::BodyColor,
        8 => Mutation::BodyVariant,
        9 => Mutation::MouthVariant,
        10 => Mutation::TailVariant,
        11 => Mutation::Doublefish,
        12 => Mutation::Mitosis,
        13 => Mutation::GlisteningEnable,
        14 => Mutation::GlisteningDisable,
        _ => Mutation::Alienation,
    }
}

pub fn apply_mutation<M: Mutatable>(target: &mut M, mutation: Mutation, rng: &mut impl RngExt) {
    let mutation_name = mutation.display_name().to_string();
    {
        let record = target.mutations_record_mut();
        record.count += 1;
        record.history.push(mutation_name);
    }
    match mutation {
        Mutation::SizeChange(delta) => {
            let max_eyes = {
                let mutant = target.mutant();
                mutant.left_eyes.len().max(mutant.right_eyes.len())
            };
            let min_size = max_eyes + MIN_BODY_CHARS;
            let new_size = (target.body_size() as i32 + delta)
                .clamp(min_size as i32, MUTATION_MAX_BODY_SIZE as i32)
                as usize;
            target.set_body_size(new_size);
        }
        Mutation::ColorPatch => {
            let width = target.color_patch_range().max(1);
            let count = rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
            let mutant = target.mutant_mut();
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
            let body_size = target.body_size();
            let max_eyes = body_size.saturating_sub(MIN_BODY_CHARS).clamp(1, 4) as i32;
            let mutant = target.mutant_mut();
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
            target.mutant_mut().eye_color = Some(random_rgb(rng));
        }
        Mutation::GlisteningSpeed { fast } => {
            let factor = if fast {
                GLISTEN_SPEED_FAST_MULT
            } else {
                GLISTEN_SPEED_SLOW_MULT
            };
            let s =
                (target.sway_speed() * factor).clamp(SWAY_SPEED_CLAMP_MIN, SWAY_SPEED_CLAMP_MAX);
            target.set_sway_speed(s);
        }
        Mutation::GlisteningMode => {
            let mutant = target.mutant_mut();
            mutant.glistening_mode = mutant.glistening_mode.random_other(rng);
        }
        Mutation::GlisteningColor => {
            let mutant = target.mutant_mut();
            mutant.glistening_color = if mutant.glistening_color.is_none() || rng.random::<bool>() {
                Some(random_rgb(rng))
            } else {
                None
            };
        }
        Mutation::BodyColor => {
            target.set_color(random_rgb(rng));
            let mutant = target.mutant_mut();
            mutant.color_patches.clear();
            if mutant.eye_color.is_some() {
                mutant.eye_color = Some(random_rgb(rng));
            }
        }
        Mutation::BodyVariant => {
            let mutant = target.mutant_mut();
            mutant.body_variant = (mutant.body_variant + rng.random_range(1..4u8)) % 4;
        }
        Mutation::TailVariant => {
            let mutant = target.mutant_mut();
            mutant.tail_variant = match mutant.tail_variant {
                MutantTail::Wide => MutantTail::Swaying,
                MutantTail::Swaying => MutantTail::Curly,
                MutantTail::Curly => MutantTail::Wide,
            };
        }
        Mutation::MouthVariant => {
            let mutant = target.mutant_mut();
            mutant.mouth_inverted = !mutant.mouth_inverted;
        }
        Mutation::Doublefish => {
            let count = target.doublefish_eye_count(rng);
            let mutant = target.mutant_mut();
            mutant.double_head_eyes = (0..count).map(|_| EyeState::new(rng)).collect();
            mutant.is_double = true;
        }
        Mutation::Mitosis => unreachable!(),
        Mutation::GlisteningEnable => {
            let s = target
                .default_sway_speed()
                .max(SWAY_SPEED_GLISTEN_FLOOR)
                .clamp(SWAY_SPEED_CLAMP_MIN, SWAY_SPEED_CLAMP_MAX);
            target.set_sway_speed(s);
            target.mutant_mut().glistening_color = Some(WHITE);
        }
        Mutation::GlisteningDisable => {
            target.set_sway_speed(target.default_sway_speed());
            target.mutant_mut().glistening_color = None;
        }
        Mutation::Alienation => {
            target.set_color(LIGHT_GREEN);
            target.ensure_mutant(rng);
            let mutant = target.mutant_mut();
            mutant.eye_color = Some(DARK_GRAY);
            mutant.color_patches.clear();
        }
        Mutation::Strawberry => {
            target.set_color(PINK);
            target.ensure_mutant(rng);
            let mutant = target.mutant_mut();
            mutant.color_patches.clear();
        }
    }
    target.recompute_display_width();
}

pub fn apply_mutation_to_fish(fish: &mut Fish, mutation: Mutation, rng: &mut impl RngExt) {
    apply_mutation(fish, mutation, rng);
}
