use std::collections::HashSet;

use fishtank::consumable::{CASTS_PER_BUFF, MilkStatus};
use fishtank::entities::cow::CowVariant;
use fishtank::loot::MilkVariant;
use fishtank::testing::Tui;

#[test]
fn every_fishing_buff_has_exactly_one_milk() {
    let granted: Vec<MilkStatus> = MilkVariant::ALL
        .iter()
        .filter_map(|milk| milk.status())
        .collect();
    let distinct: HashSet<MilkStatus> = granted.iter().copied().collect();
    assert_eq!(granted.len(), distinct.len(), "no two milks share a buff");
    assert_eq!(
        distinct.len(),
        MilkStatus::ALL.len(),
        "every buff is some milk's"
    );
}

#[test]
fn drinking_a_fishing_milk_grants_its_own_buff_and_nothing_else() {
    for &milk in MilkVariant::ALL {
        let Some(status) = milk.status() else {
            continue;
        };
        let mut tui = Tui::new();
        let name = milk.display_name().to_ascii_lowercase();
        tui.run(&format!("/give {name}"));

        tui.run(&format!("/consume {name}"));

        let statuses: Vec<MilkStatus> = tui.app.active_statuses.iter().map(|s| s.kind).collect();
        assert_eq!(statuses, vec![status], "{name} grants its one buff");
        tui.screen().expect_find(status.display_name());
        tui.screen().expect_absent("to the fishes");
    }
}

#[test]
fn a_second_glass_is_a_second_stack_with_its_own_casts() {
    let mut tui = Tui::new();
    tui.run("/give blueberry milk");

    tui.run("/consume blueberry milk");
    tui.run("/consume blueberry milk");

    let statuses = &tui.app.active_statuses;
    assert_eq!(statuses.len(), 2);
    for status in statuses {
        assert_eq!(status.kind, MilkStatus::VisualCalculus);
        assert_eq!(status.stacks, 1);
        assert_eq!(status.casts_left, CASTS_PER_BUFF);
    }
    tui.screen().expect_find("visual-calculus II: 5 casts");
}

#[test]
fn every_milk_but_irradiated_comes_from_its_own_cow() {
    let from_cows: Vec<MilkVariant> = CowVariant::ALL.iter().map(|cow| cow.milk()).collect();
    for &milk in MilkVariant::ALL {
        let cows = from_cows.iter().filter(|&&m| m == milk).count();
        let expected = usize::from(milk != MilkVariant::Irradiated);
        assert_eq!(
            cows,
            expected,
            "{} comes from {expected} cow(s); irradiated milk comes from the tank",
            milk.display_name()
        );
    }
}

#[test]
fn every_cow_parses_back_from_its_display_name() {
    for &cow in CowVariant::ALL {
        assert_eq!(CowVariant::parse(cow.display_name()), Some(cow));
    }
}
