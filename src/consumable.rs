use rand::RngExt;

use crate::fishes::fish::Fish;
use crate::fishes::mutations::{Mutation, apply_mutation_to_fish};
use crate::fishes::parts::Part;
use crate::loot::{ConsumableKind, MilkVariant, StockItem};
use crate::restore::Restorable;
use crate::ui::hints::{HINT_ENTER_CONSUME, HINT_ENTER_ETCH, HINT_ENTER_INSTALL};

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsumeTarget {
    Milk(MilkVariant),
    Part(Part),
    Etch(usize),
}

impl ConsumeTarget {
    pub fn display_name(self) -> &'static str {
        match self {
            ConsumeTarget::Milk(variant) => variant.display_name(),
            ConsumeTarget::Part(part) => part.display_name(),
            ConsumeTarget::Etch(_) => ConsumableKind::Fabricator.display_name(),
        }
    }

    pub fn stock(self) -> StockItem {
        match self {
            ConsumeTarget::Milk(variant) => StockItem::Consumable(ConsumableKind::Milk(variant)),
            ConsumeTarget::Part(part) => StockItem::Consumable(ConsumableKind::Part(part)),
            ConsumeTarget::Etch(_) => StockItem::Consumable(ConsumableKind::Fabricator),
        }
    }

    pub fn header(self, name: &str, remaining: u32) -> String {
        match self {
            ConsumeTarget::Milk(_) => format!(" {name} to the fishes ({remaining}) "),
            ConsumeTarget::Part(_) | ConsumeTarget::Etch(_) => {
                format!(" {name} to the botfishes ({remaining}) ")
            }
        }
    }

    pub fn confirm_hint(self) -> &'static str {
        match self {
            ConsumeTarget::Milk(_) => HINT_ENTER_CONSUME,
            ConsumeTarget::Part(_) => HINT_ENTER_INSTALL,
            ConsumeTarget::Etch(_) => HINT_ENTER_ETCH,
        }
    }

    pub fn accepts(self, fish: &Fish) -> bool {
        if fish.unfish_state.is_some() {
            return false;
        }
        match self {
            ConsumeTarget::Milk(_) => true,
            ConsumeTarget::Part(_) | ConsumeTarget::Etch(_) => fish.is_programmable(),
        }
    }

    pub fn apply_to(self, fish: &mut Fish, rng: &mut impl RngExt) -> bool {
        match self {
            ConsumeTarget::Milk(variant) => {
                apply_milk_to_fish(variant, fish, rng);
                true
            }
            ConsumeTarget::Part(part) => fish.script_mut().is_some_and(|bot| bot.install(part)),
            ConsumeTarget::Etch(_) => false,
        }
    }
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
        MilkVariant::Irradiated => {
            fish.pending_rad_mutations += rng.random_range(10..=15);
        }
    }
}
