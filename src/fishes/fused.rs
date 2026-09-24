use crate::entities::cow::{Cow, CowVariant};
use crate::fishes::botfish::BotfishState;
use crate::fishes::fish::Fish;
use crate::fishes::species::FishSpecies;
use crate::fishes::unfish::{UnfishKind, UnfishState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Lineage {
    Fish(FishSpecies),
    Unfish(UnfishKind),
    Cow(CowVariant),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Snapshot {
    Fish(Box<Fish>),
    Cow(Box<Cow>),
}

impl Snapshot {
    pub fn as_fish(&self) -> Option<&Fish> {
        match self {
            Snapshot::Fish(fish) => Some(fish),
            Snapshot::Cow(_) => None,
        }
    }

    pub fn as_fish_mut(&mut self) -> Option<&mut Fish> {
        match self {
            Snapshot::Fish(fish) => Some(fish),
            Snapshot::Cow(_) => None,
        }
    }

    pub fn as_cow(&self) -> Option<&Cow> {
        match self {
            Snapshot::Cow(cow) => Some(cow),
            Snapshot::Fish(_) => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FusedComponent {
    pub lineage: Lineage,
    pub name: String,
    pub weight_g: u32,
    pub persona: Option<Box<UnfishState>>,
    pub program: Option<Box<BotfishState>>,
    pub snapshot: Option<Snapshot>,
}

impl FusedComponent {
    pub fn fish(species: FishSpecies, name: String, weight_g: u32) -> Self {
        Self {
            lineage: Lineage::Fish(species),
            name,
            weight_g,
            persona: None,
            program: None,
            snapshot: None,
        }
    }

    pub fn unfish(kind: UnfishKind, name: String, weight_g: u32) -> Self {
        Self {
            lineage: Lineage::Unfish(kind),
            name,
            weight_g,
            persona: None,
            program: None,
            snapshot: None,
        }
    }

    pub fn cow(variant: CowVariant, name: String) -> Self {
        Self {
            lineage: Lineage::Cow(variant),
            name,
            weight_g: 0,
            persona: None,
            program: None,
            snapshot: None,
        }
    }

    pub fn with_snapshot(mut self, snapshot: Fish) -> Self {
        self.snapshot = Some(Snapshot::Fish(Box::new(snapshot)));
        self
    }

    pub fn with_cow_snapshot(mut self, snapshot: Cow) -> Self {
        self.snapshot = Some(Snapshot::Cow(Box::new(snapshot)));
        self
    }

    pub fn fish_snapshot(&self) -> Option<&Fish> {
        self.snapshot.as_ref().and_then(Snapshot::as_fish)
    }

    pub fn fish_snapshot_mut(&mut self) -> Option<&mut Fish> {
        self.snapshot.as_mut().and_then(Snapshot::as_fish_mut)
    }

    pub fn cow_snapshot(&self) -> Option<&Cow> {
        self.snapshot.as_ref().and_then(Snapshot::as_cow)
    }

    pub fn cow_snapshot_mut(&mut self) -> Option<&mut Cow> {
        match self.snapshot.as_mut()? {
            Snapshot::Cow(cow) => Some(cow),
            Snapshot::Fish(_) => None,
        }
    }

    pub fn ability_species(&self) -> Vec<FishSpecies> {
        match self.fish_snapshot() {
            Some(snapshot) => snapshot.ability_components(),
            None => self.fish_species().into_iter().collect(),
        }
    }

    pub fn milk_variants(&self) -> Vec<CowVariant> {
        match self.cow_snapshot() {
            Some(snapshot) => snapshot.milk_components(),
            None => self.cow_variant().into_iter().collect(),
        }
    }

    pub fn flattened(mut self) -> Vec<FusedComponent> {
        let nested = self
            .fish_snapshot()
            .map(|snapshot| snapshot.fused_components().to_vec())
            .unwrap_or_default();
        self.snapshot = None;
        if nested.is_empty() {
            return vec![self];
        }
        let mut flat: Vec<FusedComponent> = nested
            .into_iter()
            .flat_map(FusedComponent::flattened)
            .map(|mut stack| {
                stack.persona = None;
                stack.program = None;
                stack
            })
            .collect();
        if let Some(first) = flat.first_mut() {
            first.persona = self.persona;
            first.program = self.program;
        }
        flat
    }

    pub fn fish_species(&self) -> Option<FishSpecies> {
        match self.lineage {
            Lineage::Fish(species) => Some(species),
            _ => None,
        }
    }

    pub fn cow_variant(&self) -> Option<CowVariant> {
        match self.lineage {
            Lineage::Cow(variant) => Some(variant),
            _ => None,
        }
    }
}
