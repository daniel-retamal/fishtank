use rand::RngExt;

use crate::fishes::fish::{Fish, compute_display_width};
use crate::fishes::mutant::{
    EXTRA_BODY_FOR_DOUBLE, EyeState, MIN_BODY_CHARS, MutantState, MutantTail, MutationRecord,
    random_rgb,
};
use crate::fishes::mutations::{
    MUTATION_PATCH_COUNT_MAX, MUTATION_PATCH_COUNT_MIN, MUTATION_PATCH_MAX, Mutation,
    apply_mutation_to_fish, pick_random_miracle_mutation, pick_random_mutation,
    tail_kind_to_mutant_tail,
};
use crate::fishes::species::{BodyTemplate, FishSpecies};
use crate::fishes::unfish::{
    SLIME_GLISTEN_SPEED_FAST, SLIME_GLISTEN_SPEED_SLOW, UnfishKind, worm_display_width,
};
use crate::util::{hyperbolic_scale, sample_exponential};

use super::{
    MIN_SPLIT_BODY_SIZE, MUTATION_ALPHA, MUTATION_INTERVAL_BASE, MUTATION_MEAN_FLOOR_SECS,
};
use super::{Tank, TankKind};

impl Tank {
    pub(super) fn tick_mutations(&mut self, dt: f32) {
        let mutant_count = self
            .fish
            .iter()
            .filter(|f| f.species == FishSpecies::Mutantfish)
            .count();
        if mutant_count == 0 {
            return;
        }
        self.mutation_timer -= dt;
        if self.mutation_timer <= 0.0 {
            let mut rng = rand::rng();
            let mean =
                hyperbolic_scale(MUTATION_INTERVAL_BASE, mutant_count as u32, MUTATION_ALPHA)
                    .max(MUTATION_MEAN_FLOOR_SECS);
            self.mutation_timer = sample_exponential(&mut rng, mean);
            self.apply_random_mutation();
        }
    }

    fn apply_random_mutation(&mut self) {
        let mutant_indices: Vec<usize> = self
            .fish
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                if f.species.config().auto_mutate {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        if mutant_indices.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let fish_idx = mutant_indices[rng.random_range(0..mutant_indices.len())];
        let mutation = pick_random_mutation(&self.fish[fish_idx], &mut rng);
        if matches!(mutation, Mutation::Mitosis) {
            self.apply_mitosis(fish_idx);
        } else {
            apply_mutation_to_fish(&mut self.fish[fish_idx], mutation, &mut rng);
        }
    }

    fn apply_mitosis(&mut self, idx: usize) {
        if matches!(
            self.fish[idx].species.config().body,
            BodyTemplate::Fixed { .. }
        ) {
            self.apply_fixed_body_mitosis(idx);
            return;
        }
        let is_double = self.fish[idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.is_double);
        if !is_double {
            return;
        }

        let parent_body_size = self.fish[idx].body_size;
        let parent_color = self.fish[idx].color;
        let parent_sway_speed = self.fish[idx].sway_speed;
        let spawn_x = self.fish[idx].position.x;
        let spawn_y = self.fish[idx].position.y;
        let parent_name = self.fish[idx].name.clone();

        let (
            parent_max_eyes,
            double_eye_count,
            half,
            other_half,
            new_patches,
            body_variant,
            glistening_mode,
            glistening_color,
            eye_color,
        ) = {
            let mutant = self.fish[idx].mutant.as_ref().unwrap();
            let max_eyes = mutant.left_eyes.len().max(mutant.right_eyes.len());
            let double_eye_count = mutant.double_head_eyes.len().max(1);

            let total_body = parent_body_size + EXTRA_BODY_FOR_DOUBLE;
            let raw_half = total_body / 2;
            let min_half = (max_eyes + MIN_BODY_CHARS).max(MIN_SPLIT_BODY_SIZE);
            let min_other = (double_eye_count + MIN_BODY_CHARS).max(MIN_SPLIT_BODY_SIZE);
            let half = raw_half.max(min_half);
            let other_half = (total_body - raw_half).max(min_other);

            let split_pos = 1 + max_eyes + half;
            let body_start_new = 1 + double_eye_count;
            let new_patches: Vec<_> = mutant
                .color_patches
                .iter()
                .filter(|&&(pos, _)| pos >= split_pos && pos < split_pos + other_half)
                .map(|&(pos, c)| (pos - split_pos + body_start_new, c))
                .collect();

            (
                max_eyes,
                double_eye_count,
                half,
                other_half,
                new_patches,
                mutant.body_variant,
                mutant.glistening_mode,
                mutant.glistening_color,
                mutant.eye_color,
            )
        };
        let parent_mutation_count = self.fish[idx].mutations.as_ref().map_or(0, |mr| mr.count);

        let mut rng = rand::rng();
        let tail_for_new = match rng.random_range(0u32..3) {
            0 => MutantTail::Wide,
            1 => MutantTail::Swaying,
            _ => MutantTail::Curly,
        };

        {
            let fish = &mut self.fish[idx];
            fish.body_size = half;
            let mutant = fish.mutant.as_mut().unwrap();
            mutant.is_double = false;
            mutant.double_head_eyes.clear();
            let split_pos_orig = 1 + parent_max_eyes + half;
            mutant
                .color_patches
                .retain(|&(pos, _)| pos < split_pos_orig);
            fish.display_width = mutant.display_width(half);
        }

        let new_name = self.unique_name(&parent_name);
        let mut new_fish = Fish::new(
            FishSpecies::Mutantfish,
            new_name.clone(),
            spawn_x,
            spawn_y,
            &mut rng,
        );
        new_fish.color = parent_color;
        new_fish.sway_speed = parent_sway_speed;
        new_fish.body_size = other_half;
        {
            let mutant = new_fish.mutant.as_mut().unwrap();
            mutant.body_variant = body_variant;
            mutant.glistening_mode = glistening_mode;
            mutant.glistening_color = glistening_color;
            mutant.eye_color = eye_color;
            mutant.tail_variant = tail_for_new;
            mutant.color_patches = new_patches;
            mutant.is_double = false;
            mutant.double_head_eyes.clear();
            mutant.left_eyes.clear();
            mutant.right_eyes.clear();
            for _ in 0..double_eye_count {
                mutant.left_eyes.push(EyeState::new(&mut rng));
                mutant.right_eyes.push(EyeState::new(&mut rng));
            }
        }
        if parent_mutation_count > 0 {
            new_fish
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()))
                .count = parent_mutation_count;
        }
        new_fish.display_width = new_fish.mutant.as_ref().unwrap().display_width(other_half);
        if self.kind == TankKind::Hell {
            new_fish.devil_marked = true;
        }

        self.used_names.insert(new_name);
        self.fish.push(new_fish);

        let new_idx = self.fish.len() - 1;
        let new_fish_name = self.fish[new_idx].name.clone();
        self.fish[idx]
            .mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
            .partners
            .push(new_fish_name);
        let parent_name = self.fish[idx].name.clone();
        self.fish[new_idx]
            .mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
            .partners
            .push(parent_name);
    }

    fn apply_fixed_body_mitosis(&mut self, idx: usize) {
        let is_double = self.fish[idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.is_double);
        if !is_double {
            return;
        }
        let species = self.fish[idx].species;
        let parent_name = self.fish[idx].name.clone();
        let spawn_x = self.fish[idx].position.x;
        let spawn_y = self.fish[idx].position.y;
        let parent_color = self.fish[idx].color;
        let parent_sway_speed = self.fish[idx].sway_speed;
        let (glistening_mode, glistening_color, eye_color) = {
            let mutant = self.fish[idx].mutant.as_ref().unwrap();
            (
                mutant.glistening_mode,
                mutant.glistening_color,
                mutant.eye_color,
            )
        };
        let mut_count = self.fish[idx].mutations.as_ref().map_or(0, |mr| mr.count);
        {
            let mutant = self.fish[idx].mutant.as_mut().unwrap();
            mutant.is_double = false;
            mutant.double_head_eyes.clear();
        }
        self.fish[idx].display_width = compute_display_width(species, self.fish[idx].body_size);
        let mut rng = rand::rng();
        let new_name = self.unique_name(&parent_name);
        let mut new_fish =
            crate::fishes::fish::Fish::new(species, new_name.clone(), spawn_x, spawn_y, &mut rng);
        new_fish.color = parent_color;
        new_fish.sway_speed = parent_sway_speed;
        new_fish.display_width = compute_display_width(species, new_fish.body_size);
        let mut new_mutant = MutantState::new_for_standard(MutantTail::Wide, &mut rng);
        new_mutant.glistening_mode = glistening_mode;
        new_mutant.glistening_color = glistening_color;
        new_mutant.eye_color = eye_color;
        new_fish.mutant = Some(Box::new(new_mutant));
        if mut_count > 0 {
            new_fish
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()))
                .count = mut_count;
        }
        if self.kind == TankKind::Hell {
            new_fish.devil_marked = true;
        }
        let parent_fish_name = self.fish[idx].name.clone();
        self.fish[idx]
            .mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
            .partners
            .push(new_name.clone());
        self.used_names.insert(new_name);
        self.fish.push(new_fish);
        let new_idx = self.fish.len() - 1;
        self.fish[new_idx]
            .mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
            .partners
            .push(parent_fish_name);
    }

    pub fn apply_named_mutation(&mut self, fish_name: &str, mutation_name: &str) {
        let fish_idx = match self.fish.iter().position(|f| f.name == fish_name) {
            Some(i) => i,
            None => return,
        };
        let mut rng = rand::rng();
        if self.fish[fish_idx].species == FishSpecies::Unfish {
            let kind = self.fish[fish_idx].unfish_state.as_ref().map(|us| us.kind);
            if matches!(
                kind,
                Some(
                    UnfishKind::Ball
                        | UnfishKind::Skull
                        | UnfishKind::Reversed
                        | UnfishKind::Blinker
                        | UnfishKind::Doppleganger
                        | UnfishKind::Phantom
                )
            ) {
                self.apply_slime_mutation(fish_idx, mutation_name, &mut rng);
            } else if matches!(kind, Some(UnfishKind::Worm)) {
                if mutation_name.eq_ignore_ascii_case("mitosis") {
                    self.apply_worm_mitosis(fish_idx);
                } else {
                    self.apply_worm_mutation(fish_idx, mutation_name, &mut rng);
                }
            }
            return;
        }
        let tail_variant = match self.fish[fish_idx].species.config().body {
            BodyTemplate::Standard(bc) | BodyTemplate::Alternating(bc, _) => {
                tail_kind_to_mutant_tail(bc.tail)
            }
            BodyTemplate::Fixed { left, .. } => {
                let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                if n_chars <= 1 {
                    let allowed = [
                        "bodycolor",
                        "doublefish",
                        "mitosis",
                        "glistenenable",
                        "glistendisable",
                        "glistenfast",
                        "glistenslow",
                        "glistenmode",
                        "glistencolor",
                    ];
                    if !allowed.contains(&mutation_name.to_ascii_lowercase().as_str()) {
                        return;
                    }
                }
                MutantTail::Wide
            }
        };
        if self.fish[fish_idx].mutant.is_none() {
            let mut mutant = MutantState::new_for_standard(tail_variant, &mut rng);
            if let BodyTemplate::Fixed { left, .. } = self.fish[fish_idx].species.config().body {
                let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                if n_chars > 1 {
                    let n_base = n_chars.saturating_sub(2).max(1);
                    self.fish[fish_idx].body_size = n_base;
                    mutant.left_eyes.clear();
                    mutant.right_eyes.clear();
                }
                self.fish[fish_idx].display_width =
                    compute_display_width(self.fish[fish_idx].species, 0);
            } else {
                self.fish[fish_idx].display_width =
                    mutant.display_width(self.fish[fish_idx].body_size);
            }
            self.fish[fish_idx].mutant = Some(Box::new(mutant));
        }
        let is_double = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.is_double);
        let has_glisten = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.glistening_color.is_some());
        if mutation_name.eq_ignore_ascii_case("mitosis") {
            if is_double {
                self.apply_mitosis(fish_idx);
            }
            return;
        }
        let mutation = match mutation_name.to_ascii_lowercase().as_str() {
            "size+" => Mutation::SizeChange(1),
            "size-" => Mutation::SizeChange(-1),
            "colorpatch" => Mutation::ColorPatch,
            "eye+" => Mutation::EyeChange { delta: 1 },
            "eye-" => Mutation::EyeChange { delta: -1 },
            "eyecolor" => Mutation::EyeColor,
            "glistenfast" => Mutation::GlisteningSpeed { fast: true },
            "glistenslow" => Mutation::GlisteningSpeed { fast: false },
            "glistenmode" => Mutation::GlisteningMode,
            "glistencolor" => Mutation::GlisteningColor,
            "bodycolor" => Mutation::BodyColor,
            "bodyvariant" => Mutation::BodyVariant,
            "tailvariant" => Mutation::TailVariant,
            "mouthvariant" => Mutation::MouthVariant,
            "doublefish" => {
                if is_double {
                    return;
                }
                Mutation::Doublefish
            }
            "glistenenable" => {
                if has_glisten {
                    return;
                }
                Mutation::GlisteningEnable
            }
            "glistendisable" => {
                if !has_glisten {
                    return;
                }
                Mutation::GlisteningDisable
            }
            "alienation" => Mutation::Alienation,
            _ => return,
        };
        apply_mutation_to_fish(&mut self.fish[fish_idx], mutation, &mut rng);
    }

    fn apply_slime_mutation(
        &mut self,
        fish_idx: usize,
        mutation_name: &str,
        rng: &mut impl RngExt,
    ) {
        let mutation_name_lower = mutation_name.to_ascii_lowercase();
        let sprite_width = self.fish[fish_idx].display_width;
        let applied = {
            let unfish_state = match self.fish[fish_idx].unfish_state.as_mut() {
                Some(s) => s,
                None => return,
            };
            match mutation_name_lower.as_str() {
                "eye+" => {
                    unfish_state.add_floating_eye(rng);
                    true
                }
                "eye-" => {
                    unfish_state.remove_floating_eye(rng);
                    true
                }
                "bodycolor" => {
                    unfish_state.slime_body_color = Some(random_rgb(rng));
                    true
                }
                "eyecolor" => {
                    unfish_state.slime_eye_color = Some(random_rgb(rng));
                    true
                }
                "glistenenable" => {
                    unfish_state.slime_glisten_enabled = true;
                    true
                }
                "glistendisable" => {
                    unfish_state.slime_glisten_enabled = false;
                    true
                }
                "glistenfast" => {
                    unfish_state.slime_glisten_speed = SLIME_GLISTEN_SPEED_FAST;
                    true
                }
                "glistenslow" => {
                    unfish_state.slime_glisten_speed = SLIME_GLISTEN_SPEED_SLOW;
                    true
                }
                "glistenmode" => {
                    unfish_state.slime_glisten_mode =
                        unfish_state.slime_glisten_mode.random_other(rng);
                    true
                }
                "glistencolor" => {
                    unfish_state.slime_glisten_color = Some(random_rgb(rng));
                    true
                }
                "colorpatch" => {
                    let count =
                        rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
                    for _ in 0..count {
                        let pos = rng.random_range(0..sprite_width);
                        unfish_state
                            .slime_color_patches
                            .push((pos, random_rgb(rng)));
                    }
                    if unfish_state.slime_color_patches.len() > MUTATION_PATCH_MAX {
                        let excess = unfish_state.slime_color_patches.len() - MUTATION_PATCH_MAX;
                        unfish_state.slime_color_patches.drain(0..excess);
                    }
                    true
                }
                _ => false,
            }
        };
        if applied {
            let mutation_record = self.fish[fish_idx]
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()));
            mutation_record.count += 1;
            mutation_record.history.push(mutation_name_lower);
        }
    }

    fn apply_worm_mutation(&mut self, fish_idx: usize, mutation_name: &str, rng: &mut impl RngExt) {
        let mutation_name_lower = mutation_name.to_ascii_lowercase();
        let sprite_width = self.fish[fish_idx].display_width;
        let applied = {
            let unfish_state = match self.fish[fish_idx].unfish_state.as_mut() {
                Some(s) => s,
                None => return,
            };
            match mutation_name_lower.as_str() {
                "size+" => {
                    unfish_state.worm_segments = (unfish_state.worm_segments + 1).min(12);
                    true
                }
                "size-" => {
                    unfish_state.worm_segments =
                        unfish_state.worm_segments.saturating_sub(1).max(1);
                    true
                }
                "eye+" => {
                    unfish_state.worm_extra_eyes = (unfish_state.worm_extra_eyes + 1).min(4);
                    true
                }
                "eye-" => {
                    if unfish_state.worm_extra_eyes > 0 {
                        unfish_state.worm_extra_eyes -= 1;
                    }
                    true
                }
                "bodycolor" => {
                    unfish_state.slime_body_color = Some(random_rgb(rng));
                    true
                }
                "eyecolor" => {
                    unfish_state.slime_eye_color = Some(random_rgb(rng));
                    true
                }
                "doublefish" => {
                    if !unfish_state.worm_is_double {
                        unfish_state.worm_is_double = true;
                    }
                    true
                }
                "glistenenable" => {
                    unfish_state.slime_glisten_enabled = true;
                    true
                }
                "glistendisable" => {
                    unfish_state.slime_glisten_enabled = false;
                    true
                }
                "glistenfast" => {
                    unfish_state.slime_glisten_speed = SLIME_GLISTEN_SPEED_FAST;
                    true
                }
                "glistenslow" => {
                    unfish_state.slime_glisten_speed = SLIME_GLISTEN_SPEED_SLOW;
                    true
                }
                "glistenmode" => {
                    unfish_state.slime_glisten_mode =
                        unfish_state.slime_glisten_mode.random_other(rng);
                    true
                }
                "glistencolor" => {
                    unfish_state.slime_glisten_color = Some(random_rgb(rng));
                    true
                }
                "colorpatch" => {
                    let count =
                        rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
                    for _ in 0..count {
                        let pos = rng.random_range(0..sprite_width);
                        unfish_state
                            .slime_color_patches
                            .push((pos, random_rgb(rng)));
                    }
                    if unfish_state.slime_color_patches.len() > MUTATION_PATCH_MAX {
                        let excess = unfish_state.slime_color_patches.len() - MUTATION_PATCH_MAX;
                        unfish_state.slime_color_patches.drain(0..excess);
                    }
                    true
                }
                _ => false,
            }
        };
        if applied {
            let mutation_record = self.fish[fish_idx]
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()));
            mutation_record.count += 1;
            mutation_record.history.push(mutation_name_lower);
        }
        let (seg, extra, double) = self.fish[fish_idx]
            .unfish_state
            .as_ref()
            .map(|unfish_state| {
                (
                    unfish_state.worm_segments,
                    unfish_state.worm_extra_eyes,
                    unfish_state.worm_is_double,
                )
            })
            .unwrap();
        self.fish[fish_idx].display_width = worm_display_width(seg, extra, double);
    }

    fn apply_worm_mitosis(&mut self, idx: usize) {
        let is_double = self.fish[idx]
            .unfish_state
            .as_ref()
            .is_some_and(|unfish_state| unfish_state.worm_is_double);
        if !is_double {
            return;
        }
        let parent_name = self.fish[idx].name.clone();
        let spawn_x = self.fish[idx].position.x;
        let spawn_y = self.fish[idx].position.y;
        let (
            segments,
            extra_eyes,
            body_color,
            glisten_enabled,
            glisten_color,
            glisten_mode,
            glisten_speed,
        ) = {
            let unfish_state = self.fish[idx].unfish_state.as_ref().unwrap();
            (
                unfish_state.worm_segments,
                unfish_state.worm_extra_eyes,
                unfish_state.slime_body_color,
                unfish_state.slime_glisten_enabled,
                unfish_state.slime_glisten_color,
                unfish_state.slime_glisten_mode,
                unfish_state.slime_glisten_speed,
            )
        };
        let half = (segments / 2).max(1);
        let other_half = (segments - half).max(1);
        {
            let unfish_state = self.fish[idx].unfish_state.as_mut().unwrap();
            unfish_state.worm_segments = half;
            unfish_state.worm_is_double = false;
        }
        self.fish[idx].display_width = worm_display_width(half, extra_eyes, false);
        let mut rng = rand::rng();
        let new_name = self.unique_name(&parent_name);
        let mut new_fish = crate::fishes::fish::Fish::new_unfish(
            UnfishKind::Worm,
            new_name.clone(),
            spawn_x,
            spawn_y,
            &mut rng,
        );
        if let Some(ref mut unfish_state) = new_fish.unfish_state {
            unfish_state.worm_segments = other_half;
            unfish_state.worm_extra_eyes = extra_eyes;
            unfish_state.worm_is_double = false;
            unfish_state.slime_body_color = body_color;
            unfish_state.slime_glisten_enabled = glisten_enabled;
            unfish_state.slime_glisten_color = glisten_color;
            unfish_state.slime_glisten_mode = glisten_mode;
            unfish_state.slime_glisten_speed = glisten_speed;
        }
        new_fish.display_width = worm_display_width(other_half, extra_eyes, false);
        if self.kind == TankKind::Hell {
            new_fish.devil_marked = true;
        }
        self.used_names.insert(new_name);
        self.fish.push(new_fish);
        let new_idx = self.fish.len() - 1;
        let new_fish_name = self.fish[new_idx].name.clone();
        {
            let mutation_record = self.fish[idx]
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()));
            mutation_record.partners.push(new_fish_name);
            mutation_record.count += 1;
            mutation_record.history.push("mitosis".to_string());
        }
        let child_name = self.fish[idx].name.clone();
        self.fish[new_idx]
            .mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
            .partners
            .push(child_name);
    }

    pub fn apply_named_mutation_to_cow(&mut self, cow_name: &str, mutation_name: &str) -> bool {
        use crate::fishes::mutations::apply_mutation;
        let idx = match self
            .cows
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(cow_name))
        {
            Some(i) => i,
            None => return false,
        };
        let cow = &mut self.cows[idx];
        let is_double = cow.mutant.is_double;
        let has_glisten = cow.mutant.glistening_color.is_some();
        let mutation = match mutation_name.to_ascii_lowercase().as_str() {
            "size+" => Mutation::SizeChange(1),
            "size-" => Mutation::SizeChange(-1),
            "colorpatch" => Mutation::ColorPatch,
            "eye+" => Mutation::EyeChange { delta: 1 },
            "eye-" => Mutation::EyeChange { delta: -1 },
            "eyecolor" => Mutation::EyeColor,
            "glistenfast" => Mutation::GlisteningSpeed { fast: true },
            "glistenslow" => Mutation::GlisteningSpeed { fast: false },
            "glistenmode" => Mutation::GlisteningMode,
            "glistencolor" => Mutation::GlisteningColor,
            "bodycolor" => Mutation::BodyColor,
            "mouthvariant" => Mutation::MouthVariant,
            "doublefish" => {
                if is_double {
                    return false;
                }
                Mutation::Doublefish
            }
            "glistenenable" => {
                if has_glisten {
                    return false;
                }
                Mutation::GlisteningEnable
            }
            "glistendisable" => {
                if !has_glisten {
                    return false;
                }
                Mutation::GlisteningDisable
            }
            "alienation" => Mutation::Alienation,
            "strawberry" => Mutation::Strawberry,
            "mitosis" => {
                if !is_double {
                    return false;
                }
                return self.apply_cow_mitosis(idx);
            }
            _ => return false,
        };
        let mut rng = rand::rng();
        apply_mutation(cow, mutation, &mut rng);
        true
    }

    fn apply_cow_mitosis(&mut self, idx: usize) -> bool {
        use crate::entities::cow::Cow;
        use crate::fishes::mutations::Mutatable;
        if !self.cows[idx].mutant.is_double {
            return false;
        }
        let parent_name = self.cows[idx].name.clone();
        let parent_x = self.cows[idx].position.x;
        let parent_y = self.cows[idx].position.y;
        let parent_w = self.cows[idx].display_width;
        let variant = self.cows[idx].variant;
        let parent_color = self.cows[idx].color;
        let parent_sway_speed = self.cows[idx].sway_speed;
        let (right_eyes_count, body_variant, glistening_mode, glistening_color, eye_color) = {
            let m = &self.cows[idx].mutant;
            (
                m.double_head_eyes.len().max(2),
                m.body_variant,
                m.glistening_mode,
                m.glistening_color,
                m.eye_color,
            )
        };
        let parent_mut_count = self.cows[idx].mutations.as_ref().map_or(0, |mr| mr.count);

        {
            let p = &mut self.cows[idx];
            p.mutant.is_double = false;
            p.mutant.double_head_eyes.clear();
            p.recompute_display_width();
        }

        let mut rng = rand::rng();
        let new_name = self.unique_cow_name(&parent_name);
        let mut new_cow = Cow::new(
            new_name.clone(),
            variant,
            parent_x + parent_w as f32 + 1.0,
            parent_y,
            &mut rng,
        );
        new_cow.color = parent_color;
        new_cow.sway_speed = parent_sway_speed;
        new_cow.mutant.left_eyes.clear();
        for _ in 0..right_eyes_count {
            new_cow.mutant.left_eyes.push(EyeState::new(&mut rng));
        }
        new_cow.mutant.eye_color = eye_color;
        new_cow.mutant.glistening_mode = glistening_mode;
        new_cow.mutant.glistening_color = glistening_color;
        new_cow.mutant.body_variant = body_variant;
        new_cow.mutant.is_double = false;
        new_cow.mutant.double_head_eyes.clear();
        new_cow.recompute_display_width();

        let max_x = (self.width as i32 - new_cow.display_width as i32).max(0) as f32;
        if new_cow.position.x > max_x {
            new_cow.position.x = max_x;
        }

        {
            let record = self.cows[idx]
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()));
            record.count += 1;
            record.history.push("mitosis".to_string());
            record.partners.push(new_name.clone());
        }
        let parent_name_clone = self.cows[idx].name.clone();
        {
            let record = new_cow
                .mutations
                .get_or_insert_with(|| Box::new(MutationRecord::default()));
            record.count = parent_mut_count + 1;
            record.partners.push(parent_name_clone);
        }

        self.used_cow_names.insert(new_name);
        self.cows.push(new_cow);
        true
    }

    pub fn miracle_mutate_fish(&mut self, fish_name: &str, mutation_name: &str) -> bool {
        let fish_idx = match self
            .fish
            .iter()
            .position(|f| f.name.eq_ignore_ascii_case(fish_name))
        {
            Some(i) => i,
            None => return false,
        };
        let mut rng = rand::rng();
        if self.fish[fish_idx].species == FishSpecies::Unfish {
            let kind = self.fish[fish_idx].unfish_state.as_ref().map(|us| us.kind);
            let mutation_name_lower = mutation_name.to_ascii_lowercase();
            if matches!(
                kind,
                Some(
                    UnfishKind::Ball
                        | UnfishKind::Skull
                        | UnfishKind::Reversed
                        | UnfishKind::Blinker
                        | UnfishKind::Doppleganger
                        | UnfishKind::Phantom
                )
            ) {
                self.apply_slime_mutation(fish_idx, &mutation_name_lower, &mut rng);
            } else if matches!(kind, Some(UnfishKind::Worm)) {
                if mutation_name_lower == "mitosis" {
                    self.apply_worm_mitosis(fish_idx);
                } else {
                    self.apply_worm_mutation(fish_idx, &mutation_name_lower, &mut rng);
                }
            }
            return true;
        }
        if self.fish[fish_idx].mutant.is_none() {
            if self.fish[fish_idx].species == FishSpecies::Mutantfish {
                let body_size = self.fish[fish_idx].body_size;
                let seed = rng.random::<u64>();
                let mutant = MutantState::new(body_size, seed, &mut rng);
                self.fish[fish_idx].display_width = mutant.display_width(body_size);
                self.fish[fish_idx].mutant = Some(Box::new(mutant));
            } else {
                let tail_variant = match self.fish[fish_idx].species.config().body {
                    BodyTemplate::Standard(bc) | BodyTemplate::Alternating(bc, _) => {
                        tail_kind_to_mutant_tail(bc.tail)
                    }
                    BodyTemplate::Fixed { .. } => MutantTail::Wide,
                };
                let mut mutant = MutantState::new_for_standard(tail_variant, &mut rng);
                if let BodyTemplate::Fixed { left, .. } = self.fish[fish_idx].species.config().body
                {
                    let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                    if n_chars > 1 {
                        let n_base = n_chars.saturating_sub(2).max(1);
                        self.fish[fish_idx].body_size = n_base;
                        mutant.left_eyes.clear();
                        mutant.right_eyes.clear();
                    }
                    self.fish[fish_idx].display_width =
                        compute_display_width(self.fish[fish_idx].species, 0);
                } else {
                    self.fish[fish_idx].display_width =
                        mutant.display_width(self.fish[fish_idx].body_size);
                }
                self.fish[fish_idx].mutant = Some(Box::new(mutant));
            }
        }
        let is_double = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.is_double);
        let has_glisten = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|mutant| mutant.glistening_color.is_some());
        let mutation = if mutation_name.is_empty() {
            pick_random_miracle_mutation(&self.fish[fish_idx], is_double, has_glisten, &mut rng)
        } else {
            match mutation_name.to_ascii_lowercase().as_str() {
                "size+" => Mutation::SizeChange(1),
                "size-" => Mutation::SizeChange(-1),
                "colorpatch" => Mutation::ColorPatch,
                "eye+" => Mutation::EyeChange { delta: 1 },
                "eye-" => Mutation::EyeChange { delta: -1 },
                "eyecolor" => Mutation::EyeColor,
                "glistenfast" => Mutation::GlisteningSpeed { fast: true },
                "glistenslow" => Mutation::GlisteningSpeed { fast: false },
                "glistenmode" => Mutation::GlisteningMode,
                "glistencolor" => Mutation::GlisteningColor,
                "bodycolor" => Mutation::BodyColor,
                "bodyvariant" => Mutation::BodyVariant,
                "tailvariant" => Mutation::TailVariant,
                "mouthvariant" => Mutation::MouthVariant,
                "doublefish" if !is_double => Mutation::Doublefish,
                "mitosis" if is_double => {
                    self.apply_mitosis(fish_idx);
                    return true;
                }
                "glistenenable" if !has_glisten => Mutation::GlisteningEnable,
                "glistendisable" if has_glisten => Mutation::GlisteningDisable,
                "alienation" => Mutation::Alienation,
                _ => pick_random_miracle_mutation(
                    &self.fish[fish_idx],
                    is_double,
                    has_glisten,
                    &mut rng,
                ),
            }
        };
        apply_mutation_to_fish(&mut self.fish[fish_idx], mutation, &mut rng);
        true
    }
}
