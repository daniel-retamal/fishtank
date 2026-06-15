use rand::RngExt;

use crate::fishes::fish::Fish;
use crate::fishes::mutations::{Mutation, apply_mutation_to_fish};
use crate::loot::MilkVariant;
use crate::restore::Restorable;

pub const COFFEE_DURATION: f32 = 60.0;
pub const BAIT_DURATION: f32 = 60.0;
pub const CONSUMABLE_STACK_BONUS: f32 = 30.0;

pub const COFFEE_SPEED_MULT: f32 = 0.8;
pub const COFFEE_SWAY_MULT: f32 = 0.7;
pub const COFFEE_ZOOMIE_DT_MULT: f32 = 0.3;

const CHOCOLATE_WEIGHT_BONUS_G: u32 = 5000;

pub const MILK_STATUS_DURATION: f32 = 60.0;
pub const MILK_STATUS_STACK_BONUS: f32 = 30.0;

pub const VISUAL_CALCULUS_ALPHA: f32 = 0.175;
pub const VOLITION_ALPHA: f32 = 0.9;
pub const PHYSICAL_INSTRUMENT_ALPHA: f32 = 0.9;
pub const REACTION_SPEED_ALPHA: f32 = 0.9;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MilkStatus {
    VisualCalculus,
    Volition,
    PhysicalInstrument,
    ReactionSpeed,
}

impl MilkStatus {
    pub const ALL: &'static [MilkStatus] = &[
        MilkStatus::VisualCalculus,
        MilkStatus::Volition,
        MilkStatus::PhysicalInstrument,
        MilkStatus::ReactionSpeed,
    ];

    pub fn random(rng: &mut impl RngExt) -> Self {
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            MilkStatus::VisualCalculus => "visual-calculus",
            MilkStatus::Volition => "volition",
            MilkStatus::PhysicalInstrument => "physical-instrument",
            MilkStatus::ReactionSpeed => "reaction-speed",
        }
    }
}

#[derive(Clone)]
pub struct ActiveMilkStatus {
    pub kind: MilkStatus,
    pub stacks: u32,
    pub time_remaining: f32,
}

pub fn apply_milk_to_fish(variant: MilkVariant, fish: &mut Fish, rng: &mut impl RngExt) {
    match variant {
        MilkVariant::Plain => {}
        MilkVariant::Chocolate => {
            fish.weight_g = fish.weight_g.saturating_add(CHOCOLATE_WEIGHT_BONUS_G);
            apply_mutation_to_fish(fish, Mutation::SizeIncrease, rng);
        }
        MilkVariant::Strawberry => {
            apply_mutation_to_fish(fish, Mutation::Strawberry, rng);
        }
        MilkVariant::Vanilla => fish.restore(),
        MilkVariant::Alien => {
            apply_mutation_to_fish(fish, Mutation::Alienation, rng);
        }
    }
}
