use std::collections::HashSet;

use rand::RngExt;

use ratatui::style::Color;

use crate::economy::{Purchasable, Rarity, Sellable};
use crate::tanks::alien::{AlienBackground, extend_alien_pyramids, extend_alien_tentacles};
use crate::entities::bubble::{Bubble, BubbleSpawner};
use crate::tanks::coral::{CoralStructure, FloorAlgae, extend_coral_reef};
use crate::fishes::fish::{Fish, FishState};
use crate::entities::food::Food;
use crate::tanks::hell::{HellBackground, HellPlant, extend_hell_plants};
use crate::entities::components::extend_spaced;
use crate::entities::plant::{Plant, Seaweed};
use crate::fishes::species::FishSpecies;
use crate::fishes::unfish::VOID_SPAWN_MEAN_SECS;
use crate::tanks::void::VoidBackground;
use crate::loot::ConsumableKind;
use crate::names;
use crate::settings::Settings;
use crate::util::sample_exponential;

mod mutations;
mod simulation;

pub enum TankEvent {
    PhantomCrossTank { fish_name: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TankKind {
    Base,
    CoralReef,
    Hell,
    Void,
    Alien,
}

pub struct TankConfig {
    pub display_name: &'static str,
    pub parse_name: &'static str,
    pub un_name: &'static str,
    pub shop_name: &'static str,
    pub buy_price: u32,
    pub sell_price: u32,
    pub capacity: usize,
    pub bubble_color: Color,
    pub rarity: Rarity,
}

impl TankKind {
    pub fn config(self) -> TankConfig {
        match self {
            TankKind::Base => TankConfig {
                display_name: "Fishtank",
                parse_name: "fishtank",
                un_name: "UnFishtank",
                shop_name: "Base Fishtank",
                buy_price: 3000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: Color::Cyan,
                rarity: Rarity::Common,
            },
            TankKind::CoralReef => TankConfig {
                display_name: "Coralreeftank",
                parse_name: "coralreeftank",
                un_name: "UnCoral Reef",
                shop_name: "Coral Reef Fishtank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: Color::Cyan,
                rarity: Rarity::Rare,
            },
            TankKind::Hell => TankConfig {
                display_name: "Helltank",
                parse_name: "helltank",
                un_name: "UnHelltank",
                shop_name: "Helltank",
                buy_price: 8000,
                sell_price: 7000,
                capacity: 100,
                bubble_color: Color::Red,
                rarity: Rarity::Legendary,
            },
            TankKind::Void => TankConfig {
                display_name: "Voidtank",
                parse_name: "voidtank",
                un_name: "UnVoidtank",
                shop_name: "Voidtank",
                buy_price: 8000,
                sell_price: 7000,
                capacity: 100,
                bubble_color: Color::Cyan,
                rarity: Rarity::Legendary,
            },
            TankKind::Alien => TankConfig {
                display_name: "Alientank",
                parse_name: "alientank",
                un_name: "UnAlientank",
                shop_name: "Alientank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 75,
                bubble_color: Color::Cyan,
                rarity: Rarity::Rare,
            },
        }
    }

    pub fn display_name(self) -> &'static str { self.config().display_name }
    pub fn un_name(self) -> &'static str { self.config().un_name }
    pub fn buy_price(self) -> u32 { self.config().buy_price }
    pub fn sell_price(self) -> u32 { self.config().sell_price }
    pub fn shop_name(self) -> &'static str { self.config().shop_name }
}

impl Purchasable for TankKind {
    fn buy_price(&self) -> u32 { TankKind::buy_price(*self) }
    fn display_name(&self) -> &str { TankKind::display_name(*self) }
}

impl Sellable for TankKind {
    fn sell_price(&self) -> u32 { TankKind::sell_price(*self) }
    fn display_name(&self) -> &str { TankKind::display_name(*self) }
}

impl TankKind {

    pub fn parse(s: &str) -> Option<Self> {
        let normalized: String = s.to_ascii_lowercase().split_whitespace().collect();
        Self::all().iter().find(|&&k| k.config().parse_name == normalized.as_str()).copied()
    }

    pub fn all() -> &'static [TankKind] {
        &[
            TankKind::Base,
            TankKind::CoralReef,
            TankKind::Hell,
            TankKind::Void,
            TankKind::Alien,
        ]
    }
}

pub struct ActiveConsumable {
    pub kind: ConsumableKind,
    pub stacks: u32,
    pub time_remaining: f32,
}


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
const BUBBLE_ZOOMIE_CHANCE: f32 = 0.45;
const MUTATION_INTERVAL_BASE: f32 = 30.0 * 60.0;
const MUTATION_ALPHA: f32 = 1.0 / 3.0;
const MUTATION_MEAN_FLOOR_SECS: f32 = 3.0;
const MIN_SPLIT_BODY_SIZE: usize = 2;
const FISH_SPAWN_X_MIN: f32 = 5.0;
const FISH_SPAWN_X_MAX_OFFSET: f32 = 15.0;
const FISH_SPAWN_X_SAFE_MIN: f32 = 6.0;
const FISH_SPAWN_Y_MIN: f32 = 2.0;
const FISH_SPAWN_Y_MAX_OFFSET: f32 = 5.0;
const FISH_SPAWN_Y_SAFE_MIN: f32 = 3.0;
const FOOD_WEIGHT_GAIN_G: u32 = 50;

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
    pub alien_bg: Option<AlienBackground>,
    pub width: u16,
    pub height: u16,
    pub used_names: HashSet<String>,
    pub pending_star_cash: u32,
    pub extra_capacity: u32,
    bubble_spawner: BubbleSpawner,
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
        let mut alien_bg = None;
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
            TankKind::Alien => {
                let mut bg = AlienBackground::new(&mut rng);
                let color = bg.color;
                extend_alien_tentacles(
                    &mut bg.tentacles,
                    INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                    color,
                    &mut rng,
                );
                extend_alien_pyramids(
                    &mut bg.pyramids,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    &mut rng,
                );
                bg.init_stars(&mut rng, INITIAL_WIDTH, INITIAL_HEIGHT);
                alien_bg = Some(bg);
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
            alien_bg,
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
            used_names: HashSet::new(),
            pending_star_cash: 0,
            extra_capacity: 0,
            bubble_spawner: BubbleSpawner::new(&mut rng),
            mutation_timer: MUTATION_INTERVAL_BASE,
            void_spawn_timer: sample_exponential(&mut rng, VOID_SPAWN_MEAN_SECS),
        }
    }

    pub fn capacity(&self) -> usize {
        self.kind.config().capacity + self.extra_capacity as usize
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
        let mut rng = rand::rng();
        if width > old_width {
            self.extend_environment(&mut rng);
        }
        if let Some(bg) = &mut self.alien_bg {
            bg.init_stars(&mut rng, width, height);
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
            TankKind::Alien => {
                if let Some(bg) = &mut self.alien_bg {
                    let color = bg.color;
                    extend_alien_tentacles(
                        &mut bg.tentacles,
                        self.width as i32 + PLANT_SPAWN_LOOKAHEAD,
                        color,
                        rng,
                    );
                    extend_alien_pyramids(
                        &mut bg.pyramids,
                        self.width as i32 + CORAL_SPAWN_LOOKAHEAD,
                        rng,
                    );
                }
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
            plant.tick(dt);
        }
        if let Some(bg) = &mut self.hell_bg {
            bg.tick(dt, &mut rng, self.width, self.height);
        }
        if let Some(bg) = &mut self.void_bg {
            bg.tick();
        }
        if let Some(bg) = &mut self.alien_bg {
            bg.tick(dt, &mut rng, self.width, self.height);
        }
        for plant in &mut self.hell_plants {
            plant.tick(dt);
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
        self.pending_star_cash += star;
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

    fn unique_name(&self, requested: &str) -> String {
        names::unique_name_in(&self.used_names, requested)
    }
}

fn extend_plants(plants: &mut Vec<Plant>, to_width: i32, rng: &mut impl RngExt) {
    extend_spaced(
        plants, to_width, 0,
        PLANT_SPACING_MIN, PLANT_SPACING_MAX,
        rng, |p| p.x,
        |x, rng| Plant::new(x, rng.random_range(PLANT_HEIGHT_MIN..=PLANT_HEIGHT_MAX)),
    );
}
