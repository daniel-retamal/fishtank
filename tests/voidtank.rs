use crossterm::event::KeyCode;
use fishtank::{
    economy::Money,
    loot::{ConsumableKind, StockItem},
    tank::TankKind,
    testing::Tui,
    ui::consume_picker::ConsumePickerSource,
};

const GIFTED_VOIDTANK: &str = "UnVoidtank";

fn lab() -> Tui {
    let mut tui = Tui::new();
    tui.clear_tank();
    tui
}

fn voidtanks(tui: &Tui) -> usize {
    tui.app
        .tanks
        .iter()
        .filter(|tank| tank.kind == TankKind::Void)
        .count()
}

fn void_seeds(tui: &Tui) -> u32 {
    tui.app
        .inventory
        .get(&StockItem::VOID_SEED)
        .copied()
        .unwrap_or(0)
}

fn say_nothing(tui: &mut Tui) {
    tui.run("/cheat");
    tui.type_text("nothing");
    tui.key(KeyCode::Enter);
}

#[test]
fn there_is_only_ever_one_voidtank() {
    let mut tui = lab();
    tui.run("/give voidtank");
    tui.run("/give voidtank");
    assert_eq!(voidtanks(&tui), 1);
}

#[test]
fn a_second_void_seed_is_never_handed_over() {
    let mut tui = lab();
    say_nothing(&mut tui);
    say_nothing(&mut tui);
    assert_eq!(void_seeds(&tui), 1);
}

#[test]
fn a_held_void_seed_is_the_voidtank_to_be() {
    let mut tui = lab();
    say_nothing(&mut tui);
    tui.run("/give voidtank");
    assert_eq!(voidtanks(&tui), 0);
}

#[test]
fn no_void_seed_appears_while_a_voidtank_exists() {
    let mut tui = lab();
    tui.run("/give voidtank");
    say_nothing(&mut tui);
    assert_eq!(void_seeds(&tui), 0);
}

#[test]
fn growing_the_seed_spends_it_and_closes_both_doors() {
    let mut tui = lab();
    say_nothing(&mut tui);
    assert!(
        tui.app
            .try_consume_kind(ConsumableKind::VoidSeed, ConsumePickerSource::FromInventory)
    );
    tui.type_text("Abyss");
    tui.key(KeyCode::Enter);
    assert_eq!(voidtanks(&tui), 1);
    assert_eq!(void_seeds(&tui), 0);

    say_nothing(&mut tui);
    tui.run("/give voidtank");
    assert_eq!(voidtanks(&tui), 1);
    assert_eq!(void_seeds(&tui), 0);
}

#[test]
fn the_voidtank_can_be_sold_and_then_the_void_seed_comes_back() {
    let mut tui = lab();
    tui.run("/give voidtank");
    let cash = tui.app.purse.balance();

    tui.run(&format!("/sell tank \"{GIFTED_VOIDTANK}\""));

    assert_eq!(voidtanks(&tui), 0);
    assert_eq!(
        tui.app.purse.balance(),
        cash + Money::from(TankKind::Void.sell_price())
    );
    say_nothing(&mut tui);
    assert_eq!(void_seeds(&tui), 1);
}
