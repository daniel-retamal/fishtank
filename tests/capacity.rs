use crossterm::event::KeyCode;
use fishtank::{
    entities::ufo::Ufo,
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    tank::{Tank, TankKind},
    testing::Tui,
};

const MUTATION_ROUNDS: usize = 40;
const ABDUCTION_TICK_BUDGET: usize = 5000;

fn lab() -> Tui {
    let mut tui = Tui::new();
    tui.clear_tank();
    tui
}

fn fill(tank: &mut Tank, species: FishSpecies) {
    let mut n = 0;
    while !tank.is_full() {
        tank.spawn_fish(species, format!("Filler {n}"), &mut rand::rng());
        n += 1;
    }
}

fn no_tank_is_over_capacity(tui: &Tui) {
    for tank in &tui.app.tanks {
        assert!(
            tank.fish.len() <= tank.capacity(),
            "{} holds {} fish in room for {}",
            tank.name,
            tank.fish.len(),
            tank.capacity()
        );
    }
}

fn index_of(tui: &Tui, kind: TankKind) -> usize {
    tui.app
        .tanks
        .iter()
        .position(|tank| tank.kind == kind)
        .expect("the tank exists")
}

fn living(tui: &Tui, name: &str) -> Option<usize> {
    tui.app
        .tanks
        .iter()
        .position(|tank| tank.fish.iter().any(|fish| fish.name == name))
}

fn a_full_fishtank_beside_a_coralreeftank() -> Tui {
    let mut tui = lab();
    tui.run("/give coralreeftank");
    tui.run("/switch \"Fishtank\"");
    let home = index_of(&tui, TankKind::Base);
    fill(&mut tui.app.tanks[home], FishSpecies::Merluza);
    tui
}

#[test]
fn a_tank_on_the_way_counts_a_ufo_carrying_a_fish_as_a_fish_already_home() {
    let mut tank = Tank::new("Test".to_string(), TankKind::Base, &[]);
    fill(&mut tank, FishSpecies::Merluza);
    let last = tank.fish.len() - 1;
    let passenger = tank.take_fish(last);
    assert!(!tank.is_full());

    tank.ufos.push(Ufo::new_drop_fish(0.0, 0.0, passenger));

    assert!(tank.is_full(), "the seat is held for the fish in the beam");
    assert_eq!(tank.room(), 0);
}

#[test]
fn a_gift_to_a_full_tank_lands_in_the_next_tank_with_room() {
    let mut tui = a_full_fishtank_beside_a_coralreeftank();
    tui.run("/give salmon");

    let coral = index_of(&tui, TankKind::CoralReef);
    assert_eq!(living(&tui, "Salmon"), Some(coral));
    no_tank_is_over_capacity(&tui);
}

#[test]
fn a_clone_and_a_revival_land_where_there_is_room() {
    let mut tui = a_full_fishtank_beside_a_coralreeftank();
    tui.run("/clone \"Filler 0\"");
    tui.run("/kill \"Filler 1\"");
    fill(&mut tui.app.tanks[0], FishSpecies::Merluza);
    tui.run("/revive \"Filler 1\"");

    let coral = index_of(&tui, TankKind::CoralReef);
    assert_eq!(living(&tui, "Filler 0's Clone"), Some(coral));
    assert_eq!(living(&tui, "Filler 1"), Some(coral));
    no_tank_is_over_capacity(&tui);
}

#[test]
fn with_no_room_anywhere_a_gift_a_clone_and_a_purchase_do_nothing() {
    let mut tui = lab();
    tui.stake();
    fill(&mut tui.app.tanks[0], FishSpecies::Merluza);
    let cash = tui.app.purse.balance();

    tui.run("/give salmon");
    tui.run("/clone \"Filler 0\"");
    tui.run("/buy salmon");

    assert_eq!(living(&tui, "Salmon"), None);
    assert_eq!(living(&tui, "Filler 0's Clone"), None);
    assert_eq!(tui.app.purse.balance(), cash, "nothing is spent");
    tui.screen().expect_absent("for sale!");
    no_tank_is_over_capacity(&tui);
}

#[test]
fn with_no_room_anywhere_a_revival_leaves_the_fish_in_its_grave() {
    let mut tui = lab();
    tui.run("/spawn merluza \"Ann\"");
    tui.run("/kill \"Ann\"");
    let home = index_of(&tui, TankKind::Base);
    fill(&mut tui.app.tanks[home], FishSpecies::Merluza);

    tui.run("/revive \"Ann\"");

    assert_eq!(living(&tui, "Ann"), None);
    assert!(tui.app.graveyard.names().any(|name| name == "Ann"));
}

#[test]
fn a_double_divides_only_when_its_tank_has_room_for_the_child() {
    let mut tui = lab();
    tui.run("/spawn merluza \"Dup\"");
    tui.run("/mutate \"Dup\" telophase");
    fill(&mut tui.app.tanks[0], FishSpecies::Merluza);
    let full = tui.app.tanks[0].fish.len();

    tui.run("/mutate \"Dup\" cytokinesis");
    assert_eq!(tui.app.tanks[0].fish.len(), full, "no room, no division");

    tui.run("/kill \"Filler 0\"");
    tui.run("/mutate \"Dup\" cytokinesis");
    assert_eq!(tui.app.tanks[0].fish.len(), full, "the child took the room");
    no_tank_is_over_capacity(&tui);
}

#[test]
fn a_full_tank_of_mutants_mutating_at_random_never_overflows() {
    let mut tank = Tank::new("Test".to_string(), TankKind::Rad, &[]);
    fill(&mut tank, FishSpecies::Mutantfish);
    for _ in 0..MUTATION_ROUNDS {
        let names: Vec<String> = tank.fish.iter().map(|fish| fish.name.clone()).collect();
        for name in names {
            tank.apply_named_mutation(&name, "");
            assert!(tank.fish.len() <= tank.capacity());
        }
    }
}

#[test]
fn an_abduction_into_a_full_base_founds_a_new_one() {
    let mut tui = lab();
    tui.run("/give alientank");
    let base = index_of(&tui, TankKind::Alien);
    fill(&mut tui.app.tanks[base], FishSpecies::Merluza);
    tui.run("/switch \"Fishtank\"");
    tui.run("/spawn merluza \"Taken\"");

    tui.run("/startfishabduction");
    for _ in 0..ABDUCTION_TICK_BUDGET {
        if living(&tui, "Taken").is_some_and(|tank| tank != 0) {
            break;
        }
        tui.tick_n(1);
    }

    let landed = living(&tui, "Taken").expect("the abducted fish lives on");
    assert_ne!(landed, base, "the full base is never pushed past its room");
    assert!(tui.app.tanks[landed].kind.is_ufo_base());
    no_tank_is_over_capacity(&tui);
}

#[test]
fn a_cheatfish_with_no_room_anywhere_waits_and_lands_when_room_appears() {
    let mut tui = Tui::as_player(100, 30);
    tui.clear_tank();
    fill(&mut tui.app.tanks[0], FishSpecies::Merluza);

    tui.run("/cheat");
    tui.type_text("show me the money");
    tui.key(KeyCode::Enter);
    tui.tick_n(1);

    let cheatfish = |tui: &Tui| {
        tui.app.tanks[0]
            .fish
            .iter()
            .any(|fish: &Fish| fish.species == FishSpecies::Cheatfish)
    };
    assert!(!cheatfish(&tui), "no room: the price of the cheat waits");
    no_tank_is_over_capacity(&tui);

    tui.app.tanks[0].take_fish(0);
    tui.tick_n(1);
    assert!(cheatfish(&tui), "and lands the moment there is room");
}
