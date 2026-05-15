use std::collections::HashSet;

use rand::RngExt;

use crate::entities::bubble::Bubble;
use crate::entities::fish::{Direction, EATING_DURATION, Fish, FishState};
use crate::entities::food::Food;
use crate::entities::mutant::{
    EXTRA_BODY_FOR_DOUBLE, EyeState, MIN_BODY_CHARS, MutantTail, random_rgb,
};
use crate::entities::plant::Plant;
use crate::entities::species::FishSpecies;
use crate::loot::ConsumableKind;
use crate::names;
use crate::settings::Settings;

pub struct ActiveConsumable {
    pub kind: ConsumableKind,
    pub stacks: u32,
    pub time_remaining: f32,
}

pub const TANK_CAPACITY: usize = 50;

const PLANT_FIELD_WIDTH: i32 = 600;
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
const BUBBLE_BOTTOM_SPAWN_RATE_MIN: f32 = 0.3;
const BUBBLE_BOTTOM_SPAWN_RATE_MAX: f32 = 1.5;
const BUBBLE_SURFACE_SPAWN_RATE_MIN: f32 = 1.0;
const BUBBLE_SURFACE_SPAWN_RATE_MAX: f32 = 3.0;
const BUBBLE_ZOOMIE_CHANCE: f32 = 0.45;
const MUTATION_INTERVAL_BASE: f32 = 30.0 * 60.0;
const MUTATION_MAX_BODY_SIZE: usize = 12;
const MUTATION_PATCH_MAX: usize = 12;
const MUTATION_PATCH_COUNT_MIN: usize = 2;
const MUTATION_PATCH_COUNT_MAX: usize = 4;
const GLISTEN_SPEED_FAST_MULT: f32 = 3.5;
const GLISTEN_SPEED_SLOW_MULT: f32 = 0.25;
const SWAY_SPEED_CLAMP_MAX: f32 = 2.5;
const DOUBLE_EYE_COUNT_MAX: usize = 3;
const MIN_SPLIT_BODY_SIZE: usize = 2;

enum Mutation {
    SizeChange(i32),
    ColorPatch,
    EyeChange { left: bool, delta: i32 },
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
}

pub struct Tank {
    pub name: String,
    pub fish: Vec<Fish>,
    pub food: Vec<Food>,
    pub plants: Vec<Plant>,
    pub bubbles: Vec<Bubble>,
    pub width: u16,
    pub height: u16,
    pub used_names: HashSet<String>,
    pub pending_star_money: u32,
    bubble_bottom_timer: f32,
    bubble_surface_timer: f32,
    mutation_timer: f32,
}

impl Tank {
    pub fn new(name: String) -> Self {
        let mut rng = rand::rng();
        Self {
            name,
            fish: Vec::new(),
            food: Vec::new(),
            plants: Self::generate_plants(),
            bubbles: Vec::new(),
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
            used_names: HashSet::new(),
            pending_star_money: 0,
            bubble_bottom_timer: rng
                .random_range(BUBBLE_BOTTOM_SPAWN_RATE_MIN..BUBBLE_BOTTOM_SPAWN_RATE_MAX),
            bubble_surface_timer: rng
                .random_range(BUBBLE_SURFACE_SPAWN_RATE_MIN..BUBBLE_SURFACE_SPAWN_RATE_MAX),
            mutation_timer: MUTATION_INTERVAL_BASE,
        }
    }

    pub fn is_full(&self) -> bool {
        self.fish.len() >= TANK_CAPACITY
    }

    pub fn resize(&mut self, width: u16, height: u16) {
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
        let x_max = (self.width as f32 - 15.0).max(6.0);
        let y_max = (self.height as f32 - 5.0).max(3.0);
        let x = rng.random_range(5.0..x_max);
        let y = rng.random_range(2.0..y_max);
        let fish = Fish::new(species, actual_name.clone(), x, y, rng);
        self.used_names.insert(actual_name);
        self.fish.push(fish);
        true
    }

    pub fn place_fish(&mut self, mut fish: Fish, name: String, rng: &mut impl RngExt) {
        let actual_name = self.unique_name(&name);
        let x_max = (self.width as f32 - 15.0).max(6.0);
        let y_max = (self.height as f32 - 5.0).max(3.0);
        fish.position.x = rng.random_range(5.0..x_max);
        fish.position.y = rng.random_range(2.0..y_max);
        fish.name = actual_name.clone();
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

    pub fn tick(&mut self, settings: &Settings, coffee: u32) {
        let dt = 1.0 / settings.fps;

        for plant in &mut self.plants {
            plant.tick();
        }
        for food in &mut self.food {
            food.tick(settings, self.width, self.height);
        }
        let star: u32 = self.bubbles.iter_mut()
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
            self.mutation_timer = MUTATION_INTERVAL_BASE / mutant_count as f32;
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

        self.used_names.insert(new_name);
        self.fish.push(new_fish);
    }

    pub fn apply_named_mutation(&mut self, fish_name: &str, mutation_name: &str) {
        let fish_idx = match self.fish.iter().position(|f| f.name == fish_name) {
            Some(i) => i,
            None => return,
        };
        if self.fish[fish_idx].species != FishSpecies::Mutantfish {
            return;
        }
        let is_double = self.fish[fish_idx]
            .mutant
            .as_ref()
            .is_some_and(|m| m.is_double);
        if mutation_name.eq_ignore_ascii_case("mitosis") {
            if is_double {
                self.apply_mitosis(fish_idx);
            }
            return;
        }
        let mut rng = rand::rng();
        let mutation = match mutation_name.to_ascii_lowercase().as_str() {
            "size+" => Mutation::SizeChange(1),
            "size-" => Mutation::SizeChange(-1),
            "colorpatch" => Mutation::ColorPatch,
            "eye_left+" => Mutation::EyeChange {
                left: true,
                delta: 1,
            },
            "eye_left-" => Mutation::EyeChange {
                left: true,
                delta: -1,
            },
            "eye_right+" => Mutation::EyeChange {
                left: false,
                delta: 1,
            },
            "eye_right-" => Mutation::EyeChange {
                left: false,
                delta: -1,
            },
            "eyecolor" => Mutation::EyeColor,
            "glistenfast" => Mutation::GlisteningSpeed { fast: true },
            "glisten_slow" => Mutation::GlisteningSpeed { fast: false },
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
            _ => return,
        };
        apply_mutation_to_fish(&mut self.fish[fish_idx], mutation, &mut rng);
    }

    fn generate_plants() -> Vec<Plant> {
        let mut rng = rand::rng();
        let mut plants = Vec::new();
        let mut x = rng.random_range(PLANT_SPACING_MIN..=PLANT_SPACING_MAX);
        while x < PLANT_FIELD_WIDTH {
            let height = rng.random_range(PLANT_HEIGHT_MIN..=PLANT_HEIGHT_MAX);
            plants.push(Plant::new(x, height));
            x += rng.random_range(PLANT_SPACING_MIN..=PLANT_SPACING_MAX);
        }
        plants
    }

    fn unique_name(&self, requested: &str) -> String {
        names::unique_name_in(&self.used_names, requested)
    }

    fn spawn_bubbles(&mut self, dt: f32) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let mut rng = rand::rng();

        self.bubble_bottom_timer -= dt;
        if self.bubble_bottom_timer <= 0.0 {
            let x = rng.random_range(0.0..self.width as f32);
            let y = (self.height as f32 - 1.0).max(0.0);
            self.bubbles.push(Bubble::new_rising(x, y));
            self.bubble_bottom_timer =
                rng.random_range(BUBBLE_BOTTOM_SPAWN_RATE_MIN..BUBBLE_BOTTOM_SPAWN_RATE_MAX);
        }

        self.bubble_surface_timer -= dt;
        if self.bubble_surface_timer <= 0.0 {
            let x = rng.random_range(0.0..self.width as f32);
            let max_y = (self.height as f32 * 0.25).max(1.0);
            let y = rng.random_range(0.0..max_y);
            self.bubbles.push(Bubble::new_surface(x, y));
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
                    _ => Bubble::new_rising(tail_x, fish.position.y),
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
                let cap = self.fish[i].species.config().weight_cap[cat as usize];
                let new_w = self.fish[i].weight_g + 50;
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
}

fn sq(x: f32) -> f32 {
    x * x
}

fn pick_random_mutation(fish: &Fish, rng: &mut impl RngExt) -> Mutation {
    let is_double = fish.mutant.as_ref().is_some_and(|m| m.is_double);
    let mut valid: Vec<u8> = (0..11).collect();
    if !is_double {
        valid.push(11);
        valid.push(12);
    }
    if is_double {
        valid.push(13);
    }
    match valid[rng.random_range(0..valid.len())] {
        0 => Mutation::SizeChange(if rng.random::<bool>() { 1 } else { -1 }),
        1 => Mutation::ColorPatch,
        2 => Mutation::EyeChange {
            left: true,
            delta: if rng.random::<bool>() { 1 } else { -1 },
        },
        3 => Mutation::EyeChange {
            left: false,
            delta: if rng.random::<bool>() { 1 } else { -1 },
        },
        4 => Mutation::EyeColor,
        5 => Mutation::GlisteningSpeed {
            fast: rng.random::<bool>(),
        },
        6 => Mutation::GlisteningMode,
        7 => Mutation::GlisteningColor,
        8 => Mutation::BodyColor,
        9 => Mutation::BodyVariant,
        10 => Mutation::MouthVariant,
        11 => Mutation::TailVariant,
        12 => Mutation::Doublefish,
        _ => Mutation::Mitosis,
    }
}

fn apply_mutation_to_fish(fish: &mut Fish, mutation: Mutation, rng: &mut impl RngExt) {
    let m = fish.mutant.as_mut().unwrap();
    m.mutation_count += 1;
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
        Mutation::EyeChange { left, delta } => {
            let max_eyes = fish
                .body_size
                .saturating_sub(crate::entities::mutant::MIN_BODY_CHARS)
                .clamp(1, 4) as i32;
            if left {
                let new_count = (m.left_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
                while m.left_eyes.len() < new_count {
                    m.left_eyes.push(EyeState::new(rng));
                }
                m.left_eyes.truncate(new_count);
            } else {
                let new_count = (m.right_eyes.len() as i32 + delta).clamp(1, max_eyes) as usize;
                while m.right_eyes.len() < new_count {
                    m.right_eyes.push(EyeState::new(rng));
                }
                m.right_eyes.truncate(new_count);
            }
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
            let count = rng.random_range(1..=DOUBLE_EYE_COUNT_MAX);
            m.double_head_eyes = (0..count).map(|_| EyeState::new(rng)).collect();
            m.is_double = true;
        }
        Mutation::Mitosis => unreachable!(),
    }
    fish.display_width = fish.mutant.as_ref().unwrap().display_width(fish.body_size);
}
