use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fishtank::{
    app::{App, Launch},
    economy::Purse,
    entities::cow::CowVariant,
    fishes::mutations::{Mutatable, Mutation, apply_mutation},
    fishes::species::{FishSpecies, Habitat},
    loot::Companion,
    tank::{
        ALIEN_NAME_WORDS_MAX, ALIEN_NAME_WORDS_MIN, ALIEN_SOUNDS, ALIEN_TONGUE, TankKind,
        alien_name,
    },
    void_ritual::GIVE_RESOURCE_AMOUNT,
};

const RANDOM_DRAWS: usize = 4000;
const UFO_TICK_BUDGET: usize = 5000;
const A_SCRIPT_LINE_TICKS: usize = 40;

fn enter() -> Event {
    Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
}

fn speak(app: &mut App, line: &str) {
    app.editor.set(line.to_string());
    app.handle_input(enter());
}

fn empty_app() -> App {
    let mut app = App::launch(Launch::Debug);
    app.tanks[0].fish.clear();
    app.tanks[0].used_names.clear();
    app
}

fn alien_fish(app: &mut App, name: &str) {
    app.tanks[0].spawn_fish(FishSpecies::Merluza, name.to_string(), &mut rand::rng());
    let fish = app.tanks[0]
        .fish
        .iter_mut()
        .find(|fish| fish.name == name)
        .expect("the fish was spawned");
    apply_mutation(fish, Mutation::Alienation, &mut rand::rng());
}

fn wait_for_the_ufo(app: &mut App) {
    for _ in 0..UFO_TICK_BUDGET {
        if app.tanks[0].ufos.is_empty() {
            return;
        }
        app.tick();
    }
    panic!("the mothership never finished its delivery");
}

fn speaker_bubble(app: &App, name: &str) -> Option<String> {
    app.tanks[0]
        .fish
        .iter()
        .find(|fish| fish.name == name)
        .and_then(|fish| fish.speech.as_ref())
        .map(|bubble| bubble.text.clone())
}

#[test]
fn neither_identity_mutation_is_ever_drawn_at_random() {
    let mut rng = rand::rng();
    let mut app = empty_app();
    app.tanks[0].spawn_fish(FishSpecies::Merluza, "Ann".to_string(), &mut rng);
    let fish = &app.tanks[0].fish[0];
    for _ in 0..RANDOM_DRAWS {
        let drawn = fish.random_mutation(&mut rng);
        assert!(
            !matches!(drawn, Some(Mutation::Alienation | Mutation::Strawberry)),
            "an identity is given (milk, abduction, /mutate), never rolled"
        );
    }
}

#[test]
fn an_alien_calls_home_and_the_mothership_brings_an_alien_companion() {
    let mut app = empty_app();
    alien_fish(&mut app, "Zed");
    let fish_before = app.tanks[0].fish.len();
    let cows_before = app.tanks[0].cows.len();

    speak(&mut app, "/startcallhome");

    assert_eq!(speaker_bubble(&app, "Zed").as_deref(), Some(ALIEN_TONGUE));
    assert!(!app.tanks[0].ufos.is_empty(), "the mothership answers");
    wait_for_the_ufo(&mut app);
    let tank = &app.tanks[0];
    let new_fish = tank.fish.len() - fish_before;
    let new_cows = tank.cows.len() - cows_before;
    assert!(
        new_fish + new_cows >= 1,
        "the call brings a companion (an alien may call again on its own clock meanwhile)"
    );
    for companion in tank.fish.iter().filter(|fish| fish.name != "Zed") {
        assert!(companion.is_alienated(), "a companion arrives alien");
        assert!(!companion.name.starts_with("Un"), "Un is for unfishes");
    }
}

#[test]
fn an_alien_is_named_in_its_own_tongue() {
    let mut rng = rand::rng();
    for _ in 0..RANDOM_DRAWS {
        let name = alien_name(&mut rng);
        let words: Vec<&str> = name.split(' ').collect();
        assert!((ALIEN_NAME_WORDS_MIN..=ALIEN_NAME_WORDS_MAX).contains(&words.len()));
        assert!(
            words.iter().all(|word| ALIEN_SOUNDS.contains(word)),
            "{name}"
        );
    }
}

#[test]
fn a_full_tank_calls_home_but_nobody_comes() {
    let mut app = empty_app();
    alien_fish(&mut app, "Zed");
    let mut n = 0;
    while !app.tanks[0].is_full() {
        app.tanks[0].spawn_fish(FishSpecies::Merluza, format!("Filler{n}"), &mut rand::rng());
        n += 1;
    }

    speak(&mut app, "/startcallhome");

    assert_eq!(speaker_bubble(&app, "Zed").as_deref(), Some(ALIEN_TONGUE));
    assert!(
        app.tanks[0].ufos.is_empty(),
        "a full tank has no room to answer"
    );
}

#[test]
fn two_calls_bring_two_ufos_and_two_companions() {
    let mut app = empty_app();
    alien_fish(&mut app, "Zed");
    let before = app.tanks[0].fish.len() + app.tanks[0].cows.len();

    speak(&mut app, "/startcallhome");
    speak(&mut app, "/startcallhome");

    assert_eq!(
        app.tanks[0].ufos.len(),
        2,
        "each call is answered by its own UFO"
    );
    wait_for_the_ufo(&mut app);
    let after = app.tanks[0].fish.len() + app.tanks[0].cows.len();
    assert!(after - before >= 2, "and each one delivers");
}

#[test]
fn two_abductions_take_two_different_fish() {
    let mut app = empty_app();
    for name in ["Ann", "Bob", "Cid"] {
        app.tanks[0].spawn_fish(FishSpecies::Merluza, name.to_string(), &mut rand::rng());
    }

    speak(&mut app, "/startfishabduction");
    speak(&mut app, "/startfishabduction");

    let targets: Vec<&str> = app.tanks[0]
        .ufos
        .iter()
        .filter_map(|ufo| ufo.abductee())
        .collect();
    assert_eq!(targets.len(), 2, "two calls, two abductions");
    assert_ne!(targets[0], targets[1], "never the same fish twice");
}

#[test]
fn a_mothership_call_does_not_wait_for_a_cow_delivery() {
    let mut app = empty_app();
    alien_fish(&mut app, "Zed");

    speak(&mut app, "/startcowabduction");
    speak(&mut app, "/startcallhome");

    assert_eq!(app.tanks[0].ufos.len(), 2);
}

#[test]
fn a_tank_with_no_alien_cannot_call_home() {
    let mut app = empty_app();
    app.tanks[0].spawn_fish(FishSpecies::Merluza, "Ann".to_string(), &mut rand::rng());

    speak(&mut app, "/startcallhome");

    assert!(app.tanks[0].ufos.is_empty());
    assert!(speaker_bubble(&app, "Ann").is_none());
}

#[test]
fn an_alien_cow_calls_home_too() {
    let mut app = empty_app();
    app.tanks[0].spawn_cow(CowVariant::LightGreen, &mut rand::rng());

    speak(&mut app, "/startcallhome");

    let cow = &app.tanks[0].cows[0];
    assert_eq!(
        cow.speech.as_ref().map(|bubble| bubble.text.as_str()),
        Some(ALIEN_TONGUE)
    );
    assert!(!app.tanks[0].ufos.is_empty());
}

#[test]
fn a_botfish_hears_an_alien_calling_home() {
    let mut app = empty_app();
    app.purse = Purse::holding(0);
    alien_fish(&mut app, "Zed");
    app.tanks[0].spawn_fish(FishSpecies::Botfish, "Neo".to_string(), &mut rand::rng());
    let bot = app.tanks[0]
        .fish
        .iter_mut()
        .find_map(|fish| fish.botfish_state.as_mut())
        .expect("a botfish is programmable");
    bot.program(ALIEN_TONGUE.to_string(), vec!["/give cash".to_string()]);

    speak(&mut app, "/startcallhome");
    for _ in 0..A_SCRIPT_LINE_TICKS {
        app.tick();
    }

    assert_eq!(app.purse.balance(), GIVE_RESOURCE_AMOUNT);
}

#[test]
fn a_companion_is_rolled_from_the_tanks_own_catch_weights() {
    let mut rng = rand::rng();
    let mut saw_a_cow = false;
    for _ in 0..RANDOM_DRAWS {
        match Companion::roll(TankKind::Base, &mut rng) {
            Companion::Cow(_) => saw_a_cow = true,
            Companion::Fish(species) => assert!(
                !matches!(species.config().habitat, Habitat::Native(kind) if kind != TankKind::Base),
                "{species:?} is caught only on another tank's banner"
            ),
        }
    }
    assert!(saw_a_cow, "the mothership sometimes brings a cow");
}
