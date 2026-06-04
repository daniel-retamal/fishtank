use fishtank::colors::{LIGHT_GREEN, PINK};
use fishtank::consumable::apply_milk_to_fish;
use fishtank::entities::cow::{Cow, CowVariant};
use fishtank::fishes::fish::Fish;
use fishtank::fishes::mutations::{Mutatable, Mutation, apply_mutation};
use fishtank::fishes::species::{ALL_SPECIES, FishSpecies, SizeCategory};
use fishtank::loot::MilkVariant;
use fishtank::tank::TankKind;

fn rng() -> impl rand::RngExt {
    rand::rng()
}

const STRAWBERRY_SELL_BONUS_PCT: u8 = 25;
const CHOCOLATE_WEIGHT_BONUS_G: u32 = 5000;

#[test]
fn unfish_sells_for_nothing_regardless_of_weight() {
    assert_eq!(
        FishSpecies::Unfish.sell_value(9999, SizeCategory::XL, 5),
        0,
        "an unfish must always sell for 0"
    );
}

#[test]
fn common_fish_below_weight_base_sells_at_base() {
    assert_eq!(
        FishSpecies::Merluza.sell_value(0, SizeCategory::S, 0),
        12,
        "a small Merluza at zero weight sells at its base price"
    );
}

#[test]
fn common_fish_at_weight_cap_sells_at_cap() {
    assert_eq!(
        FishSpecies::Merluza.sell_value(2500, SizeCategory::S, 0),
        40,
        "a small Merluza at the weight cap sells at its cap price"
    );
}

#[test]
fn mutantfish_sell_value_is_base_plus_mutations_plus_weight_tenth() {
    assert_eq!(
        FishSpecies::Mutantfish.sell_value(100, SizeCategory::M, 2),
        710,
        "mutantfish = 500 base + 2*100 per mutation + 100/10 weight"
    );
}

#[test]
fn sell_value_is_monotonic_in_weight() {
    let light = FishSpecies::Snapper.sell_value(500, SizeCategory::M, 0);
    let heavy = FishSpecies::Snapper.sell_value(5000, SizeCategory::M, 0);
    assert!(heavy >= light, "heavier fish must never sell for less");
}

#[test]
fn plain_milk_changes_nothing() {
    let mut rng = rng();
    let mut fish = Fish::new_for_display(FishSpecies::Merluza, &mut rng);
    let color = fish.color;
    let weight = fish.weight_g;
    apply_milk_to_fish(MilkVariant::Plain, &mut fish, &mut rng);
    assert_eq!(fish.color, color, "plain milk leaves color untouched");
    assert_eq!(fish.weight_g, weight, "plain milk leaves weight untouched");
}

#[test]
fn strawberry_milk_turns_fish_pink_and_adds_sell_bonus() {
    let mut rng = rng();
    let mut fish = Fish::new_for_display(FishSpecies::Merluza, &mut rng);
    apply_milk_to_fish(MilkVariant::Strawberry, &mut fish, &mut rng);
    assert_eq!(fish.color, PINK, "strawberry milk recolors the fish pink");
    assert_eq!(
        fish.sell_price_bonus_pct, STRAWBERRY_SELL_BONUS_PCT,
        "strawberry milk grants the sell bonus"
    );
}

#[test]
fn alien_milk_alienates_fish_to_light_green() {
    let mut rng = rng();
    let mut fish = Fish::new_for_display(FishSpecies::Merluza, &mut rng);
    apply_milk_to_fish(MilkVariant::Alien, &mut fish, &mut rng);
    assert_eq!(
        fish.color, LIGHT_GREEN,
        "alien milk applies the alienation recolor"
    );
}

#[test]
fn chocolate_milk_adds_weight() {
    let mut rng = rng();
    let mut fish = Fish::new_for_display(FishSpecies::Mutantfish, &mut rng);
    let weight = fish.weight_g;
    apply_milk_to_fish(MilkVariant::Chocolate, &mut fish, &mut rng);
    assert_eq!(
        fish.weight_g,
        weight + CHOCOLATE_WEIGHT_BONUS_G,
        "chocolate milk adds a fixed weight bonus"
    );
}

#[test]
fn cow_size_plus_grows_torso_by_one() {
    let mut rng = rng();
    let mut cow = Cow::new("Bessie".to_string(), CowVariant::Brown, 5.0, 5.0, &mut rng);
    let before = cow.body_size();
    apply_mutation(&mut cow, Mutation::SizeChange(1), &mut rng);
    assert_eq!(
        cow.body_size(),
        before + 1,
        "size+ grows a cow's torso by one"
    );
}

#[test]
fn cow_alienation_recolors_and_marks_alienated() {
    let mut rng = rng();
    let mut cow = Cow::new("Bessie".to_string(), CowVariant::Brown, 5.0, 5.0, &mut rng);
    apply_mutation(&mut cow, Mutation::Alienation, &mut rng);
    assert_eq!(
        cow.color, LIGHT_GREEN,
        "alienation recolors a cow light green"
    );
    assert!(cow.is_alienated(), "alienation marks the cow as alienated");
}

#[test]
fn every_tank_kind_parses_back_from_its_display_name() {
    for &kind in TankKind::all() {
        let lower = kind.display_name().to_ascii_lowercase();
        assert!(
            TankKind::parse(&lower) == Some(kind),
            "tank '{}' must parse back from its lowercased display name",
            kind.display_name()
        );
    }
}

#[test]
fn every_species_parses_back_from_its_display_name() {
    for &species in ALL_SPECIES {
        assert_eq!(
            FishSpecies::parse(species.display_name()),
            Some(species),
            "species '{}' must parse back from its display name",
            species.display_name()
        );
    }
}
