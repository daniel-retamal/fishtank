use std::collections::HashSet;

use rand::RngExt;

use ratatui::style::Color;

use crate::colors::{
    CYAN, INDIGO, LIGHT_MAGENTA, MAGENTA, PINK, PURPLE, PURPLE_DARK, PURPLE_LIGHT, RED, VIOLET,
    WHITE,
};

use crate::economy::{Purchasable, Rarity, Sellable};
use crate::entities::bubble::{Bubble, BubbleSpawner};
use crate::entities::components::extend_spaced;
use crate::entities::cow::{Cow, CowVariant, cow_display_width};
use crate::entities::food::Food;
use crate::entities::plant::{Plant, Seaweed};
use crate::entities::ufo::Ufo;
use crate::fishes::fish::{Fish, FishState};
use crate::fishes::species::FishSpecies;
use crate::fishes::unfish::VOID_SPAWN_MEAN_SECS;
use crate::loot::ConsumableKind;
use crate::names;
use crate::settings::Settings;
use crate::tanks::alien::{AlienBackground, extend_alien_pyramids, extend_alien_tentacles};
use crate::tanks::candy::{
    CandyBackground, extend_candy_decos, extend_candy_plants, man_sway_offset,
};
use crate::tanks::coral::{CoralStructure, FloorAlgae, extend_coral_reef};
use crate::tanks::desert::{DesertSky, extend_desert_cacti};
use crate::tanks::haunted::{HauntedBackground, extend_haunted};
use crate::tanks::hell::{HellBackground, HellPlant, extend_hell_plants};
use crate::tanks::void::VoidBackground;
use crate::util::sample_exponential;

mod mutations;
mod simulation;

pub enum TankEvent {
    PhantomCrossTank { fish_name: String },
    UfoTimerFired,
    UfoLockFish { fish_name: String },
    UfoTakeFish { fish_name: String },
    UfoReleaseFish(Box<Fish>),
    UfoReleaseCow(Box<Cow>),
    UfoFinished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TankKind {
    Base,
    CoralReef,
    Hell,
    Void,
    Alien,
    Haunted,
    Candy,
    Desert,
}

pub struct TankConfig {
    pub display_name: &'static str,
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
                buy_price: 3000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: CYAN,
                rarity: Rarity::Common,
            },
            TankKind::CoralReef => TankConfig {
                display_name: "Coralreeftank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: CYAN,
                rarity: Rarity::Rare,
            },
            TankKind::Hell => TankConfig {
                display_name: "Helltank",
                buy_price: 8000,
                sell_price: 7000,
                capacity: 100,
                bubble_color: RED,
                rarity: Rarity::Legendary,
            },
            TankKind::Void => TankConfig {
                display_name: "Voidtank",
                buy_price: 8000,
                sell_price: 7000,
                capacity: 100,
                bubble_color: CYAN,
                rarity: Rarity::Legendary,
            },
            TankKind::Alien => TankConfig {
                display_name: "Alientank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 100,
                bubble_color: CYAN,
                rarity: Rarity::Rare,
            },
            TankKind::Haunted => TankConfig {
                display_name: "Hauntedtank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: WHITE,
                rarity: Rarity::Rare,
            },
            TankKind::Candy => TankConfig {
                display_name: "Candytank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: PINK,
                rarity: Rarity::Rare,
            },
            TankKind::Desert => TankConfig {
                display_name: "Desertank",
                buy_price: 5000,
                sell_price: 2500,
                capacity: 50,
                bubble_color: CYAN,
                rarity: Rarity::Rare,
            },
        }
    }

    pub fn display_name(self) -> &'static str {
        self.config().display_name
    }
    pub fn un_name(self) -> String {
        format!("Un{}", self.config().display_name)
    }
    pub fn buy_price(self) -> u32 {
        self.config().buy_price
    }
    pub fn sell_price(self) -> u32 {
        self.config().sell_price
    }
}

impl Purchasable for TankKind {
    fn buy_price(&self) -> u32 {
        TankKind::buy_price(*self)
    }
    fn display_name(&self) -> &str {
        TankKind::display_name(*self)
    }
}

impl Sellable for TankKind {
    fn sell_price(&self) -> u32 {
        TankKind::sell_price(*self)
    }
    fn display_name(&self) -> &str {
        TankKind::display_name(*self)
    }
}

impl TankKind {
    pub fn parse(s: &str) -> Option<Self> {
        let normalized: String = s.to_ascii_lowercase().split_whitespace().collect();
        Self::all()
            .iter()
            .find(|&&k| k.config().display_name.to_ascii_lowercase() == normalized.as_str())
            .copied()
    }

    pub fn all() -> &'static [TankKind] {
        &[
            TankKind::Base,
            TankKind::CoralReef,
            TankKind::Hell,
            TankKind::Void,
            TankKind::Alien,
            TankKind::Haunted,
            TankKind::Candy,
            TankKind::Desert,
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
const CANDY_PINK_PLANT_COLORS: &[Color] = &[
    PINK,
    MAGENTA,
    LIGHT_MAGENTA,
    PURPLE_DARK,
    PURPLE,
    PURPLE_LIGHT,
    VIOLET,
    INDIGO,
];
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
pub const UFO_MEAN_SECS: f32 = 60.0 * 60.0;
const UFO_DESERT_FREQUENCY_MULT: f32 = 2.0;

fn ufo_mean_secs(kind: TankKind) -> f32 {
    if kind == TankKind::Desert {
        UFO_MEAN_SECS / UFO_DESERT_FREQUENCY_MULT
    } else {
        UFO_MEAN_SECS
    }
}
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
    pub cows: Vec<Cow>,
    pub food: Vec<Food>,
    pub plants: Vec<Plant>,
    pub corals: Vec<CoralStructure>,
    pub floor_algae: Vec<FloorAlgae>,
    pub bubbles: Vec<Bubble>,
    pub hell_bg: Option<HellBackground>,
    pub hell_plants: Vec<HellPlant>,
    pub void_bg: Option<VoidBackground>,
    pub alien_bg: Option<AlienBackground>,
    pub haunted_bg: Option<HauntedBackground>,
    pub candy_bg: Option<CandyBackground>,
    pub desert_bg: Option<DesertSky>,
    pub width: u16,
    pub height: u16,
    pub used_names: HashSet<String>,
    pub used_cow_names: HashSet<String>,
    pub pending_star_cash: u32,
    pub extra_capacity: u32,
    pub cow_abduction_count: u32,
    bubble_spawner: BubbleSpawner,
    mutation_timer: f32,
    void_spawn_timer: f32,
    pub ufo_timer: f32,
    pub ufo: Option<Ufo>,
    pub(super) candy_tick: u32,
}

impl Tank {
    pub fn new(name: String, kind: TankKind, dead_names: &[String]) -> Self {
        let mut rng = rand::rng();
        let mut plants = Vec::new();
        let mut corals = Vec::new();
        let mut floor_algae = Vec::new();
        let mut hell_bg = None;
        let mut hell_plants = Vec::new();
        let mut void_bg = None;
        let mut alien_bg = None;
        let mut haunted_bg = None;
        let mut candy_bg = None;
        let mut desert_bg = None;
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
            TankKind::Haunted => {
                let mut bg = HauntedBackground::new(&mut rng);
                extend_haunted(
                    &mut bg.graves,
                    &mut bg.pumpkins,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    dead_names,
                    &mut rng,
                );
                haunted_bg = Some(bg);
            }
            TankKind::Candy => {
                let mut bg = CandyBackground::new();
                let target = INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD;
                extend_candy_plants(&mut bg.plants, target, &mut rng);
                extend_candy_decos(&mut bg.decos, target, &mut rng);
                extend_candy_pink_plants(&mut plants, target, &mut rng);
                candy_bg = Some(bg);
            }
            TankKind::Desert => {
                let mut bg = DesertSky::new(&mut rng);
                extend_desert_cacti(
                    &mut bg.cacti,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    &mut rng,
                );
                bg.init_stars(&mut rng, INITIAL_WIDTH, INITIAL_HEIGHT);
                desert_bg = Some(bg);
            }
        }
        Self {
            name,
            kind,
            fish: Vec::new(),
            cows: Vec::new(),
            food: Vec::new(),
            plants,
            corals,
            floor_algae,
            bubbles: Vec::new(),
            hell_bg,
            hell_plants,
            void_bg,
            alien_bg,
            haunted_bg,
            candy_bg,
            desert_bg,
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
            used_names: HashSet::new(),
            used_cow_names: HashSet::new(),
            pending_star_cash: 0,
            extra_capacity: 0,
            cow_abduction_count: 0,
            bubble_spawner: BubbleSpawner::new(&mut rng),
            mutation_timer: MUTATION_INTERVAL_BASE,
            void_spawn_timer: sample_exponential(&mut rng, VOID_SPAWN_MEAN_SECS),
            ufo_timer: sample_exponential(&mut rng, ufo_mean_secs(kind)),
            ufo: None,
            candy_tick: 0,
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

    pub fn resize(&mut self, width: u16, height: u16, dead_names: &[String]) {
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
        let cow_floor = (height as f32) - (crate::entities::cow::Cow::sprite_height() as f32);
        for cow in &mut self.cows {
            cow.position.y = cow_floor.max(0.0);
            let max_x = (width as i32 - cow.display_width as i32).max(0) as f32;
            if cow.position.x > max_x {
                cow.position.x = max_x;
            }
        }
        let mut rng = rand::rng();
        if width > old_width {
            self.extend_environment(&mut rng, dead_names);
        }
        if let Some(bg) = &mut self.alien_bg {
            bg.init_stars(&mut rng, width, height);
        }
        if let Some(bg) = &mut self.desert_bg {
            bg.init_stars(&mut rng, width, height);
        }
    }

    pub fn clear_grave_name(&mut self, name: &str) {
        if let Some(bg) = &mut self.haunted_bg {
            bg.clear_grave_name(name);
        }
    }

    fn extend_environment(&mut self, rng: &mut impl RngExt, dead_names: &[String]) {
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
            TankKind::Haunted => {
                if let Some(bg) = &mut self.haunted_bg {
                    extend_haunted(
                        &mut bg.graves,
                        &mut bg.pumpkins,
                        self.width as i32 + CORAL_SPAWN_LOOKAHEAD,
                        dead_names,
                        rng,
                    );
                }
            }
            TankKind::Candy => {
                if let Some(bg) = &mut self.candy_bg {
                    let target = self.width as i32 + CORAL_SPAWN_LOOKAHEAD;
                    extend_candy_plants(&mut bg.plants, target, rng);
                    extend_candy_decos(&mut bg.decos, target, rng);
                    extend_candy_pink_plants(&mut self.plants, target, rng);
                }
            }
            TankKind::Desert => {
                if let Some(bg) = &mut self.desert_bg {
                    extend_desert_cacti(
                        &mut bg.cacti,
                        self.width as i32 + CORAL_SPAWN_LOOKAHEAD,
                        rng,
                    );
                }
            }
        }
    }

    pub fn candy_man_sway(&self) -> i32 {
        man_sway_offset(self.candy_tick)
    }

    pub(super) fn mark_if_hell(&self, fish: &mut Fish) {
        if self.kind == TankKind::Hell {
            fish.devil_marked = true;
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
        self.mark_if_hell(&mut fish);
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
        self.mark_if_hell(&mut fish);
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
        if let Some(bg) = &mut self.haunted_bg {
            bg.tick(dt, &mut rng, self.width, self.height);
        }
        if let Some(bg) = &mut self.desert_bg {
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

        self.candy_tick = self.candy_tick.wrapping_add(1);
        self.steer_seeking_fish();
        self.tick_fish(settings, coffee);
        self.spawn_bubbles(dt);
        self.tick_candyfish_effects();
        self.check_eating_collisions();

        let prev_len = self.food.len();
        self.food.retain(|f| !f.eaten);
        if self.food.len() < prev_len {
            for fish in &mut self.fish {
                if matches!(fish.state, FishState::SeekingFood { .. }) {
                    fish.cancel_seek();
                }
            }
        }

        self.assign_food_to_idle_fish();
        self.tick_mutations(dt);
        self.tick_dopplegangers();
        self.tick_cows(dt);
        if self.kind == TankKind::Void {
            self.tick_void_spawn(dt, &mut rng);
        }
        let mut events = self.tick_phantoms(dt, &mut rng);
        self.tick_ufo_timer(dt, &mut rng, &mut events);
        self.tick_ufo_animation(dt, &mut events);
        events
    }

    fn tick_ufo_timer(&mut self, dt: f32, rng: &mut impl RngExt, events: &mut Vec<TankEvent>) {
        self.ufo_timer -= dt;
        if self.ufo_timer <= 0.0 {
            self.ufo_timer = sample_exponential(rng, ufo_mean_secs(self.kind));
            events.push(TankEvent::UfoTimerFired);
        }
    }

    fn tick_ufo_animation(&mut self, dt: f32, events: &mut Vec<TankEvent>) {
        use crate::entities::ufo::{
            UFO_CENTER_COL, UFO_PAYLOAD_CONE_ROW, UFO_SHIP_ROWS, UfoPayload, UfoPhase,
            UfoTickResult,
        };
        let Some(ufo) = self.ufo.as_mut() else { return };
        if let UfoPayload::AbductingFish { fish_name } = &ufo.payload
            && let Some(fish) = self.fish.iter().find(|f| &f.name == fish_name)
        {
            let target_x =
                fish.position.x + fish.display_width as f32 / 2.0 - UFO_CENTER_COL as f32;
            let target_y = fish.position.y - (UFO_SHIP_ROWS + UFO_PAYLOAD_CONE_ROW) as f32;
            match ufo.phase {
                UfoPhase::Descending => {
                    ufo.x = target_x;
                    ufo.target_y = target_y;
                }
                UfoPhase::GrowingCone { .. } => {
                    ufo.x = target_x;
                    ufo.y = target_y;
                }
                _ => {}
            }
        }
        match ufo.tick(dt) {
            UfoTickResult::None => {}
            UfoTickResult::LockFish(name) => {
                events.push(TankEvent::UfoLockFish { fish_name: name });
            }
            UfoTickResult::TakeFish(name) => {
                events.push(TankEvent::UfoTakeFish { fish_name: name });
            }
            UfoTickResult::ReleaseFish(f) => {
                events.push(TankEvent::UfoReleaseFish(Box::new(f)));
            }
            UfoTickResult::ReleaseCow(c) => {
                events.push(TankEvent::UfoReleaseCow(Box::new(c)));
            }
            UfoTickResult::Finished => {
                events.push(TankEvent::UfoFinished);
            }
        }
        if ufo.is_done() {
            self.ufo = None;
        }
    }

    fn unique_name(&self, requested: &str) -> String {
        names::unique_name_in(&self.used_names, requested)
    }

    pub fn unique_cow_name(&self, requested: &str) -> String {
        names::unique_name_in(&self.used_cow_names, requested)
    }

    pub fn spawn_cow(&mut self, variant: CowVariant, rng: &mut impl RngExt) -> String {
        let name = self.unique_cow_name("Vaquita");
        let cow = Cow::new(name.clone(), variant, 0.0, 0.0, rng);
        self.used_cow_names.insert(name.clone());
        let cow_floor = (self.height as f32) - (Cow::sprite_height() as f32);
        let max_x = (self.width as i32 - cow_display_width(&cow) as i32).max(0);
        let x = if max_x > 0 {
            rng.random_range(0..max_x) as f32
        } else {
            0.0
        };
        self.cows.push(Cow {
            position: crate::entities::components::Position {
                x,
                y: cow_floor.max(0.0),
            },
            ..cow
        });
        name
    }

    pub fn place_cow_dropped(&mut self, mut cow: Cow) {
        let actual = self.unique_cow_name(&cow.name);
        cow.name = actual.clone();
        let cow_floor = (self.height as f32) - (Cow::sprite_height() as f32);
        cow.position.y = cow_floor.max(0.0);
        let max_x = (self.width as i32 - cow.display_width as i32).max(0) as f32;
        if cow.position.x > max_x {
            cow.position.x = max_x;
        }
        if cow.position.x < 0.0 {
            cow.position.x = 0.0;
        }
        self.used_cow_names.insert(actual);
        self.cows.push(cow);
    }

    pub fn place_cow(&mut self, mut cow: Cow, rng: &mut impl RngExt) {
        let actual = self.unique_cow_name(&cow.name);
        cow.name = actual.clone();
        let cow_floor = (self.height as f32) - (Cow::sprite_height() as f32);
        let max_x = (self.width as i32 - cow.display_width as i32).max(0);
        cow.position.x = if max_x > 0 {
            rng.random_range(0..max_x) as f32
        } else {
            0.0
        };
        cow.position.y = cow_floor.max(0.0);
        self.used_cow_names.insert(actual);
        self.cows.push(cow);
    }

    pub fn cow_count_by_variant(&self, variant: CowVariant) -> u32 {
        self.cows.iter().filter(|c| c.variant == variant).count() as u32
    }

    pub fn has_cow(&self) -> bool {
        !self.cows.is_empty()
    }

    fn tick_cows(&mut self, dt: f32) {
        for cow in &mut self.cows {
            cow.tick(dt);
        }
    }
}

fn extend_plants(plants: &mut Vec<Plant>, to_width: i32, rng: &mut impl RngExt) {
    extend_spaced(
        plants,
        to_width,
        0,
        PLANT_SPACING_MIN..=PLANT_SPACING_MAX,
        rng,
        |p| p.x,
        |x, rng| Plant::new(x, rng.random_range(PLANT_HEIGHT_MIN..=PLANT_HEIGHT_MAX)),
    );
}

fn extend_candy_pink_plants(plants: &mut Vec<Plant>, to_width: i32, rng: &mut impl RngExt) {
    extend_spaced(
        plants,
        to_width,
        0,
        PLANT_SPACING_MIN..=PLANT_SPACING_MAX,
        rng,
        |p| p.x,
        |x, rng| {
            let mut p = Plant::new(x, rng.random_range(PLANT_HEIGHT_MIN..=PLANT_HEIGHT_MAX));
            p.color = CANDY_PINK_PLANT_COLORS[rng.random_range(0..CANDY_PINK_PLANT_COLORS.len())];
            p
        },
    );
}
