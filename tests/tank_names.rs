use fishtank::fishes::species::FishSpecies;
use fishtank::tank::{Tank, TankKind};

fn make_tank() -> Tank {
    Tank::new("Test".to_string(), TankKind::Base, &[])
}

fn rng() -> impl rand::RngExt {
    rand::rng()
}

#[test]
fn first_spawn_uses_requested_name() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    assert_eq!(tank.fish[0].name, "Nemo");
}

#[test]
fn duplicate_name_gets_roman_ii_suffix() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    assert_eq!(tank.fish[1].name, "Nemo II");
}

#[test]
fn third_duplicate_gets_roman_iii_suffix() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    assert_eq!(tank.fish[2].name, "Nemo III");
}

#[test]
fn fourth_duplicate_gets_roman_iv_suffix() {
    let mut tank = make_tank();
    let mut rng = rng();
    for _ in 0..4 {
        tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    }
    assert_eq!(tank.fish[3].name, "Nemo IV");
}

#[test]
fn different_names_are_independent() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    tank.spawn_fish(FishSpecies::Merluza, "Dory".to_string(), &mut rng);
    assert_eq!(tank.fish[0].name, "Nemo");
    assert_eq!(tank.fish[1].name, "Dory");
}

#[test]
fn multi_word_name_deduplication() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Blue Tang".to_string(), &mut rng);
    tank.spawn_fish(FishSpecies::Merluza, "Blue Tang".to_string(), &mut rng);
    assert_eq!(tank.fish[1].name, "Blue Tang II");
}

#[test]
fn used_names_set_is_updated_on_spawn() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Nemo".to_string(), &mut rng);
    assert!(
        tank.used_names.contains("Nemo"),
        "used_names must track spawned fish name"
    );
}
