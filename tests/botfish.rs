use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fishtank::{
    app::{App, Launch},
    economy::{Money, Purse},
    entities::cow::CowVariant,
    fishes::species::{FishSpecies, Habitat},
    tank::Tank,
    tank::TankKind,
    void_ritual::GIVE_RESOURCE_AMOUNT,
};

fn enter() -> Event {
    Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
}

fn speak(app: &mut App, line: &str) {
    app.editor.set(line.to_string());
    app.handle_input(enter());
}

fn spawn_botfish(app: &mut App, tank_idx: usize, name: &str, trigger: &str, script: &[&str]) {
    app.tanks[tank_idx].spawn_fish(FishSpecies::Botfish, name.to_string(), &mut rand::rng());
    let bot = app.tanks[tank_idx]
        .fish
        .iter_mut()
        .find_map(|f| f.botfish_state.as_mut())
        .expect("botfish should carry programmable state");
    bot.program(
        trigger.to_string(),
        script.iter().map(|s| s.to_string()).collect(),
    );
}

fn tick_n(app: &mut App, n: usize) {
    for _ in 0..n {
        app.tick();
    }
}

#[test]
fn botfish_species_is_a_special_legendary() {
    let config = FishSpecies::Botfish.config();
    assert!(config.programmable);
    assert!(
        !config.buyable,
        "a botfish is a Legendary: caught on its banner, never bought"
    );
    assert_eq!(
        config.habitat,
        Habitat::Native(TankKind::Matrix),
        "a botfish is caught only on its banner, the Matrixtank"
    );
    assert!(config.mutatable);
}

#[test]
fn player_speech_runs_the_botfish_script() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    spawn_botfish(&mut app, 0, "Neo", "wake up", &["/give cash"]);

    speak(&mut app, "wake up");
    tick_n(&mut app, 40);

    assert_eq!(app.purse.balance(), Money::from(GIVE_RESOURCE_AMOUNT));
}

#[test]
fn non_matching_speech_does_not_run_the_script() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    spawn_botfish(&mut app, 0, "Neo", "wake up", &["/give cash"]);

    speak(&mut app, "follow the white rabbit");
    tick_n(&mut app, 40);

    assert_eq!(app.purse.balance(), 0);
}

#[test]
fn a_failing_command_aborts_the_rest_of_the_script() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    spawn_botfish(
        &mut app,
        0,
        "Neo",
        "wake up",
        &["/buy necronomicon 4500", "/give cash"],
    );

    speak(&mut app, "wake up");
    tick_n(&mut app, 80);

    assert_eq!(
        app.purse.balance(),
        0,
        "the give must never run after the bad buy"
    );
}

#[test]
fn cowsay_triggers_a_botfish_in_the_same_tank() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    app.tanks[0].spawn_cow(CowVariant::WhiteBlack, &mut rand::rng());
    spawn_botfish(&mut app, 0, "Neo", "moo", &["/give cash"]);

    speak(&mut app, "/cowsay moo");
    tick_n(&mut app, 40);

    assert_eq!(app.purse.balance(), Money::from(GIVE_RESOURCE_AMOUNT));
}

#[test]
fn cowsay_does_not_trigger_a_botfish_in_another_tank() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    app.tanks[0].spawn_cow(CowVariant::WhiteBlack, &mut rand::rng());
    app.tanks
        .push(Tank::new("Annex".to_string(), TankKind::Base, &[]));
    spawn_botfish(&mut app, 1, "Neo", "moo", &["/give cash"]);

    speak(&mut app, "/cowsay moo");
    tick_n(&mut app, 40);

    assert_eq!(
        app.purse.balance(),
        0,
        "a cow only speaks to bots in its own tank"
    );
}

#[test]
fn a_typed_say_triggers_a_botfish_in_the_tank_you_are_watching() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    spawn_botfish(&mut app, 0, "Neo", "wake up", &["/give cash"]);

    speak(&mut app, "/say \"wake up\"");
    tick_n(&mut app, 40);

    assert_eq!(app.purse.balance(), Money::from(GIVE_RESOURCE_AMOUNT));
}

#[test]
fn a_typed_say_does_not_carry_to_another_tank() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    app.tanks
        .push(Tank::new("Annex".to_string(), TankKind::Base, &[]));
    spawn_botfish(&mut app, 1, "Neo", "wake up", &["/give cash"]);

    speak(&mut app, "/say \"wake up\"");
    tick_n(&mut app, 40);

    assert_eq!(
        app.purse.balance(),
        0,
        "speech is heard in the room it was said in"
    );
}

#[test]
fn a_typed_say_with_nothing_to_say_does_nothing() {
    let mut app = App::launch(Launch::Debug);
    app.purse = Purse::holding(0);
    spawn_botfish(&mut app, 0, "Neo", "", &["/give cash"]);

    speak(&mut app, "/say \"\"");
    tick_n(&mut app, 40);

    assert_eq!(app.purse.balance(), 0, "an empty trigger answers no one");
}
