use fishtank::fishes::fish::Fish;
use fishtank::fishes::species::FishSpecies;
use fishtank::fishes::unfish::UnfishKind;
use fishtank::tank::{Tank, TankKind};

fn make_tank() -> Tank {
    Tank::new("Test".to_string(), TankKind::Base, &[])
}

fn rng() -> impl rand::RngExt {
    rand::rng()
}

#[test]
fn anchoveta_eye_plus_sets_display_width_and_left_eye() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Anchoveta, "Tiny".to_string(), &mut rng);
    tank.apply_named_mutation("Tiny", "eye+");
    let fish = &tank.fish[0];
    assert_eq!(fish.display_width, 4);
    let m = fish
        .mutant
        .as_ref()
        .expect("mutant should exist after eye+");
    assert_eq!(m.left_eyes.len(), 1);
}

#[test]
fn anchoveta_size_plus_sets_display_width_and_body_size() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Anchoveta, "Tiny".to_string(), &mut rng);
    tank.apply_named_mutation("Tiny", "size+");
    let fish = &tank.fish[0];
    assert_eq!(fish.display_width, 4);
    assert_eq!(fish.body_size, 2);
}

#[test]
fn anchoveta_doublefish_sets_is_double_and_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Anchoveta, "Tiny".to_string(), &mut rng);
    tank.apply_named_mutation("Tiny", "doublefish");
    let fish = &tank.fish[0];
    assert_eq!(fish.display_width, 4);
    let m = fish
        .mutant
        .as_ref()
        .expect("mutant should exist after doublefish");
    assert!(m.is_double);
    assert!(
        m.double_head_eyes.is_empty(),
        "anchoveta has no eyes, so the second head must also have none"
    );
}

#[test]
fn standard_fish_doublefish_has_one_double_head_eye() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Merluza, "Bubbles".to_string(), &mut rng);
    tank.apply_named_mutation("Bubbles", "doublefish");
    let m = tank.fish[0]
        .mutant
        .as_ref()
        .expect("mutant should exist after doublefish");
    assert!(m.is_double);
    assert_eq!(
        m.double_head_eyes.len(),
        1,
        "standard fish start with 1 eye, so the second head must also have exactly 1"
    );
}

#[test]
fn anchoveta_bodycolor_changes_fish_color() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Anchoveta, "Tiny".to_string(), &mut rng);
    let color_before = tank.fish[0].color;
    tank.apply_named_mutation("Tiny", "bodycolor");
    let color_after = tank.fish[0].color;
    assert_ne!(
        color_before, color_after,
        "bodycolor mutation should change fish color"
    );
}

#[test]
fn anchoveta_bodycolor_sets_glistening_color_on_mutant() {
    let mut tank = make_tank();
    let mut rng = rng();
    tank.spawn_fish(FishSpecies::Anchoveta, "Tiny".to_string(), &mut rng);
    tank.apply_named_mutation("Tiny", "bodycolor");
    let fish = &tank.fish[0];
    assert!(
        fish.mutant.is_some(),
        "mutant should exist after bodycolor mutation"
    );
}

#[test]
fn ball_unfish_eyecolor_mutation_sets_slime_eye_color() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "eyecolor");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(us.slime_eye_color.is_some());
}

#[test]
fn ball_unfish_bodycolor_mutation_sets_slime_body_color() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "bodycolor");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(us.slime_body_color.is_some());
}

#[test]
fn ball_unfish_eyecolor_increments_mutation_count() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "eyecolor");
    let record = tank.fish[0]
        .mutations
        .as_ref()
        .expect("mutation record must exist");
    assert_eq!(record.count, 1);
}

#[test]
fn ball_unfish_bodycolor_adds_to_mutation_history() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "bodycolor");
    let record = tank.fish[0]
        .mutations
        .as_ref()
        .expect("mutation record must exist");
    assert_eq!(record.history.len(), 1);
    assert_eq!(record.history[0], "bodycolor");
}

#[test]
fn worm_mitosis_both_fish_have_partner_name() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Worm, "Wiggly".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Wiggly".to_string(), &mut rng);
    tank.apply_named_mutation("Wiggly", "doublefish");
    tank.apply_named_mutation("Wiggly", "mitosis");
    assert_eq!(tank.fish.len(), 2, "mitosis should produce a second fish");
    let orig = &tank.fish[0];
    let child = &tank.fish[1];
    let orig_record = orig
        .mutations
        .as_ref()
        .expect("orig must have a mutation record");
    let child_record = child
        .mutations
        .as_ref()
        .expect("child must have a mutation record");
    assert!(
        orig_record.partners.contains(&child.name),
        "original fish must list child as mitosis partner"
    );
    assert!(
        child_record.partners.contains(&orig.name),
        "child fish must list original as mitosis partner"
    );
}

#[test]
fn worm_mitosis_without_doublefish_does_not_split() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Worm, "Wiggly".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Wiggly".to_string(), &mut rng);
    tank.apply_named_mutation("Wiggly", "mitosis");
    assert_eq!(
        tank.fish.len(),
        1,
        "mitosis on single worm should not spawn a second fish"
    );
}

#[test]
fn ball_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to ball unfish"
    );
}

#[test]
fn skull_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Skull,
        "Skelly".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Skelly".to_string(), &mut rng);
    tank.apply_named_mutation("Skelly", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to skull unfish"
    );
}

#[test]
fn ball_unfish_colorpatch_increments_mutation_count() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "colorpatch");
    let record = tank.fish[0]
        .mutations
        .as_ref()
        .expect("mutation record must exist");
    assert_eq!(record.count, 1);
}

#[test]
fn ball_unfish_colorpatch_adds_to_mutation_history() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "colorpatch");
    let record = tank.fish[0]
        .mutations
        .as_ref()
        .expect("mutation record must exist");
    assert_eq!(record.history.len(), 1);
    assert_eq!(record.history[0], "colorpatch");
}

#[test]
fn ball_unfish_colorpatch_patches_within_ball_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Ball, "Orb".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Orb".to_string(), &mut rng);
    tank.apply_named_mutation("Orb", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < fishtank::fishes::unfish::BALL_WIDTH as usize,
            "colorpatch pos {pos} must be within BALL_WIDTH"
        );
    }
}

#[test]
fn skull_unfish_colorpatch_patches_within_skull_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Skull,
        "Skelly".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Skelly".to_string(), &mut rng);
    tank.apply_named_mutation("Skelly", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < fishtank::fishes::unfish::SKULL_WIDTH as usize,
            "colorpatch pos {pos} must be within SKULL_WIDTH"
        );
    }
}

#[test]
fn worm_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Worm, "Wiggly".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Wiggly".to_string(), &mut rng);
    tank.apply_named_mutation("Wiggly", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to worm unfish"
    );
}

#[test]
fn worm_unfish_colorpatch_patches_within_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(UnfishKind::Worm, "Wiggly".to_string(), 20.0, 10.0, &mut rng);
    tank.place_fish(fish, "Wiggly".to_string(), &mut rng);
    tank.apply_named_mutation("Wiggly", "colorpatch");
    let dw = tank.fish[0].display_width;
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < dw,
            "colorpatch pos {pos} must be within display_width {dw}"
        );
    }
}

#[test]
fn reversed_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Reversed,
        "Flip".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Flip".to_string(), &mut rng);
    tank.apply_named_mutation("Flip", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to reversed unfish"
    );
}

#[test]
fn reversed_unfish_colorpatch_patches_within_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Reversed,
        "Flip".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Flip".to_string(), &mut rng);
    tank.apply_named_mutation("Flip", "colorpatch");
    let dw = tank.fish[0].display_width;
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < dw,
            "colorpatch pos {pos} must be within display_width {dw}"
        );
    }
}

#[test]
fn blinker_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Blinker,
        "Blink".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Blink".to_string(), &mut rng);
    tank.apply_named_mutation("Blink", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to blinker unfish"
    );
}

#[test]
fn blinker_unfish_colorpatch_patches_within_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Blinker,
        "Blink".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Blink".to_string(), &mut rng);
    tank.apply_named_mutation("Blink", "colorpatch");
    let dw = tank.fish[0].display_width;
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < dw,
            "colorpatch pos {pos} must be within display_width {dw}"
        );
    }
}

#[test]
fn doppleganger_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Doppleganger,
        "Doppel".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Doppel".to_string(), &mut rng);
    tank.apply_named_mutation("Doppel", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to doppleganger unfish"
    );
}

#[test]
fn doppleganger_unfish_colorpatch_patches_within_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Doppleganger,
        "Doppel".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Doppel".to_string(), &mut rng);
    tank.apply_named_mutation("Doppel", "colorpatch");
    let dw = tank.fish[0].display_width;
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < dw,
            "colorpatch pos {pos} must be within display_width {dw}"
        );
    }
}

#[test]
fn phantom_unfish_colorpatch_adds_slime_color_patches() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Phantom,
        "Ghost".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Ghost".to_string(), &mut rng);
    tank.apply_named_mutation("Ghost", "colorpatch");
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    assert!(
        !us.slime_color_patches.is_empty(),
        "colorpatch mutation must add at least one patch to phantom unfish"
    );
}

#[test]
fn phantom_unfish_colorpatch_patches_within_display_width() {
    let mut tank = make_tank();
    let mut rng = rng();
    let fish = Fish::new_unfish(
        UnfishKind::Phantom,
        "Ghost".to_string(),
        20.0,
        10.0,
        &mut rng,
    );
    tank.place_fish(fish, "Ghost".to_string(), &mut rng);
    tank.apply_named_mutation("Ghost", "colorpatch");
    let dw = tank.fish[0].display_width;
    let us = tank.fish[0]
        .unfish_state
        .as_ref()
        .expect("unfish_state must exist");
    for &(pos, _) in &us.slime_color_patches {
        assert!(
            pos < dw,
            "colorpatch pos {pos} must be within display_width {dw}"
        );
    }
}
