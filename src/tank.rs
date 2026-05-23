use std::collections::HashSet;

use rand::RngExt;

use crate::entities::bubble::Bubble;
use crate::entities::coral::{CoralStructure, FloorAlgae, extend_coral_reef};
use crate::entities::fish::{Direction, EATING_DURATION, Fish, FishState, compute_display_width};
use crate::entities::food::Food;
use crate::entities::hell::{HELL_BUBBLE_COLOR, HellBackground, HellPlant, extend_hell_plants};
use crate::entities::mutant::{
    EXTRA_BODY_FOR_DOUBLE, EyeState, MIN_BODY_CHARS, MutantState, MutantTail, random_rgb,
};
use crate::entities::plant::Plant;
use crate::entities::species::{BodyTemplate, FishSpecies, TailKind};
use crate::entities::unfish::{
    PHANTOM_CROSS_TANK_CHANCE, PHANTOM_TELEPORT_MEAN,
    SLIME_GLISTEN_SPEED_FAST, SLIME_GLISTEN_SPEED_SLOW, SPAWNABLE_UNFISH, UnfishKind,
    VOID_SPAWN_MEAN_SECS, worm_display_width,
};
use crate::entities::void::VoidBackground;
use crate::loot::ConsumableKind;
use crate::names;
use crate::settings::Settings;
use crate::util::{hyperbolic_scale, sample_exponential};

pub enum TankEvent {
    PhantomCrossTank { fish_name: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TankKind {
    Base,
    CoralReef,
    Hell,
    Void,
}

impl TankKind {
    pub fn display_name(self) -> &'static str {
        match self {
            TankKind::Base => "Base",
            TankKind::CoralReef => "Coral Reef",
            TankKind::Hell => "Helltank",
            TankKind::Void => "Voidtank",
        }
    }
}

pub struct ActiveConsumable {
    pub kind: ConsumableKind,
    pub stacks: u32,
    pub time_remaining: f32,
}

pub const TANK_CAPACITY: usize = 50;
pub const HELL_TANK_CAPACITY: usize = 100;
pub const VOID_TANK_CAPACITY: usize = 100;

const PLANT_SPACING_MIN: i32 = 3;
const PLANT_SPACING_MAX: i32 = 6;
const PLANT_HEIGHT_MIN: usize = 8;
const PLANT_HEIGHT_MAX: usize = 27;
const SEEK_BOOST_GROWTH: f32 = 0.8;
const FOOD_SPAWN_SPREAD: f32 = 15.0;
const INITIAL_WIDTH: u16 = 80;
const INITIAL_HEIGHT: u16 = 24;
const SEEK_DX_DEADZONE: f32 = 0.5;
const SEEK_NORM_MIN: f32 = 0.01;
const SEEK_DY_MULTIPLIER: f32 = 1.2;
const SEEK_BOOST_INITIAL_MAX: f32 = 0.2;
const CORAL_SPAWN_LOOKAHEAD: i32 = 130;
const PLANT_SPAWN_LOOKAHEAD: i32 = 30;
const BUBBLE_BOTTOM_SPAWN_RATE_MIN: f32 = 0.3;
const BUBBLE_BOTTOM_SPAWN_RATE_MAX: f32 = 1.5;
const BUBBLE_SURFACE_SPAWN_RATE_MIN: f32 = 1.0;
const BUBBLE_SURFACE_SPAWN_RATE_MAX: f32 = 3.0;
const BUBBLE_ZOOMIE_CHANCE: f32 = 0.45;
const MUTATION_INTERVAL_BASE: f32 = 30.0 * 60.0;
const MUTATION_ALPHA: f32 = 1.0 / 3.0;
const MUTATION_MEAN_FLOOR_SECS: f32 = 3.0;
const MUTATION_MAX_BODY_SIZE: usize = 12;
const MUTATION_PATCH_MAX: usize = 12;
const MUTATION_PATCH_COUNT_MIN: usize = 2;
const MUTATION_PATCH_COUNT_MAX: usize = 4;
const GLISTEN_SPEED_FAST_MULT: f32 = 3.5;
const GLISTEN_SPEED_SLOW_MULT: f32 = 0.25;
const SWAY_SPEED_CLAMP_MAX: f32 = 2.5;
const DOUBLE_EYE_COUNT_MAX: usize = 3;
const MIN_SPLIT_BODY_SIZE: usize = 2;
const FISH_SPAWN_X_MIN: f32 = 5.0;
const FISH_SPAWN_X_MAX_OFFSET: f32 = 15.0;
const FISH_SPAWN_X_SAFE_MIN: f32 = 6.0;
const FISH_SPAWN_Y_MIN: f32 = 2.0;
const FISH_SPAWN_Y_MAX_OFFSET: f32 = 5.0;
const FISH_SPAWN_Y_SAFE_MIN: f32 = 3.0;
const FOOD_WEIGHT_GAIN_G: u32 = 50;

enum Mutation {
    SizeChange(i32),
    ColorPatch,
    EyeChange { delta: i32 },
    EyeColor,
    GlisteningSpeed { fast: bool },
    GlisteningMode,
    GlisteningColor,
    BodyColor,
    BodyVariant,
    TailVariant,
    MouthVariant,
    Doublefish,
    Mitosis,
    GlisteningEnable,
    GlisteningDisable,
}

impl Mutation {
    fn display_name(&self) -> &'static str {
        match self {
            Mutation::SizeChange(d) => {
                if *d > 0 {
                    "size+"
                } else {
                    "size-"
                }
            }
            Mutation::ColorPatch => "colorpatch",
            Mutation::EyeChange { delta } => {
                if *delta > 0 {
                    "eye+"
                } else {
                    "eye-"
                }
            }
            Mutation::EyeColor => "eyecolor",
            Mutation::GlisteningSpeed { fast } => {
                if *fast {
                    "glistenfast"
                } else {
                    "glistenslow"
                }
            }
            Mutation::GlisteningMode => "glistenmode",
            Mutation::GlisteningColor => "glistencolor",
            Mutation::BodyColor => "bodycolor",
            Mutation::BodyVariant => "bodyvariant",
            Mutation::TailVariant => "tailvariant",
            Mutation::MouthVariant => "mouthvariant",
            Mutation::Doublefish => "doublefish",
            Mutation::Mitosis => "mitosis",
            Mutation::GlisteningEnable => "glistenenable",
            Mutation::GlisteningDisable => "glistendisable",
        }
    }
}

pub struct Tank {
    pub name: String,
    pub kind: TankKind,
    pub fish: Vec<Fish>,
    pub food: Vec<Food>,
    pub plants: Vec<Plant>,
    pub corals: Vec<CoralStructure>,
    pub floor_algae: Vec<FloorAlgae>,
    pub bubbles: Vec<Bubble>,
    pub hell_bg: Option<HellBackground>,
    pub hell_plants: Vec<HellPlant>,
    pub void_bg: Option<VoidBackground>,
    pub width: u16,
    pub height: u16,
    pub used_names: HashSet<String>,
    pub pending_star_money: u32,
    pub extra_capacity: u32,
    bubble_bottom_timer: f32,
    bubble_surface_timer: f32,
    mutation_timer: f32,
    void_spawn_timer: f32,
}

impl Tank {
    pub fn new(name: String, kind: TankKind) -> Self {
        let mut rng = rand::rng();
        let mut plants = Vec::new();
        let mut corals = Vec::new();
        let mut floor_algae = Vec::new();
        let mut hell_bg = None;
        let mut hell_plants = Vec::new();
        let mut void_bg = None;
        match kind {
            TankKind::Base => extend_plants(
                &mut plants,
                INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                &mut rng,
            ),
            TankKind::CoralReef => extend_coral_reef(
                &mut corals,
                &mut floor_algae,
                INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                &mut rng,
            ),
            TankKind::Hell => {
                hell_bg = Some(HellBackground::new(&mut rng));
                extend_hell_plants(
                    &mut hell_plants,
                    INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                    &mut rng,
                );
            }
            TankKind::Void => {
                void_bg = Some(VoidBackground::new());
            }
        }
        Self {
            name,
            kind,
            fish: Vec::new(),
            food: Vec::new(),
            plants,
            corals,
            floor_algae,
            bubbles: Vec::new(),
            hell_bg,
            hell_plants,
            void_bg,
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
            used_names: HashSet::new(),
            pending_star_money: 0,
            extra_capacity: 0,
            bubble_bottom_timer: rng
                .random_range(BUBBLE_BOTTOM_SPAWN_RATE_MIN..BUBBLE_BOTTOM_SPAWN_RATE_MAX),
            bubble_surface_timer: rng
                .random_range(BUBBLE_SURFACE_SPAWN_RATE_MIN..BUBBLE_SURFACE_SPAWN_RATE_MAX),
            mutation_timer: MUTATION_INTERVAL_BASE,
            void_spawn_timer: sample_exponential(&mut rng, VOID_SPAWN_MEAN_SECS),
        }
    }

    pub fn capacity(&self) -> usize {
        let base = match self.kind {
            TankKind::Hell => HELL_TANK_CAPACITY,
            TankKind::Void => VOID_TANK_CAPACITY,
            _ => TANK_CAPACITY,
        };
        base + self.extra_capacity as usize
    }

    pub fn expand(&mut self, amount: u32) {
        self.extra_capacity += amount;
    }

    pub fn is_full(&self) -> bool {
        self.fish.len() >= self.capacity()
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let old_width = self.width;
        self.width = width;
        self.height = height;
        let max_y = height as f32 - 1.0;
        for fish in &mut self.fish {
            if fish.position.y > max_y {
                fish.position.y = max_y;
                if fish.velocity.dy > 0.0 {
                    fish.velocity.dy = -fish.velocity.dy;
                }
            }
        }
        for food in &mut self.food {
            if food.position.y > max_y {
                food.position.y = max_y;
            }
        }
        if width > old_width {
            let mut rng = rand::rng();
            self.extend_environment(&mut rng);
        }
    }

    fn extend_environment(&mut self, rng: &mut impl RngExt) {
        match self.kind {
            TankKind::Base => extend_plants(
                &mut self.plants,
                self.width as i32 + PLANT_SPAWN_LOOKAHEAD,
                rng,
            ),
            TankKind::CoralReef => extend_coral_reef(
                &mut self.corals,
                &mut self.floor_algae,
                self.width as i32 + CORAL_SPAWN_LOOKAHEAD,
                rng,
            ),
            TankKind::Hell => extend_hell_plants(
                &mut self.hell_plants,
                self.width as i32 + PLANT_SPAWN_LOOKAHEAD,
                rng,
            ),
            TankKind::Void => {}
        }
    }

    pub fn spawn_fish(
        &mut self,
        species: FishSpecies,
        name: String,
        rng: &mut impl RngExt,
    ) -> bool {
        if self.is_full() {
            return false;
        }
        let actual_name = self.unique_name(&name);
        let x_max = (self.width as f32 - FISH_SPAWN_X_MAX_OFFSET).max(FISH_SPAWN_X_SAFE_MIN);
        let y_max = (self.height as f32 - FISH_SPAWN_Y_MAX_OFFSET).max(FISH_SPAWN_Y_SAFE_MIN);
        let x = rng.random_range(FISH_SPAWN_X_MIN..x_max);
        let y = rng.random_range(FISH_SPAWN_Y_MIN..y_max);
        let mut fish = Fish::new(species, actual_name.clone(), x, y, rng);
        if self.kind == TankKind::Hell {
            fish.devil_marked = true;
        }
        self.used_names.insert(actual_name);
        self.fish.push(fish);
        true
    }

    pub fn place_fish(&mut self, mut fish: Fish, name: String, rng: &mut impl RngExt) {
        let actual_name = self.unique_name(&name);
        let x_max = (self.width as f32 - FISH_SPAWN_X_MAX_OFFSET).max(FISH_SPAWN_X_SAFE_MIN);
        let y_max = (self.height as f32 - FISH_SPAWN_Y_MAX_OFFSET).max(FISH_SPAWN_Y_SAFE_MIN);
        fish.position.x = rng.random_range(FISH_SPAWN_X_MIN..x_max);
        fish.position.y = rng.random_range(FISH_SPAWN_Y_MIN..y_max);
        fish.name = actual_name.clone();
        if self.kind == TankKind::Hell {
            fish.devil_marked = true;
        }
        self.used_names.insert(actual_name);
        self.fish.push(fish);
    }

    pub fn feed(&mut self, count: usize, food_supply: &mut u32) {
        if self.width == 0 {
            return;
        }
        let actual = count.min(*food_supply as usize);
        if actual == 0 {
            return;
        }
        *food_supply -= actual as u32;
        let mut rng = rand::rng();
        let spread = FOOD_SPAWN_SPREAD;
        let min_center = spread;
        let max_center = (self.width as f32 - 1.0 - spread).max(spread + 1.0);
        let center_x = rng.random_range(min_center..max_center);
        for _ in 0..actual {
            let offset = rng.random_range(-spread..spread);
            let x = (center_x + offset).clamp(0.0, self.width as f32 - 1.0);
            self.food.push(Food::new(x));
        }
    }

    pub fn tick(&mut self, settings: &Settings, coffee: u32) -> Vec<TankEvent> {
        let dt = 1.0 / settings.fps;
        let mut rng = rand::rng();

        for plant in &mut self.plants {
            plant.tick();
        }
        if let Some(bg) = &mut self.hell_bg {
            bg.tick(dt, &mut rng, self.width, self.height);
        }
        if let Some(bg) = &mut self.void_bg {
            bg.tick();
        }
        for plant in &mut self.hell_plants {
            plant.tick(dt, &mut rng);
        }
        for coral in &mut self.corals {
            coral.tick(dt, &mut rng);
        }
        for fa in &mut self.floor_algae {
            fa.tick(dt);
        }
        for food in &mut self.food {
            food.tick(settings, self.width, self.height);
        }
        let star: u32 = self
            .bubbles
            .iter_mut()
            .map(|b| b.tick(settings, self.width))
            .sum();
        self.pending_star_money += star;
        self.bubbles.retain(|b| !b.dead);

        self.steer_seeking_fish();
        self.tick_fish(settings, coffee);
        self.spawn_bubbles(dt);
        self.check_eating_collisions();

        let prev_len = self.food.len();
        self.food.retain(|f| !f.eaten);
        if self.food.len() < prev_len {
            for fish in &mut self.fish {
                if matches!(fish.state, FishState::SeekingFood(_, _)) {
                    fish.cancel_seek();
                }
            }
        }

        self.assign_food_to_idle_fish();
        self.tick_mutations(dt);
        self.tick_dopplegangers();
        if self.kind == TankKind::Void {
            self.tick_void_spawn(dt, &mut rng);
        }
        self.tick_phantoms(dt, &mut rng)
    }

    fn tick_mutations(&mut self, dt: f32) {
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
                if f.species == FishSpecies::Mutantfish {
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
        let is_double = self.fish[idx].mutant.as_ref().is_some_and(|m| m.is_double);
        if !is_double {
            return;
        }

        let orig_body_size = self.fish[idx].body_size;
        let orig_color = self.fish[idx].color;
        let orig_sway_speed = self.fish[idx].sway_speed;
        let orig_pos_x = self.fish[idx].position.x;
        let orig_pos_y = self.fish[idx].position.y;
        let orig_name = self.fish[idx].name.clone();

        let (
            max_eyes_orig,
            double_eye_count,
            half,
            other_half,
            new_patches,
            body_variant,
            glistening_mode,
            glistening_color,
            eye_color,
            orig_mutation_count,
        ) = {
            let m = self.fish[idx].mutant.as_ref().unwrap();
            let max_eyes = m.left_eyes.len().max(m.right_eyes.len());
            let double_eye_count = m.double_head_eyes.len().max(1);

            let total_body = orig_body_size + EXTRA_BODY_FOR_DOUBLE;
            let raw_half = total_body / 2;
            let min_half = (max_eyes + MIN_BODY_CHARS).max(MIN_SPLIT_BODY_SIZE);
            let min_other = (double_eye_count + MIN_BODY_CHARS).max(MIN_SPLIT_BODY_SIZE);
            let half = raw_half.max(min_half);
            let other_half = (total_body - raw_half).max(min_other);

            let split_pos = 1 + max_eyes + half;
            let body_start_new = 1 + double_eye_count;
            let new_patches: Vec<_> = m
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
                m.body_variant,
                m.glistening_mode,
                m.glistening_color,
                m.eye_color,
                m.mutation_count,
            )
        };

        let mut rng = rand::rng();
        let tail_for_new = match rng.random_range(0u32..3) {
            0 => MutantTail::Wide,
            1 => MutantTail::Swaying,
            _ => MutantTail::Curly,
        };

        {
            let fish = &mut self.fish[idx];
            fish.body_size = half;
            let m = fish.mutant.as_mut().unwrap();
            m.is_double = false;
            m.double_head_eyes.clear();
            let split_pos_orig = 1 + max_eyes_orig + half;
            m.color_patches.retain(|&(pos, _)| pos < split_pos_orig);
            fish.display_width = m.display_width(half);
        }

        let new_name = self.unique_name(&orig_name);
        let mut new_fish = Fish::new(
            FishSpecies::Mutantfish,
            new_name.clone(),
            orig_pos_x,
            orig_pos_y,
            &mut rng,
        );
        new_fish.color = orig_color;
        new_fish.sway_speed = orig_sway_speed;
        new_fish.body_size = other_half;
        {
            let m = new_fish.mutant.as_mut().unwrap();
            m.body_variant = body_variant;
            m.glistening_mode = glistening_mode;
            m.glistening_color = glistening_color;
            m.eye_color = eye_color;
            m.tail_variant = tail_for_new;
            m.color_patches = new_patches;
            m.is_double = false;
            m.double_head_eyes.clear();
            m.left_eyes.clear();
            m.right_eyes.clear();
            for _ in 0..double_eye_count {
                m.left_eyes.push(EyeState::new(&mut rng));
                m.right_eyes.push(EyeState::new(&mut rng));
            }
            m.mutation_count = orig_mutation_count;
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
            .mutant
            .as_mut()
            .unwrap()
            .mitosis_partners
            .push(new_fish_name);
        let orig_name = self.fish[idx].name.clone();
        self.fish[new_idx]
            .mutant
            .as_mut()
            .unwrap()
            .mitosis_partners
            .push(orig_name);
    }

    fn apply_fixed_body_mitosis(&mut self, idx: usize) {
        let is_double = self.fish[idx].mutant.as_ref().is_some_and(|m| m.is_double);
        if !is_double {
            return;
        }
        let species = self.fish[idx].species;
        let orig_name = self.fish[idx].name.clone();
        let orig_pos_x = self.fish[idx].position.x;
        let orig_pos_y = self.fish[idx].position.y;
        let orig_color = self.fish[idx].color;
        let orig_sway_speed = self.fish[idx].sway_speed;
        let (glistening_mode, glistening_color, eye_color, mut_count) = {
            let m = self.fish[idx].mutant.as_ref().unwrap();
            (
                m.glistening_mode,
                m.glistening_color,
                m.eye_color,
                m.mutation_count,
            )
        };
        {
            let m = self.fish[idx].mutant.as_mut().unwrap();
            m.is_double = false;
            m.double_head_eyes.clear();
        }
        self.fish[idx].display_width = compute_display_width(species, self.fish[idx].body_size);
        let mut rng = rand::rng();
        let new_name = self.unique_name(&orig_name);
        let mut new_fish = crate::entities::fish::Fish::new(
            species,
            new_name.clone(),
            orig_pos_x,
            orig_pos_y,
            &mut rng,
        );
        new_fish.color = orig_color;
        new_fish.sway_speed = orig_sway_speed;
        new_fish.display_width = compute_display_width(species, new_fish.body_size);
        let mut new_m = MutantState::new_for_standard(MutantTail::Wide, &mut rng);
        new_m.glistening_mode = glistening_mode;
        new_m.glistening_color = glistening_color;
        new_m.eye_color = eye_color;
        new_m.mutation_count = mut_count;
        new_fish.mutant = Some(Box::new(new_m));
        if self.kind == TankKind::Hell {
            new_fish.devil_marked = true;
        }
        let orig_fish_name = self.fish[idx].name.clone();
        self.fish[idx]
            .mutant
            .as_mut()
            .unwrap()
            .mitosis_partners
            .push(new_name.clone());
        self.used_names.insert(new_name);
        self.fish.push(new_fish);
        let new_idx = self.fish.len() - 1;
        self.fish[new_idx]
            .mutant
            .as_mut()
            .unwrap()
            .mitosis_partners
            .push(orig_fish_name);
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
            BodyTemplate::Standard(bc) => tail_kind_to_mutant_tail(bc.tail),
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
            let mut m = MutantState::new_for_standard(tail_variant, &mut rng);
            if let BodyTemplate::Fixed { left, .. } = self.fish[fish_idx].species.config().body {
                let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                if n_chars > 1 {
                    let n_base = n_chars.saturating_sub(2).max(1);
                    self.fish[fish_idx].body_size = n_base;
                    m.left_eyes.clear();
                    m.right_eyes.clear();
                }
                self.fish[fish_idx].display_width =
                    compute_display_width(self.fish[fish_idx].species, 0);
            } else {
                self.fish[fish_idx].display_width = m.display_width(self.fish[fish_idx].body_size);
            }
            self.fish[fish_idx].mutant = Some(Box::new(m));
        }
        let is_double = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|m| m.is_double);
        let has_glisten = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|m| m.glistening_color.is_some());
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
        let name_lower = mutation_name.to_ascii_lowercase();
        let sprite_w = self.fish[fish_idx].display_width;
        let applied = {
            let us = match self.fish[fish_idx].unfish_state.as_mut() {
                Some(s) => s,
                None => return,
            };
            match name_lower.as_str() {
                "eye+" => {
                    us.add_floating_eye(rng);
                    true
                }
                "eye-" => {
                    us.remove_floating_eye(rng);
                    true
                }
                "bodycolor" => {
                    us.slime_body_color = Some(random_rgb(rng));
                    true
                }
                "eyecolor" => {
                    us.slime_eye_color = Some(random_rgb(rng));
                    true
                }
                "glistenenable" => {
                    us.slime_glisten_enabled = true;
                    true
                }
                "glistendisable" => {
                    us.slime_glisten_enabled = false;
                    true
                }
                "glistenfast" => {
                    us.slime_glisten_speed = SLIME_GLISTEN_SPEED_FAST;
                    true
                }
                "glistenslow" => {
                    us.slime_glisten_speed = SLIME_GLISTEN_SPEED_SLOW;
                    true
                }
                "glistenmode" => {
                    us.slime_glisten_mode = us.slime_glisten_mode.random_other(rng);
                    true
                }
                "glistencolor" => {
                    us.slime_glisten_color = Some(random_rgb(rng));
                    true
                }
                "colorpatch" => {
                    let count =
                        rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
                    for _ in 0..count {
                        let pos = rng.random_range(0..sprite_w);
                        us.slime_color_patches.push((pos, random_rgb(rng)));
                    }
                    if us.slime_color_patches.len() > MUTATION_PATCH_MAX {
                        let excess = us.slime_color_patches.len() - MUTATION_PATCH_MAX;
                        us.slime_color_patches.drain(0..excess);
                    }
                    true
                }
                _ => false,
            }
        };
        if applied && let Some(ref mut us) = self.fish[fish_idx].unfish_state {
            us.mutation_count += 1;
            us.mutation_history.push(name_lower);
        }
    }

    fn apply_worm_mutation(&mut self, fish_idx: usize, mutation_name: &str, rng: &mut impl RngExt) {
        let name_lower = mutation_name.to_ascii_lowercase();
        let sprite_w = self.fish[fish_idx].display_width;
        let applied = {
            let us = match self.fish[fish_idx].unfish_state.as_mut() {
                Some(s) => s,
                None => return,
            };
            match name_lower.as_str() {
                "size+" => {
                    us.worm_segments = (us.worm_segments + 1).min(12);
                    true
                }
                "size-" => {
                    us.worm_segments = us.worm_segments.saturating_sub(1).max(1);
                    true
                }
                "eye+" => {
                    us.worm_extra_eyes = (us.worm_extra_eyes + 1).min(4);
                    true
                }
                "eye-" => {
                    if us.worm_extra_eyes > 0 {
                        us.worm_extra_eyes -= 1;
                    }
                    true
                }
                "bodycolor" => {
                    us.slime_body_color = Some(random_rgb(rng));
                    true
                }
                "eyecolor" => {
                    us.slime_eye_color = Some(random_rgb(rng));
                    true
                }
                "doublefish" => {
                    if !us.worm_is_double {
                        us.worm_is_double = true;
                    }
                    true
                }
                "glistenenable" => {
                    us.slime_glisten_enabled = true;
                    true
                }
                "glistendisable" => {
                    us.slime_glisten_enabled = false;
                    true
                }
                "glistenfast" => {
                    us.slime_glisten_speed = SLIME_GLISTEN_SPEED_FAST;
                    true
                }
                "glistenslow" => {
                    us.slime_glisten_speed = SLIME_GLISTEN_SPEED_SLOW;
                    true
                }
                "glistenmode" => {
                    us.slime_glisten_mode = us.slime_glisten_mode.random_other(rng);
                    true
                }
                "glistencolor" => {
                    us.slime_glisten_color = Some(random_rgb(rng));
                    true
                }
                "colorpatch" => {
                    let count =
                        rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
                    for _ in 0..count {
                        let pos = rng.random_range(0..sprite_w);
                        us.slime_color_patches.push((pos, random_rgb(rng)));
                    }
                    if us.slime_color_patches.len() > MUTATION_PATCH_MAX {
                        let excess = us.slime_color_patches.len() - MUTATION_PATCH_MAX;
                        us.slime_color_patches.drain(0..excess);
                    }
                    true
                }
                _ => false,
            }
        };
        if applied && let Some(ref mut us) = self.fish[fish_idx].unfish_state {
            us.mutation_count += 1;
            us.mutation_history.push(name_lower);
        }
        let (seg, extra, double) = self.fish[fish_idx]
            .unfish_state
            .as_ref()
            .map(|us| (us.worm_segments, us.worm_extra_eyes, us.worm_is_double))
            .unwrap();
        self.fish[fish_idx].display_width = worm_display_width(seg, extra, double);
    }

    fn apply_worm_mitosis(&mut self, idx: usize) {
        let is_double = self.fish[idx]
            .unfish_state
            .as_ref()
            .is_some_and(|us| us.worm_is_double);
        if !is_double {
            return;
        }
        let orig_name = self.fish[idx].name.clone();
        let orig_pos_x = self.fish[idx].position.x;
        let orig_pos_y = self.fish[idx].position.y;
        let (
            segments,
            extra_eyes,
            body_color,
            glisten_enabled,
            glisten_color,
            glisten_mode,
            glisten_speed,
        ) = {
            let us = self.fish[idx].unfish_state.as_ref().unwrap();
            (
                us.worm_segments,
                us.worm_extra_eyes,
                us.slime_body_color,
                us.slime_glisten_enabled,
                us.slime_glisten_color,
                us.slime_glisten_mode,
                us.slime_glisten_speed,
            )
        };
        let half = (segments / 2).max(1);
        let other_half = (segments - half).max(1);
        {
            let us = self.fish[idx].unfish_state.as_mut().unwrap();
            us.worm_segments = half;
            us.worm_is_double = false;
        }
        self.fish[idx].display_width = worm_display_width(half, extra_eyes, false);
        let mut rng = rand::rng();
        let new_name = self.unique_name(&orig_name);
        let mut new_fish = crate::entities::fish::Fish::new_unfish(
            UnfishKind::Worm,
            new_name.clone(),
            orig_pos_x,
            orig_pos_y,
            &mut rng,
        );
        if let Some(ref mut us) = new_fish.unfish_state {
            us.worm_segments = other_half;
            us.worm_extra_eyes = extra_eyes;
            us.worm_is_double = false;
            us.slime_body_color = body_color;
            us.slime_glisten_enabled = glisten_enabled;
            us.slime_glisten_color = glisten_color;
            us.slime_glisten_mode = glisten_mode;
            us.slime_glisten_speed = glisten_speed;
        }
        new_fish.display_width = worm_display_width(other_half, extra_eyes, false);
        if self.kind == TankKind::Hell {
            new_fish.devil_marked = true;
        }
        self.used_names.insert(new_name);
        self.fish.push(new_fish);
        let new_idx = self.fish.len() - 1;
        let new_fish_name = self.fish[new_idx].name.clone();
        if let Some(ref mut us) = self.fish[idx].unfish_state {
            us.mitosis_partners.push(new_fish_name);
            us.mutation_count += 1;
            us.mutation_history.push("mitosis".to_string());
        }
        let orig_name_for_partner = self.fish[idx].name.clone();
        if let Some(ref mut us) = self.fish[new_idx].unfish_state {
            us.mitosis_partners.push(orig_name_for_partner);
        }
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
            let name_lower = mutation_name.to_ascii_lowercase();
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
                self.apply_slime_mutation(fish_idx, &name_lower, &mut rng);
            } else if matches!(kind, Some(UnfishKind::Worm)) {
                if name_lower == "mitosis" {
                    self.apply_worm_mitosis(fish_idx);
                } else {
                    self.apply_worm_mutation(fish_idx, &name_lower, &mut rng);
                }
            }
            return true;
        }
        if self.fish[fish_idx].mutant.is_none() {
            if self.fish[fish_idx].species == FishSpecies::Mutantfish {
                let body_size = self.fish[fish_idx].body_size;
                let seed = rng.random::<u64>();
                let m = MutantState::new(body_size, seed, &mut rng);
                self.fish[fish_idx].display_width = m.display_width(body_size);
                self.fish[fish_idx].mutant = Some(Box::new(m));
            } else {
                let tail_variant = match self.fish[fish_idx].species.config().body {
                    BodyTemplate::Standard(bc) => tail_kind_to_mutant_tail(bc.tail),
                    BodyTemplate::Fixed { .. } => MutantTail::Wide,
                };
                let mut m = MutantState::new_for_standard(tail_variant, &mut rng);
                if let BodyTemplate::Fixed { left, .. } = self.fish[fish_idx].species.config().body
                {
                    let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
                    if n_chars > 1 {
                        let n_base = n_chars.saturating_sub(2).max(1);
                        self.fish[fish_idx].body_size = n_base;
                        m.left_eyes.clear();
                        m.right_eyes.clear();
                    }
                    self.fish[fish_idx].display_width =
                        compute_display_width(self.fish[fish_idx].species, 0);
                } else {
                    self.fish[fish_idx].display_width =
                        m.display_width(self.fish[fish_idx].body_size);
                }
                self.fish[fish_idx].mutant = Some(Box::new(m));
            }
        }
        let is_double = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|m| m.is_double);
        let has_glisten = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|m| m.glistening_color.is_some());
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

    fn unique_name(&self, requested: &str) -> String {
        names::unique_name_in(&self.used_names, requested)
    }

    fn spawn_bubbles(&mut self, dt: f32) {
        if self.kind == TankKind::Void {
            return;
        }
        if self.width == 0 || self.height == 0 {
            return;
        }
        let mut rng = rand::rng();

        self.bubble_bottom_timer -= dt;
        if self.bubble_bottom_timer <= 0.0 {
            let x = rng.random_range(0.0..self.width as f32);
            let y = (self.height as f32 - 1.0).max(0.0);
            let bubble = if self.kind == TankKind::Hell {
                Bubble::new_rising_colored(x, y, HELL_BUBBLE_COLOR)
            } else {
                Bubble::new_rising(x, y)
            };
            self.bubbles.push(bubble);
            self.bubble_bottom_timer =
                rng.random_range(BUBBLE_BOTTOM_SPAWN_RATE_MIN..BUBBLE_BOTTOM_SPAWN_RATE_MAX);
        }

        self.bubble_surface_timer -= dt;
        if self.bubble_surface_timer <= 0.0 {
            let x = rng.random_range(0.0..self.width as f32);
            let max_y = (self.height as f32 * 0.25).max(1.0);
            let y = rng.random_range(0.0..max_y);
            let bubble = if self.kind == TankKind::Hell {
                Bubble::new_surface_colored(x, y, HELL_BUBBLE_COLOR)
            } else {
                Bubble::new_surface(x, y)
            };
            self.bubbles.push(bubble);
            self.bubble_surface_timer =
                rng.random_range(BUBBLE_SURFACE_SPAWN_RATE_MIN..BUBBLE_SURFACE_SPAWN_RATE_MAX);
        }

        for fish in &self.fish {
            if !matches!(fish.state, FishState::Zoomie { .. }) {
                continue;
            }
            if rng.random::<f32>() < BUBBLE_ZOOMIE_CHANCE {
                let tail_x = match fish.facing {
                    Direction::Left => fish.position.x + fish.len() as f32 - 1.0,
                    Direction::Right => fish.position.x,
                };
                let bubble = match fish.species {
                    FishSpecies::Goldenfish => {
                        let mut b = Bubble::new_rising_custom(
                            tail_x,
                            fish.position.y,
                            '☆',
                            ratatui::style::Color::Rgb(255, 255, 80),
                        );
                        b.money_value = Some(5);
                        b
                    }
                    FishSpecies::Mutantfish => {
                        Bubble::new_rising_custom(tail_x, fish.position.y, '†', fish.color)
                    }
                    _ => {
                        if self.kind == TankKind::Hell {
                            Bubble::new_rising_colored(tail_x, fish.position.y, HELL_BUBBLE_COLOR)
                        } else {
                            Bubble::new_rising(tail_x, fish.position.y)
                        }
                    }
                };
                self.bubbles.push(bubble);
            }
        }
    }

    fn steer_seeking_fish(&mut self) {
        for i in 0..self.fish.len() {
            let (idx, approach_right) = match self.fish[i].state {
                FishState::SeekingFood(idx, ar) => (idx, ar),
                _ => continue,
            };
            if idx >= self.food.len() {
                self.fish[i].cancel_seek();
                continue;
            }

            self.fish[i].seek_boost += SEEK_BOOST_GROWTH / (1.0 + self.fish[i].seek_boost);

            let food_x = self.food[idx].position.x;
            let food_y = self.food[idx].position.y;
            let fish_len = self.fish[i].len() as f32;
            let fish_y = self.fish[i].position.y;

            let target_x = if approach_right {
                food_x - (fish_len - 1.0)
            } else {
                food_x
            };

            let raw_dx = target_x - self.fish[i].position.x;
            let dx = if raw_dx.abs() < SEEK_DX_DEADZONE {
                0.0
            } else {
                raw_dx
            };
            let raw_dy = food_y - fish_y;
            let dy = if self.food[idx].settled && fish_y as i32 >= food_y as i32 {
                0.0
            } else {
                raw_dy
            };
            let norm = (dx * dx + dy * dy).sqrt().max(SEEK_NORM_MIN);
            let seek_speed = self.fish[i].speed + self.fish[i].seek_boost;
            self.fish[i].velocity.dx = (dx / norm) * seek_speed;
            self.fish[i].velocity.dy = (dy / norm) * seek_speed * SEEK_DY_MULTIPLIER;
            self.fish[i].facing = if approach_right {
                Direction::Right
            } else {
                Direction::Left
            };
        }
    }

    fn tick_fish(&mut self, settings: &Settings, coffee: u32) {
        for fish in &mut self.fish {
            fish.tick(settings, self.width, self.height, coffee);
        }
    }

    fn check_eating_collisions(&mut self) {
        for i in 0..self.fish.len() {
            let idx = match self.fish[i].state {
                FishState::SeekingFood(idx, _) => idx,
                _ => continue,
            };
            if idx >= self.food.len() || self.food[idx].eaten {
                continue;
            }
            let head_x = self.fish[i].head_x();
            let head_y = self.fish[i].position.y as i32;
            let fx = self.food[idx].position.x as i32;
            let fy = self.food[idx].position.y as i32;
            if (head_x - fx).abs() <= 1 && head_y == fy {
                self.food[idx].eaten = true;
                self.fish[i].state = FishState::Eating {
                    time_remaining: EATING_DURATION,
                };
                let cat = self.fish[i].size_category;
                let cap = if self.fish[i].is_weight_uncapped() {
                    0
                } else {
                    self.fish[i].species.config().weight_cap[cat as usize]
                };
                let new_w = self.fish[i].weight_g + FOOD_WEIGHT_GAIN_G;
                self.fish[i].weight_g = if cap == 0 { new_w } else { new_w.min(cap) };
            }
        }
    }

    fn assign_food_to_idle_fish(&mut self) {
        if self.food.is_empty() {
            return;
        }
        for i in 0..self.fish.len() {
            if !matches!(
                self.fish[i].state,
                FishState::Idle | FishState::Zoomie { .. }
            ) {
                continue;
            }
            let fish_len = self.fish[i].len() as f32;
            let head_x = self.fish[i].head_x() as f32;
            let fish_y = self.fish[i].position.y;
            let nearest_idx = self.food.iter().enumerate().min_by(|(_, a), (_, b)| {
                let da = sq(head_x - a.position.x) + sq(fish_y - a.position.y);
                let db = sq(head_x - b.position.x) + sq(fish_y - b.position.y);
                da.partial_cmp(&db).unwrap()
            });
            if let Some((idx, _)) = nearest_idx {
                let mut rng = rand::rng();
                let center_x = self.fish[i].position.x + (fish_len - 1.0) / 2.0;
                let food_x = self.food[idx].position.x;
                let approach_right = center_x <= food_x;
                self.fish[i].seek_boost = rng.random_range(0.0_f32..SEEK_BOOST_INITIAL_MAX);
                self.fish[i].facing = if approach_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                self.fish[i].state = FishState::SeekingFood(idx, approach_right);
            }
        }
    }

    pub fn spawn_unfish(&mut self, rng: &mut impl RngExt) {
        if self.is_full() {
            return;
        }
        let kind = SPAWNABLE_UNFISH[rng.random_range(0..SPAWNABLE_UNFISH.len())];
        let actual_name = if kind == crate::entities::unfish::UnfishKind::Doppleganger {
            names::unique_name_in(&self.used_names, "Doppleganger")
        } else {
            names::unique_roman_in(&self.used_names)
        };
        let x_max = (self.width as f32 - 15.0).max(6.0);
        let y_max = (self.height as f32 - 5.0).max(3.0);
        let x = rng.random_range(5.0_f32..x_max);
        let y = rng.random_range(3.0_f32..y_max);
        let fish = crate::entities::fish::Fish::new_unfish(kind, actual_name.clone(), x, y, rng);
        self.used_names.insert(actual_name);
        self.fish.push(fish);
    }

    fn tick_void_spawn(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.void_spawn_timer -= dt;
        if self.void_spawn_timer <= 0.0 {
            self.void_spawn_timer = sample_exponential(rng, VOID_SPAWN_MEAN_SECS);
            self.spawn_unfish(rng);
        }
    }

    fn tick_dopplegangers(&mut self) {
        let doppleganger_idx = self.fish.iter().position(|f| {
            f.unfish_state.as_ref().is_some_and(|us| {
                us.kind == crate::entities::unfish::UnfishKind::Doppleganger
                    && !us.doppleganger_cloned
            })
        });
        let Some(d_idx) = doppleganger_idx else {
            return;
        };

        let target_idx = self.fish.iter().position(|f| f.unfish_state.is_none());
        let Some(t_idx) = target_idx else { return };

        let target_name = self.fish[t_idx].name.clone();
        let target_clone = self.fish[t_idx].clone();

        let old_unfish_state = self.fish[d_idx].unfish_state.take();
        let old_weight = self.fish[d_idx].weight_g;

        self.fish[d_idx] = target_clone;
        self.fish[d_idx].weight_g = old_weight;
        self.fish[d_idx].unfish_state = old_unfish_state;
        if let Some(ref mut us) = self.fish[d_idx].unfish_state {
            us.doppleganger_cloned = true;
        }

        let new_name = names::unique_name_in(&self.used_names, &format!("{}?", target_name));
        self.used_names.insert(new_name.clone());
        self.fish[d_idx].name = new_name;
    }

    fn tick_phantoms(&mut self, dt: f32, rng: &mut impl RngExt) -> Vec<TankEvent> {
        let mut events = Vec::new();
        for fish in &mut self.fish {
            let Some(ref mut us) = fish.unfish_state else {
                continue;
            };
            if us.kind != crate::entities::unfish::UnfishKind::Phantom {
                continue;
            }
            us.phantom_timer -= dt;
            if us.phantom_timer <= 0.0 {
                us.phantom_timer = sample_exponential(rng, PHANTOM_TELEPORT_MEAN);
                if rng.random::<f32>() < PHANTOM_CROSS_TANK_CHANCE {
                    events.push(TankEvent::PhantomCrossTank {
                        fish_name: fish.name.clone(),
                    });
                } else {
                    let max_x = (self.width as f32 - fish.display_width as f32).max(0.0);
                    let max_y = (self.height as f32 - 1.0).max(0.0);
                    fish.position.x = rng.random_range(0.0..=max_x);
                    fish.position.y = rng.random_range(0.0..=max_y);
                }
            }
        }
        events
    }
}

fn extend_plants(plants: &mut Vec<Plant>, to_width: i32, rng: &mut impl RngExt) {
    let mut next_x = if plants.is_empty() {
        rng.random_range(PLANT_SPACING_MIN..=PLANT_SPACING_MAX)
    } else {
        plants.last().unwrap().x + rng.random_range(PLANT_SPACING_MIN..=PLANT_SPACING_MAX)
    };
    while next_x < to_width {
        let height = rng.random_range(PLANT_HEIGHT_MIN..=PLANT_HEIGHT_MAX);
        plants.push(Plant::new(next_x, height));
        next_x += rng.random_range(PLANT_SPACING_MIN..=PLANT_SPACING_MAX);
    }
}

fn sq(x: f32) -> f32 {
    x * x
}

fn tail_kind_to_mutant_tail(kind: TailKind) -> MutantTail {
    match kind {
        TailKind::WideCurly => MutantTail::Curly,
        TailKind::Swaying { .. } => MutantTail::Swaying,
        _ => MutantTail::Wide,
    }
}

fn pick_random_miracle_mutation(
    _fish: &Fish,
    is_double: bool,
    has_glisten: bool,
    rng: &mut impl RngExt,
) -> Mutation {
    let mut valid: Vec<u8> = (0..10).collect();
    if !is_double {
        valid.push(10);
        valid.push(11);
    }
    if is_double {
        valid.push(12);
    }
    if !has_glisten {
        valid.push(13);
    }
    if has_glisten {
        valid.push(14);
    }
    match valid[rng.random_range(0..valid.len())] {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange {
            delta: if rng.random::<bool>() { 1 } else { -1 },
        },
        3 => Mutation::EyeColor,
        4 => Mutation::GlisteningSpeed {
            fast: rng.random::<bool>(),
        },
        5 => Mutation::GlisteningMode,
        6 => Mutation::GlisteningColor,
        7 => Mutation::BodyColor,
        8 => Mutation::BodyVariant,
        9 => Mutation::MouthVariant,
        10 => Mutation::TailVariant,
        11 => Mutation::Doublefish,
        12 => Mutation::Mitosis,
        13 => Mutation::GlisteningEnable,
        _ => Mutation::GlisteningDisable,
    }
}

fn pick_random_mutation(fish: &Fish, rng: &mut impl RngExt) -> Mutation {
    let is_double = fish.mutant.as_ref().is_some_and(|m| m.is_double);
    let has_glisten = fish
        .mutant
        .as_ref()
        .is_some_and(|m| m.glistening_color.is_some());
    let mut valid: Vec<u8> = (0..10).collect();
    if !is_double {
        valid.push(10);
        valid.push(11);
    }
    if is_double {
        valid.push(12);
    }
    if !has_glisten {
        valid.push(13);
    }
    if has_glisten {
        valid.push(14);
    }
    match valid[rng.random_range(0..valid.len())] {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange {
            delta: if rng.random::<bool>() { 1 } else { -1 },
        },
        3 => Mutation::EyeColor,
        4 => Mutation::GlisteningSpeed {
            fast: rng.random::<bool>(),
        },
        5 => Mutation::GlisteningMode,
        6 => Mutation::GlisteningColor,
        7 => Mutation::BodyColor,
        8 => Mutation::BodyVariant,
        9 => Mutation::MouthVariant,
        10 => Mutation::TailVariant,
        11 => Mutation::Doublefish,
        12 => Mutation::Mitosis,
        13 => Mutation::GlisteningEnable,
        _ => Mutation::GlisteningDisable,
    }
}

fn apply_mutation_to_fish(fish: &mut Fish, mutation: Mutation, rng: &mut impl RngExt) {
    let mutation_name = mutation.display_name().to_string();
    let m = fish.mutant.as_mut().unwrap();
    m.mutation_count += 1;
    m.mutation_history.push(mutation_name);
    match mutation {
        Mutation::SizeChange(delta) => {
            let max_eyes = m.left_eyes.len().max(m.right_eyes.len());
            let min_size = max_eyes + crate::entities::mutant::MIN_BODY_CHARS;
            let new_size = (fish.body_size as i32 + delta)
                .clamp(min_size as i32, MUTATION_MAX_BODY_SIZE as i32)
                as usize;
            fish.body_size = new_size;
        }
        Mutation::ColorPatch => {
            let count = rng.random_range(MUTATION_PATCH_COUNT_MIN..=MUTATION_PATCH_COUNT_MAX);
            let width = m.display_width(fish.body_size);
            for _ in 0..count {
                let pos = rng.random_range(0..width);
                m.color_patches.push((pos, random_rgb(rng)));
            }
            if m.color_patches.len() > MUTATION_PATCH_MAX {
                let excess = m.color_patches.len() - MUTATION_PATCH_MAX;
                m.color_patches.drain(0..excess);
            }
        }
        Mutation::EyeChange { delta } => {
            let max_eyes = fish
                .body_size
                .saturating_sub(crate::entities::mutant::MIN_BODY_CHARS)
                .clamp(1, 4) as i32;
            let new_left = (m.left_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
            while m.left_eyes.len() < new_left {
                m.left_eyes.push(EyeState::new(rng));
            }
            m.left_eyes.truncate(new_left);
            let new_right = (m.right_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
            while m.right_eyes.len() < new_right {
                m.right_eyes.push(EyeState::new(rng));
            }
            m.right_eyes.truncate(new_right);
        }
        Mutation::EyeColor => {
            m.eye_color = Some(random_rgb(rng));
        }
        Mutation::GlisteningSpeed { fast } => {
            fish.sway_speed *= if fast {
                GLISTEN_SPEED_FAST_MULT
            } else {
                GLISTEN_SPEED_SLOW_MULT
            };
            fish.sway_speed = fish.sway_speed.clamp(0.01, SWAY_SPEED_CLAMP_MAX);
        }
        Mutation::GlisteningMode => {
            m.glistening_mode = m.glistening_mode.random_other(rng);
        }
        Mutation::GlisteningColor => {
            m.glistening_color = if m.glistening_color.is_none() || rng.random::<bool>() {
                Some(random_rgb(rng))
            } else {
                None
            };
        }
        Mutation::BodyColor => {
            fish.color = random_rgb(rng);
            m.color_patches.clear();
            if m.eye_color.is_some() {
                m.eye_color = Some(random_rgb(rng));
            }
        }
        Mutation::BodyVariant => {
            m.body_variant = (m.body_variant + rng.random_range(1..4u8)) % 4;
        }
        Mutation::TailVariant => {
            m.tail_variant = match m.tail_variant {
                MutantTail::Wide => MutantTail::Swaying,
                MutantTail::Swaying => MutantTail::Curly,
                MutantTail::Curly => MutantTail::Wide,
            };
        }
        Mutation::MouthVariant => {
            m.mouth_inverted = !m.mouth_inverted;
        }
        Mutation::Doublefish => {
            let count = if fish.species == FishSpecies::Mutantfish {
                rng.random_range(1..=DOUBLE_EYE_COUNT_MAX)
            } else {
                match fish.species.config().body {
                    BodyTemplate::Standard(_) => 1,
                    BodyTemplate::Fixed { .. } => 0,
                }
            };
            m.double_head_eyes = (0..count).map(|_| EyeState::new(rng)).collect();
            m.is_double = true;
        }
        Mutation::Mitosis => unreachable!(),
        Mutation::GlisteningEnable => {
            fish.sway_speed = fish
                .species
                .config()
                .sway_speed
                .max(0.08)
                .clamp(0.01, SWAY_SPEED_CLAMP_MAX);
            m.glistening_color = Some(random_rgb(rng));
        }
        Mutation::GlisteningDisable => {
            fish.sway_speed = fish.species.config().sway_speed;
            m.glistening_color = None;
        }
    }
    fish.display_width = match fish.species.config().body {
        BodyTemplate::Fixed { left, .. } => {
            let n_chars = left.first().map(|s| s.chars().count()).unwrap_or(1);
            if n_chars > 1 {
                let n_eyes = m.left_eyes.len().max(m.right_eyes.len());
                if m.is_double {
                    2 * (1 + n_eyes + fish.body_size)
                } else {
                    2 + n_eyes + fish.body_size
                }
            } else if m.is_double {
                2
            } else {
                compute_display_width(fish.species, fish.body_size)
            }
        }
        _ => m.display_width(fish.body_size),
    };
}
