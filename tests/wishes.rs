use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fishtank::{
    app::{App, Launch},
    economy::Rarity,
    entities::cow::CowVariant,
    fishes::{fish::Fish, species::FishSpecies},
    loot::{ConsumableKind, StockItem},
    tank::{Tank, TankKind},
    void_ritual::{
        EXPAND_AMOUNT, GIVE_RESOURCE_AMOUNT, GiveTarget, MAX_WISH_RETRIES, VoidRitualState,
        WishAction, WishCtx, parse_wish,
    },
};

fn key_press(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

fn ctx<'a>(fish: &'a [String], tanks: &'a [String], graveyard: &'a [String]) -> WishCtx<'a> {
    WishCtx {
        fish_names: fish,
        tank_names: tanks,
        graveyard_names: graveyard,
        cow_names: &[],
    }
}

fn ctx_with_cows<'a>(
    fish: &'a [String],
    tanks: &'a [String],
    graveyard: &'a [String],
    cows: &'a [String],
) -> WishCtx<'a> {
    WishCtx {
        fish_names: fish,
        tank_names: tanks,
        graveyard_names: graveyard,
        cow_names: cows,
    }
}

fn names(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

fn submit_wish(app: &mut App, wish: &str) {
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set(wish.to_string());
    app.submit_ritual_input();
}

fn retries_left(app: &App) -> u32 {
    match app.void_ritual {
        VoidRitualState::Wish { retries_left } => retries_left,
        _ => 0,
    }
}

fn is_idle(app: &App) -> bool {
    matches!(app.void_ritual, VoidRitualState::Idle { .. })
}

#[test]
fn give_cash_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give cash", &c),
        Some(WishAction::Give(GiveTarget::Cash))
    ));
}

#[test]
fn give_food_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give food", &c),
        Some(WishAction::Give(GiveTarget::Food))
    ));
}

#[test]
fn give_coffee_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give coffee", &c),
        Some(WishAction::Give(GiveTarget::Item(StockItem::Consumable(
            ConsumableKind::Coffee
        ))))
    ));
}

#[test]
fn give_bait_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give bait", &c),
        Some(WishAction::Give(GiveTarget::Item(StockItem::Consumable(
            ConsumableKind::Bait
        ))))
    ));
}

#[test]
fn give_junk_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give junk", &c),
        Some(WishAction::Give(GiveTarget::Item(StockItem::Junk)))
    ));
}

#[test]
fn give_necronomicon_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give necronomicon", &c),
        Some(WishAction::Give(GiveTarget::Item(StockItem::Consumable(
            ConsumableKind::Necronomicon
        ))))
    ));
}

#[test]
fn give_fish_species_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give merluza", &c),
        Some(WishAction::Give(GiveTarget::Fish(FishSpecies::Merluza)))
    ));
}

#[test]
fn give_tank_base_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give fishtank", &c),
        Some(WishAction::Give(GiveTarget::Tank(TankKind::Base)))
    ));
}

#[test]
fn give_tank_coral_reef_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give coralreeftank", &c),
        Some(WishAction::Give(GiveTarget::Tank(TankKind::CoralReef)))
    ));
}

#[test]
fn give_tank_hell_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give helltank", &c),
        Some(WishAction::Give(GiveTarget::Tank(TankKind::Hell)))
    ));
}

#[test]
fn give_tank_void_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("give voidtank", &c),
        Some(WishAction::Give(GiveTarget::Tank(TankKind::Void)))
    ));
}

#[test]
fn give_unknown_target_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("give foobar", &c).is_none());
}

#[test]
fn give_without_target_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("give", &c).is_none());
}

#[test]
fn mutate_valid_fish_parses() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("mutate nemo bodycolor", &c),
        Some(WishAction::Mutate { .. })
    ));
}

#[test]
fn mutate_valid_fish_captures_mutation_name() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    if let Some(WishAction::Mutate {
        fish_name,
        mutation,
    }) = parse_wish("mutate nemo bodycolor", &c)
    {
        assert_eq!(fish_name, "Nemo");
        assert_eq!(mutation, "bodycolor");
    } else {
        panic!("expected Mutate action");
    }
}

#[test]
fn mutate_valid_fish_no_mutation_name_is_empty() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    if let Some(WishAction::Mutate { mutation, .. }) = parse_wish("mutate nemo", &c) {
        assert!(mutation.is_empty());
    } else {
        panic!("expected Mutate action");
    }
}

#[test]
fn mutate_nonexistent_fish_returns_none() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("mutate ghost bodycolor", &c).is_none());
}

#[test]
fn mutate_without_fish_name_returns_none() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("mutate", &c).is_none());
}

#[test]
fn revive_dead_fish_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&["Lost"]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("revive lost", &c),
        Some(WishAction::Revive { fish_name }) if fish_name == "Lost"
    ));
}

#[test]
fn revive_alive_fish_returns_none() {
    let fish = names(&["Alive"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("revive alive", &c).is_none());
}

#[test]
fn revive_nonexistent_fish_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("revive nobody", &c).is_none());
}

#[test]
fn revive_without_name_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&["Lost"]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("revive", &c).is_none());
}

#[test]
fn clone_valid_fish_parses() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("clone nemo", &c),
        Some(WishAction::Clone { fish_name }) if fish_name == "Nemo"
    ));
}

#[test]
fn clone_nonexistent_fish_returns_none() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("clone ghost", &c).is_none());
}

#[test]
fn clone_without_name_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("clone", &c).is_none());
}

#[test]
fn bless_parses_without_a_target() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(parse_wish("bless", &c), Some(WishAction::Bless)));
}

#[test]
fn bless_ignores_any_trailing_words() {
    let fish = names(&["Nemo"]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("bless nemo", &c),
        Some(WishAction::Bless)
    ));
}

#[test]
fn expand_valid_tank_parses() {
    let fish = names(&[]);
    let tanks = names(&["Fishtank"]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("expand fishtank", &c),
        Some(WishAction::Expand { tank_name }) if tank_name == "Fishtank"
    ));
}

#[test]
fn expand_nonexistent_tank_returns_none() {
    let fish = names(&[]);
    let tanks = names(&["Fishtank"]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("expand atlantis", &c).is_none());
}

#[test]
fn expand_without_name_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("expand", &c).is_none());
}

#[test]
fn anything_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("anything", &c),
        Some(WishAction::Anything)
    ));
}

#[test]
fn nothing_parses() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(matches!(
        parse_wish("nothing", &c),
        Some(WishAction::Nothing)
    ));
}

#[test]
fn empty_input_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("", &c).is_none());
}

#[test]
fn whitespace_only_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("   ", &c).is_none());
}

#[test]
fn unknown_command_returns_none() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("foobar", &c).is_none());
}

#[test]
fn max_wish_retries_is_five() {
    assert_eq!(MAX_WISH_RETRIES, 5);
}

#[test]
fn execute_give_cash_increases_cash() {
    let mut app = App::launch(Launch::Debug);
    let before = app.purse.balance();
    submit_wish(&mut app, "give cash");
    assert_eq!(app.purse.balance(), before + GIVE_RESOURCE_AMOUNT);
}

#[test]
fn execute_give_food_increases_food_supply() {
    let mut app = App::launch(Launch::Debug);
    let before = app.food_supply;
    submit_wish(&mut app, "give food");
    assert_eq!(app.food_supply, before + GIVE_RESOURCE_AMOUNT);
}

#[test]
fn execute_give_coffee_increases_coffee_inventory() {
    let mut app = App::launch(Launch::Debug);
    let before = app.inventory.get(&StockItem::COFFEE).copied().unwrap_or(0);
    submit_wish(&mut app, "give coffee");
    assert_eq!(
        app.inventory.get(&StockItem::COFFEE).copied().unwrap_or(0),
        before + StockItem::COFFEE.gift_quantity()
    );
}

#[test]
fn execute_give_bait_increases_bait_inventory() {
    let mut app = App::launch(Launch::Debug);
    let before = app.inventory.get(&StockItem::BAIT).copied().unwrap_or(0);
    submit_wish(&mut app, "give bait");
    assert_eq!(
        app.inventory.get(&StockItem::BAIT).copied().unwrap_or(0),
        before + StockItem::BAIT.gift_quantity()
    );
}

#[test]
fn execute_give_junk_increases_junk_inventory() {
    let mut app = App::launch(Launch::Debug);
    let before = app.inventory.get(&StockItem::Junk).copied().unwrap_or(0);
    submit_wish(&mut app, "give junk");
    assert_eq!(
        app.inventory.get(&StockItem::Junk).copied().unwrap_or(0),
        before + StockItem::Junk.gift_quantity()
    );
}

#[test]
fn execute_give_necronomicon_increases_necronomicon_inventory() {
    let mut app = App::launch(Launch::Debug);
    let before = app
        .inventory
        .get(&StockItem::NECRONOMICON)
        .copied()
        .unwrap_or(0);
    submit_wish(&mut app, "give necronomicon");
    assert_eq!(
        app.inventory
            .get(&StockItem::NECRONOMICON)
            .copied()
            .unwrap_or(0),
        before + StockItem::NECRONOMICON.gift_quantity()
    );
}

#[test]
fn execute_give_fish_species_spawns_fish_in_current_tank() {
    let mut app = App::launch(Launch::Debug);
    let before = app.tanks[app.current_tank].fish.len();
    submit_wish(&mut app, "give merluza");
    assert_eq!(app.tanks[app.current_tank].fish.len(), before + 1);
}

#[test]
fn execute_give_tank_adds_new_tank() {
    let mut app = App::launch(Launch::Debug);
    let before = app.tanks.len();
    submit_wish(&mut app, "give fishtank");
    assert_eq!(app.tanks.len(), before + 1);
}

#[test]
fn execute_give_tank_adds_correct_kind() {
    let mut app = App::launch(Launch::Debug);
    submit_wish(&mut app, "give helltank");
    let added = app.tanks.last().expect("tank was not added");
    assert!(matches!(added.kind, TankKind::Hell));
}

#[test]
fn execute_mutate_valid_fish_creates_mutant_state() {
    let mut app = App::launch(Launch::Debug);
    let fish_name = app.tanks[0].fish[0].name.clone();
    let wish = format!("mutate {} bodycolor", fish_name);
    submit_wish(&mut app, &wish);
    let fish = app.tanks[0]
        .fish
        .iter()
        .find(|f| f.name == fish_name)
        .unwrap();
    assert!(fish.mutant.is_some());
}

#[test]
fn execute_mutate_nonexistent_fish_decrements_retry() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("mutate ghost bodycolor".to_string());
    app.submit_ritual_input();
    assert_eq!(retries_left(&app), MAX_WISH_RETRIES - 1);
}

#[test]
fn execute_revive_dead_fish_removes_from_graveyard_and_adds_to_tank() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let dead = Fish::new(
        FishSpecies::Merluza,
        "Cosmo".to_string(),
        10.0,
        10.0,
        &mut rng,
    );
    app.graveyard.push(dead);
    let fish_before = app.tanks[app.current_tank].fish.len();
    submit_wish(&mut app, "revive cosmo");
    assert!(
        app.graveyard.is_empty(),
        "graveyard must be empty after revive"
    );
    assert_eq!(app.tanks[app.current_tank].fish.len(), fish_before + 1);
}

#[test]
fn execute_revive_alive_fish_is_invalid_and_decrements_retry() {
    let mut app = App::launch(Launch::Debug);
    let fish_before = app.tanks[app.current_tank].fish.len();
    let grav_before = app.graveyard.len();
    let alive_name = app.tanks[0].fish[0].name.clone();
    let wish = format!("revive {}", alive_name);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set(wish);
    app.submit_ritual_input();
    assert_eq!(app.tanks[app.current_tank].fish.len(), fish_before);
    assert_eq!(app.graveyard.len(), grav_before);
    assert_eq!(retries_left(&app), MAX_WISH_RETRIES - 1);
}

#[test]
fn execute_clone_fish_adds_clone_to_tank() {
    let mut app = App::launch(Launch::Debug);
    let fish_before = app.tanks[app.current_tank].fish.len();
    let fish_name = app.tanks[0].fish[0].name.clone();
    let wish = format!("clone {}", fish_name);
    submit_wish(&mut app, &wish);
    assert_eq!(app.tanks[app.current_tank].fish.len(), fish_before + 1);
}

#[test]
fn execute_clone_fish_clone_has_expected_name_suffix() {
    let mut app = App::launch(Launch::Debug);
    let fish_name = app.tanks[0].fish[0].name.clone();
    let expected_clone_name = format!("{}'s Clone", fish_name);
    let wish = format!("clone {}", fish_name);
    submit_wish(&mut app, &wish);
    let has_clone = app.tanks[app.current_tank]
        .fish
        .iter()
        .any(|f| f.name == expected_clone_name);
    assert!(has_clone, "clone fish must be named '<original>'s Clone'");
}

#[test]
fn execute_bless_is_accepted_and_ends_the_ritual() {
    let mut app = App::launch(Launch::Debug);
    submit_wish(&mut app, "bless");
    assert!(is_idle(&app), "a target-less bless is a valid wish");
}

#[test]
fn execute_expand_increases_tank_capacity() {
    let mut app = App::launch(Launch::Debug);
    let tank_name = app.tanks[0].name.clone();
    let before = app.tanks[0].capacity();
    let wish = format!("expand {}", tank_name.to_lowercase());
    submit_wish(&mut app, &wish);
    assert_eq!(app.tanks[0].capacity(), before + EXPAND_AMOUNT as usize);
}

#[test]
fn execute_nothing_increments_nothing_stacks() {
    let mut app = App::launch(Launch::Debug);
    let before = app.nothing_stacks;
    submit_wish(&mut app, "nothing");
    assert_eq!(app.nothing_stacks, before + 1);
}

#[test]
fn execute_nothing_aborts_ritual_to_idle() {
    let mut app = App::launch(Launch::Debug);
    submit_wish(&mut app, "nothing");
    assert!(is_idle(&app));
}

#[test]
fn execute_anything_aborts_ritual_to_idle() {
    let mut app = App::launch(Launch::Debug);
    submit_wish(&mut app, "anything");
    assert!(is_idle(&app));
}

#[test]
fn valid_wish_transitions_ritual_to_idle() {
    let mut app = App::launch(Launch::Debug);
    submit_wish(&mut app, "give cash");
    assert!(is_idle(&app));
}

#[test]
fn invalid_wish_keeps_ritual_in_wish_state() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("give foobar".to_string());
    app.submit_ritual_input();
    assert!(matches!(app.void_ritual, VoidRitualState::Wish { .. }));
}

#[test]
fn four_invalid_wishes_then_valid_executes_and_aborts() {
    let mut app = App::launch(Launch::Debug);
    for _ in 0..4 {
        app.void_ritual = VoidRitualState::Wish {
            retries_left: MAX_WISH_RETRIES,
        };
        app.editor.set("give foobar".to_string());
        app.submit_ritual_input();
    }
    let before_cash = app.purse.balance();
    app.void_ritual = VoidRitualState::Wish { retries_left: 1 };
    app.editor.set("give cash".to_string());
    app.submit_ritual_input();
    assert!(is_idle(&app));
    assert_eq!(app.purse.balance(), before_cash + GIVE_RESOURCE_AMOUNT);
}

#[test]
fn five_invalid_wishes_abort_ritual() {
    let mut app = App::launch(Launch::Debug);
    for i in 0..5 {
        let retries = MAX_WISH_RETRIES - i;
        app.void_ritual = VoidRitualState::Wish {
            retries_left: retries,
        };
        app.editor.set("give foobar".to_string());
        app.submit_ritual_input();
    }
    assert!(is_idle(&app));
}

#[test]
fn retry_count_decrements_on_each_invalid_wish() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    for expected in (1..MAX_WISH_RETRIES).rev() {
        app.editor.set("give foobar".to_string());
        app.submit_ritual_input();
        assert_eq!(retries_left(&app), expected);
    }
}

#[test]
fn wish_index_command_opens_overlay_without_decrementing_retry() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/index".to_string());
    app.submit_ritual_input();
    assert_eq!(
        retries_left(&app),
        MAX_WISH_RETRIES,
        "retry count must not change for /index"
    );
    assert!(
        app.index_overlay_open(),
        "/index must open the index overlay"
    );
}

#[test]
fn wish_fishtanks_command_opens_overlay_without_decrementing_retry() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/fishtanks".to_string());
    app.submit_ritual_input();
    assert_eq!(
        retries_left(&app),
        MAX_WISH_RETRIES,
        "retry count must not change for /fishtanks"
    );
    assert!(
        app.fishtanks_overlay_open(),
        "/fishtanks must open the fishtanks overlay"
    );
}

#[test]
fn wish_names_command_toggles_names_without_decrementing_retry() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    let before = app.settings.show_names;
    app.editor.set("/names".to_string());
    app.submit_ritual_input();
    assert_eq!(
        retries_left(&app),
        MAX_WISH_RETRIES,
        "retry count must not change for /names"
    );
    assert_ne!(
        app.settings.show_names, before,
        "/names must toggle the names setting"
    );
}

#[test]
fn wish_stats_command_toggles_stats_without_decrementing_retry() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    let before = app.settings.show_stats;
    app.editor.set("/stats".to_string());
    app.submit_ritual_input();
    assert_eq!(
        retries_left(&app),
        MAX_WISH_RETRIES,
        "retry count must not change for /stats"
    );
    assert_ne!(
        app.settings.show_stats, before,
        "/stats must toggle the stats setting"
    );
}

#[test]
fn wish_overlay_command_does_not_abort_ritual() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/index".to_string());
    app.submit_ritual_input();
    assert!(
        matches!(app.void_ritual, VoidRitualState::Wish { .. }),
        "ritual must stay in Wish state after an overlay command"
    );
}

#[test]
fn wish_fishtanks_same_tank_enter_does_not_abort_ritual() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/fishtanks".to_string());
    app.submit_ritual_input();
    app.handle_input(key_press(KeyCode::Enter));
    assert!(
        matches!(app.void_ritual, VoidRitualState::Wish { .. }),
        "confirming the current tank must not abort the ritual"
    );
}

#[test]
fn wish_fishtanks_switch_tank_aborts_ritual() {
    let mut app = App::launch(Launch::Debug);
    app.tanks
        .push(Tank::new("Other".to_string(), TankKind::Base, &[]));
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/fishtanks".to_string());
    app.submit_ritual_input();
    if let Some(state) = app.fishtanks_state_mut() {
        state.selected = 1;
    }
    app.handle_input(key_press(KeyCode::Enter));
    assert!(
        is_idle(&app),
        "switching tank in fishtanks during wish must abort the ritual"
    );
}

#[test]
fn wish_fishtanks_esc_closes_overlay_without_aborting_ritual() {
    let mut app = App::launch(Launch::Debug);
    app.tanks
        .push(Tank::new("Other".to_string(), TankKind::Base, &[]));
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/fishtanks".to_string());
    app.submit_ritual_input();
    app.handle_input(key_press(KeyCode::Esc));
    assert!(
        !app.fishtanks_overlay_open(),
        "ESC must close the fishtanks overlay"
    );
    assert!(
        matches!(app.void_ritual, VoidRitualState::Wish { .. }),
        "ESC from fishtanks must not abort the ritual"
    );
}

#[test]
fn wish_index_esc_closes_overlay_without_aborting_ritual() {
    let mut app = App::launch(Launch::Debug);
    app.void_ritual = VoidRitualState::Wish {
        retries_left: MAX_WISH_RETRIES,
    };
    app.editor.set("/index".to_string());
    app.submit_ritual_input();
    app.handle_input(key_press(KeyCode::Esc));
    assert!(
        !app.index_overlay_open(),
        "ESC must close the index overlay"
    );
    assert!(
        matches!(app.void_ritual, VoidRitualState::Wish { .. }),
        "ESC from index must not abort the ritual"
    );
}

#[test]
fn mutate_cow_parses_when_name_is_a_cow() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let cows = names(&["Vaquita"]);
    let c = ctx_with_cows(&fish, &tanks, &grav, &cows);
    if let Some(WishAction::Mutate {
        fish_name,
        mutation,
    }) = parse_wish("mutate vaquita colorpatch", &c)
    {
        assert_eq!(fish_name, "Vaquita");
        assert_eq!(mutation, "colorpatch");
    } else {
        panic!("expected a Mutate action for a cow name");
    }
}

#[test]
fn mutate_cow_does_not_parse_without_cow_in_context() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let c = ctx(&fish, &tanks, &grav);
    assert!(parse_wish("mutate vaquita colorpatch", &c).is_none());
}

#[test]
fn clone_cow_parses_when_name_is_a_cow() {
    let fish = names(&[]);
    let tanks = names(&[]);
    let grav = names(&[]);
    let cows = names(&["Vaquita"]);
    let c = ctx_with_cows(&fish, &tanks, &grav, &cows);
    if let Some(WishAction::Clone { fish_name }) = parse_wish("clone vaquita", &c) {
        assert_eq!(fish_name, "Vaquita");
    } else {
        panic!("expected a Clone action for a cow name");
    }
}

#[test]
fn execute_mutate_cow_records_the_mutation() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let cow_name = app.tanks[0].spawn_cow(CowVariant::Brown, &mut rng);
    let wish = format!("mutate {} colorpatch", cow_name.to_lowercase());
    submit_wish(&mut app, &wish);
    let cow = app.tanks[0]
        .cows
        .iter()
        .find(|c| c.name == cow_name)
        .expect("cow must still exist after the wish");
    let record = cow
        .mutations
        .as_ref()
        .expect("cow must have a mutation record after a mutate wish");
    assert_eq!(record.count, 1);
}

#[test]
fn execute_mutate_cow_aborts_ritual_to_idle() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let cow_name = app.tanks[0].spawn_cow(CowVariant::Brown, &mut rng);
    let wish = format!("mutate {} colorpatch", cow_name.to_lowercase());
    submit_wish(&mut app, &wish);
    assert!(is_idle(&app));
}

#[test]
fn execute_clone_cow_adds_clone_to_tank() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let cow_name = app.tanks[0].spawn_cow(CowVariant::Pink, &mut rng);
    let cows_before = app.tanks[app.current_tank].cows.len();
    let wish = format!("clone {}", cow_name.to_lowercase());
    submit_wish(&mut app, &wish);
    assert_eq!(app.tanks[app.current_tank].cows.len(), cows_before + 1);
}

#[test]
fn execute_clone_cow_has_expected_name_suffix() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let cow_name = app.tanks[0].spawn_cow(CowVariant::Pink, &mut rng);
    let expected_clone_name = format!("{}'s Clone", cow_name);
    let wish = format!("clone {}", cow_name.to_lowercase());
    submit_wish(&mut app, &wish);
    let has_clone = app.tanks[app.current_tank]
        .cows
        .iter()
        .any(|c| c.name == expected_clone_name);
    assert!(has_clone, "cloned cow must be named '<original>'s Clone'");
}

fn submit_command(app: &mut App, cmd: &str) {
    app.editor.set(cmd.to_string());
    app.handle_input(key_press(KeyCode::Enter));
}

#[test]
fn command_give_cash_matches_wish_outcome() {
    let mut app = App::launch(Launch::Debug);
    let before = app.purse.balance();
    submit_command(&mut app, "/give cash");
    assert_eq!(app.purse.balance(), before + GIVE_RESOURCE_AMOUNT);
}

fn held(app: &App, item: StockItem) -> u32 {
    app.inventory.get(&item).copied().unwrap_or(0)
}

fn every_stock_item() -> Vec<StockItem> {
    ConsumableKind::all()
        .into_iter()
        .map(StockItem::Consumable)
        .chain([StockItem::Junk])
        .collect()
}

#[test]
fn every_stock_item_is_gifted_by_its_rarity_through_both_doors() {
    for item in every_stock_item() {
        let name = item.display_name().to_ascii_lowercase();
        if name.trim().is_empty() {
            continue;
        }
        let gift = item.rarity().gift_quantity();

        let mut app = App::launch(Launch::Debug);
        let before = held(&app, item);
        submit_command(&mut app, &format!("/give {name}"));
        assert_eq!(
            held(&app, item),
            before + gift,
            "/give {name} must grant a {:?} gift",
            item.rarity()
        );

        let mut app = App::launch(Launch::Debug);
        let before = held(&app, item);
        submit_wish(&mut app, &format!("give {name}"));
        assert_eq!(
            held(&app, item),
            before + gift,
            "the wish for {name} must agree with /give"
        );
    }
}

#[test]
fn a_fabricator_is_gifted_like_every_other_rare_thing() {
    let fabricator = StockItem::Consumable(ConsumableKind::Fabricator);
    let rare_part = every_stock_item()
        .into_iter()
        .find(|item| {
            matches!(item, StockItem::Consumable(ConsumableKind::Part(_)))
                && item.rarity() == Rarity::Rare
        })
        .expect("the bench sells a Rare part");
    let mut app = App::launch(Launch::Debug);
    submit_command(&mut app, "/give fabricator");
    submit_command(
        &mut app,
        &format!("/give {}", rare_part.display_name().to_ascii_lowercase()),
    );
    assert_eq!(held(&app, fabricator), Rarity::Rare.gift_quantity());
    assert_eq!(held(&app, fabricator), held(&app, rare_part));
}

#[test]
fn anything_grants_two_of_the_boons_give_grants() {
    let mut app = App::launch(Launch::Debug);
    let (cash, food) = (app.purse.balance(), app.food_supply);
    let (coffee, bait) = (held(&app, StockItem::COFFEE), held(&app, StockItem::BAIT));
    submit_wish(&mut app, "anything");
    let deltas = [
        (app.purse.balance() - cash, GIVE_RESOURCE_AMOUNT),
        (app.food_supply - food, GIVE_RESOURCE_AMOUNT),
        (
            held(&app, StockItem::COFFEE) - coffee,
            StockItem::COFFEE.gift_quantity(),
        ),
        (
            held(&app, StockItem::BAIT) - bait,
            StockItem::BAIT.gift_quantity(),
        ),
    ];
    for (delta, boon) in deltas {
        assert_eq!(delta % boon, 0, "anything grants whole boons only");
    }
    let boons: u32 = deltas.iter().map(|(delta, boon)| delta / boon).sum();
    assert_eq!(boons, 2);
}

#[test]
fn command_bless_runs_without_a_target() {
    let mut app = App::launch(Launch::Debug);
    submit_command(&mut app, "/bless");
    assert!(app.tanks[0].fish.iter().all(|f| !f.name.is_empty()));
}

#[test]
fn command_clone_adds_fish_to_tank() {
    let mut app = App::launch(Launch::Debug);
    let before = app.tanks[app.current_tank].fish.len();
    let fish_name = app.tanks[0].fish[0].name.clone();
    submit_command(&mut app, &format!("/clone {}", fish_name));
    assert_eq!(app.tanks[app.current_tank].fish.len(), before + 1);
}

#[test]
fn command_expand_increases_capacity() {
    let mut app = App::launch(Launch::Debug);
    let tank_name = app.tanks[0].name.clone();
    let before = app.tanks[0].capacity();
    submit_command(&mut app, &format!("/expand {}", tank_name));
    assert_eq!(app.tanks[0].capacity(), before + EXPAND_AMOUNT as usize);
}

#[test]
fn command_revive_moves_fish_from_graveyard_to_tank() {
    let mut app = App::launch(Launch::Debug);
    let mut rng = rand::rng();
    let dead = Fish::new(
        FishSpecies::Merluza,
        "Cosmo".to_string(),
        10.0,
        10.0,
        &mut rng,
    );
    app.graveyard.push(dead);
    let fish_before = app.tanks[app.current_tank].fish.len();
    submit_command(&mut app, "/revive cosmo");
    assert!(
        app.graveyard.is_empty(),
        "graveyard must be empty after revive"
    );
    assert_eq!(app.tanks[app.current_tank].fish.len(), fish_before + 1);
}
