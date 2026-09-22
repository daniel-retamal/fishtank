use std::path::Path;

use fishtank::{
    app::{App, Launch},
    cheats::{CHEAT_RESOURCE_AMOUNT, Cheat, Switch},
    colors::{RED, WHITE},
    fishes::{
        fish::Fish,
        fused::FusedComponent,
        species::{CHEATFISH_EYES, FishSpecies},
    },
    loot::StockItem,
    restore::Restorable,
    testing::Tui,
    void_ritual::parse_give_target,
};

const COLS: u16 = 100;
const ROWS: u16 = 30;
const REEL_DIR: &str = env!("CARGO_TARGET_TMPDIR");
const FILM_SIZES: [(u16, u16); 4] = [(100, 30), (60, 18), (40, 14), (28, 10)];

fn player() -> Tui {
    let mut tui = Tui::as_player(COLS, ROWS);
    tui.clear_tank();
    tui
}

fn cheatfish(app: &App) -> Vec<&Fish> {
    app.tanks
        .iter()
        .flat_map(|tank| tank.fish.iter())
        .filter(|fish| fish.species == FishSpecies::Cheatfish)
        .collect()
}

fn enter(tui: &mut Tui, code: &str) {
    tui.run("/cheat");
    tui.type_text(code);
    tui.key(crossterm::event::KeyCode::Enter);
}

#[test]
fn a_player_cannot_reach_a_god_or_a_debug_command() {
    let mut tui = player();
    let cash = tui.app.purse.balance();
    tui.run("/spawn merluza \"Dory\"");
    tui.run("/give cash");
    tui.run("/add cash 500");
    assert!(tui.app.tanks[0].fish.is_empty());
    assert_eq!(tui.app.purse.balance(), cash);
}

#[test]
fn the_cheat_popup_asks_for_a_code_and_only_offers_enter_for_a_real_one() {
    let mut tui = player();
    tui.run("/cheat");
    let screen = tui.screen();
    screen.expect_find("Enter cheat code");
    screen.expect_find("ESC close");
    screen.expect_absent("ENTER cheat");
    screen.expect_absent("invalid cheat code");
    tui.type_text("show me the mone");
    tui.screen().expect_find("invalid cheat code");
    tui.type_text("Y");
    let screen = tui.screen();
    screen.expect_find("ENTER cheat");
    screen.expect_absent("invalid cheat code");
}

#[test]
fn entering_nothing_hands_over_the_void_seed() {
    let mut tui = player();
    tui.run("/cheat");
    tui.type_text("nada");
    tui.key(crossterm::event::KeyCode::Enter);
    assert_eq!(tui.app.inventory.get(&StockItem::VOID_SEED), Some(&1));
    assert_eq!(cheatfish(&tui.app)[0].name, "Nothing");
}

#[test]
fn enter_on_a_wrong_code_does_nothing_and_esc_closes() {
    let mut tui = player();
    tui.run("/cheat");
    tui.type_text("power underwhelming");
    tui.key(crossterm::event::KeyCode::Enter);
    tui.screen().expect_find("Enter cheat code");
    tui.key(crossterm::event::KeyCode::Esc);
    tui.screen().expect_absent("Enter cheat code");
    assert!(cheatfish(&tui.app).is_empty());
}

#[test]
fn show_me_the_money_pays_and_leaves_an_unsellable_cheatfish_behind() {
    let mut tui = player();
    let cash = tui.app.purse.balance();
    enter(&mut tui, "SHOW me   the money");
    assert_eq!(tui.app.purse.balance(), cash + CHEAT_RESOURCE_AMOUNT);
    let minted = cheatfish(&tui.app);
    assert_eq!(minted.len(), 1);
    assert_eq!(minted[0].name, Cheat::ShowMeTheMoney.fish_name());
    let before = tui.app.purse.balance();
    tui.run("/sell fish \"Show Me The Money\"");
    assert_eq!(
        cheatfish(&tui.app).len(),
        1,
        "a cheatfish can never be sold"
    );
    assert_eq!(tui.app.purse.balance(), before);
}

#[test]
fn a_cheatfish_never_appears_in_the_sell_menu() {
    let mut tui = player();
    enter(&mut tui, "show me the money");
    tui.run("/shop");
    tui.key(crossterm::event::KeyCode::Down);
    tui.key(crossterm::event::KeyCode::Enter);
    tui.screen().expect_absent("Show Me The Money");
}

#[test]
fn bend_the_light_makes_the_purse_endless_until_it_is_bent_back() {
    let mut tui = player();
    let cash = tui.app.purse.balance();
    enter(&mut tui, "bend the light");
    tui.screen().expect_find("cash: ∞");
    tui.run("/buy coffee 1000");
    assert_eq!(tui.app.purse.balance(), cash, "nothing was charged");
    enter(&mut tui, "bend the light");
    tui.screen().expect_absent("∞");
    assert_eq!(
        cheatfish(&tui.app).len(),
        1,
        "switching a cheat off is not cheating"
    );
}

#[test]
fn food_for_thought_takes_the_lid_off_every_tank_old_and_new() {
    let mut tui = player();
    enter(&mut tui, "food for thought");
    tui.screen().expect_find("/∞");
    assert!(tui.app.tanks.iter().all(|tank| !tank.is_full()));
    enter(&mut tui, "highway to hell");
    tui.run("/consume necronomicon");
    tui.type_text("Pit");
    tui.key(crossterm::event::KeyCode::Enter);
    assert!(tui.app.tanks.iter().all(|tank| tank.boundless));
}

#[test]
fn godmode_opens_the_god_commands_and_each_one_mints_a_cheatfish() {
    let mut tui = player();
    enter(&mut tui, "fish to the fishtank");
    tui.screen().expect_find("godmode");
    tui.run("/spawn merluza \"Dory\"");
    tui.run("/kill \"Dory\"");
    assert_eq!(cheatfish(&tui.app).len(), 3);
    tui.run("/give cash");
    assert_eq!(cheatfish(&tui.app).len(), 3, "godmode is not debug mode");
    tui.run("/kill \"Nobody\"");
    assert_eq!(
        cheatfish(&tui.app).len(),
        3,
        "a god command that did nothing costs nothing"
    );
}

#[test]
fn debug_mode_opens_everything_and_cheats_leave_no_fish() {
    let mut tui = player();
    tui.run("!debugmode");
    tui.screen().expect_find("debug mode");
    tui.run("/spawn merluza \"Dory\"");
    tui.run("/give cash");
    enter(&mut tui, "show me the money");
    assert!(cheatfish(&tui.app).is_empty());
    tui.run("!DEBUGMODE");
    tui.screen().expect_absent("debug mode");
    tui.run("/spawn merluza \"Nemo\"");
    assert_eq!(tui.app.tanks[0].fish.len(), 1);
}

#[test]
fn the_debug_flag_launches_the_game_in_debug_mode() {
    let args = ["fishtank".to_string(), Launch::DEBUG_FLAG.to_string()];
    assert_eq!(Launch::from_args(args.into_iter()), Launch::Debug);
    assert_eq!(
        Launch::from_args(["fishtank".to_string()].into_iter()),
        Launch::Player
    );
    assert!(App::launch(Launch::Debug).debug_mode);
    assert!(!App::new().debug_mode);
}

#[test]
fn every_cheat_does_something_on_a_fresh_game() {
    for &cheat in Cheat::ALL {
        let mut app = App::new();
        if cheat == Cheat::StayingAlive {
            app.enter_cheat(Cheat::ShowMeTheMoney);
            let victim = app.tanks[0].fish[0].name.clone();
            app.enter_cheat(Cheat::FishToTheFishtank);
            app.editor.set(format!("/kill \"{victim}\""));
            app.handle_input(crossterm::event::Event::Key(
                crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            ));
        }
        assert!(app.enter_cheat(cheat), "{} did nothing", cheat.code());
    }
}

#[test]
fn a_switch_cheat_is_undone_by_its_own_code() {
    for &switch in Switch::ALL {
        let mut app = App::new();
        let cheat = Cheat::of_switch(switch);
        app.enter_cheat(cheat);
        assert!(app.is_switched_on(switch));
        app.enter_cheat(cheat);
        assert!(!app.is_switched_on(switch));
    }
}

#[test]
fn a_bot_cannot_open_the_cheat_popup_or_flip_debug_mode() {
    let mut tui = Tui::new();
    tui.clear_tank();
    tui.run("/spawn botfish \"Neo\"");
    tui.app.tanks[0].fish[0]
        .script_mut()
        .expect("a botfish is programmable")
        .program(
            "wake up".to_string(),
            vec!["!debugmode".to_string(), "/cheat".to_string()],
        );
    tui.run("!debugmode");
    tui.run("wake up");
    tui.tick_n(200);
    assert!(!tui.app.debug_mode);
    tui.screen().expect_absent("Enter cheat code");
}

#[test]
fn a_cheatfish_is_white_with_three_red_eyes_and_keeps_them_when_restored() {
    let mut fish = Fish::new(
        FishSpecies::Cheatfish,
        "Cid".to_string(),
        0.0,
        0.0,
        &mut rand::rng(),
    );
    let eyes = |fish: &Fish| {
        fish.segments()
            .iter()
            .filter(|(_, color)| *color == RED)
            .count()
    };
    assert_eq!(eyes(&fish), CHEATFISH_EYES);
    assert_eq!(fish.color, WHITE);
    fish.restore();
    assert_eq!(eyes(&fish), CHEATFISH_EYES);
}

#[test]
fn a_fish_carrying_a_cheatfish_inside_it_cannot_be_sold() {
    let mut rng = rand::rng();
    let mut carrier = Fish::new(FishSpecies::Merluza, "Host".to_string(), 0.0, 0.0, &mut rng);
    assert!(carrier.is_sellable());
    fishtank::fishes::mutations::apply_mutation_to_fish(
        &mut carrier,
        fishtank::fishes::mutations::Mutation::ColorPatch,
        &mut rng,
    );
    carrier.mutant.as_mut().expect("mutated").fused = vec![
        FusedComponent::fish(FishSpecies::Merluza, "Host".to_string(), 1),
        FusedComponent::fish(FishSpecies::Cheatfish, "Cid".to_string(), 1),
    ];
    assert!(!carrier.is_sellable());
    assert_eq!(carrier.sell_value(), 0);
}

#[test]
fn no_wish_or_gift_hands_out_a_cheatfish() {
    assert!(parse_give_target("cheatfish").is_none());
    assert!(parse_give_target("merluza").is_some());
}

#[test]
fn the_cheat_popup_is_filmed_at_every_size() {
    for (cols, rows) in FILM_SIZES {
        let mut tui = Tui::as_player(cols, rows);
        tui.film(Path::new(REEL_DIR), &format!("cheat-popup-{cols}x{rows}"));
        tui.run("/cheat");
        tui.snap("an empty cheat code");
        tui.type_text("food for thougt");
        tui.snap("a typo reads as invalid");
        tui.key(crossterm::event::KeyCode::Backspace);
        tui.type_text("ht");
        tui.snap("a real code offers ENTER");
        tui.screen().expect_find("ENTER cheat");
        tui.key(crossterm::event::KeyCode::Enter);
        tui.snap("the lid is off the tank");
        tui.screen().expect_find("∞");
        assert!(
            tui.reel().flaw_report().is_empty(),
            "{cols}x{rows}:\n{}",
            tui.reel().flaw_report()
        );
    }
}
