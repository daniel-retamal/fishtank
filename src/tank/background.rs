use rand::RngExt;

use ratatui::style::Color;

use crate::colors::{
    INDIGO, LIGHT_MAGENTA, MAGENTA, PINK, PURPLE, PURPLE_DARK, PURPLE_LIGHT, VIOLET,
};
use crate::entities::components::extend_spaced;
use crate::entities::plant::{Plant, Seaweed};
use crate::tanks::alien::{AlienBackground, extend_alien_pyramids, extend_alien_tentacles};
use crate::tanks::candy::{CandyBackground, extend_candy_decos, extend_candy_plants};
use crate::tanks::coral::{CoralStructure, FloorAlgae, extend_coral_reef};
use crate::tanks::desert::{DesertSky, extend_desert_cacti};
use crate::tanks::haunted::{HauntedBackground, extend_haunted};
use crate::tanks::hell::{HellBackground, HellPlant, extend_hell_plants};
use crate::tanks::radioactive::{RadBackground, extend_rad};
use crate::tanks::void::VoidBackground;

use super::{INITIAL_HEIGHT, INITIAL_WIDTH, TankKind};

const PLANT_SPACING_MIN: i32 = 3;
const PLANT_SPACING_MAX: i32 = 6;
const PLANT_HEIGHT_MIN: usize = 8;
const PLANT_HEIGHT_MAX: usize = 27;
const PLANT_SPAWN_LOOKAHEAD: i32 = 30;
const CORAL_SPAWN_LOOKAHEAD: i32 = 130;
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

pub enum TankBackground {
    Plain {
        plants: Vec<Plant>,
    },
    Coral {
        corals: Vec<CoralStructure>,
        floor_algae: Vec<FloorAlgae>,
    },
    Hell {
        bg: HellBackground,
        plants: Vec<HellPlant>,
    },
    Void {
        bg: VoidBackground,
    },
    Alien {
        bg: AlienBackground,
    },
    Haunted {
        bg: HauntedBackground,
    },
    Candy {
        bg: CandyBackground,
        pink_plants: Vec<Plant>,
    },
    Desert {
        bg: DesertSky,
    },
    Rad {
        bg: RadBackground,
    },
}

impl TankBackground {
    pub fn new(kind: TankKind, dead_names: &[String], rng: &mut impl RngExt) -> Self {
        match kind {
            TankKind::Base => {
                let mut plants = Vec::new();
                extend_plants(
                    &mut plants,
                    INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                    rng,
                );
                TankBackground::Plain { plants }
            }
            TankKind::CoralReef => {
                let mut corals = Vec::new();
                let mut floor_algae = Vec::new();
                extend_coral_reef(
                    &mut corals,
                    &mut floor_algae,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
                TankBackground::Coral {
                    corals,
                    floor_algae,
                }
            }
            TankKind::Hell => {
                let bg = HellBackground::new(rng);
                let mut plants = Vec::new();
                extend_hell_plants(
                    &mut plants,
                    INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                    rng,
                );
                TankBackground::Hell { bg, plants }
            }
            TankKind::Void => TankBackground::Void {
                bg: VoidBackground::new(),
            },
            TankKind::Alien => {
                let mut bg = AlienBackground::new(rng);
                let color = bg.color;
                extend_alien_tentacles(
                    &mut bg.tentacles,
                    INITIAL_WIDTH as i32 + PLANT_SPAWN_LOOKAHEAD,
                    color,
                    rng,
                );
                extend_alien_pyramids(
                    &mut bg.pyramids,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
                bg.init_stars(rng, INITIAL_WIDTH, INITIAL_HEIGHT);
                TankBackground::Alien { bg }
            }
            TankKind::Haunted => {
                let mut bg = HauntedBackground::new(rng);
                extend_haunted(
                    &mut bg.graves,
                    &mut bg.pumpkins,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    dead_names,
                    rng,
                );
                TankBackground::Haunted { bg }
            }
            TankKind::Candy => {
                let mut bg = CandyBackground::new();
                let target = INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD;
                extend_candy_plants(&mut bg.plants, target, rng);
                extend_candy_decos(&mut bg.decos, target, rng);
                let mut pink_plants = Vec::new();
                extend_candy_pink_plants(&mut pink_plants, target, rng);
                TankBackground::Candy { bg, pink_plants }
            }
            TankKind::Desert => {
                let mut bg = DesertSky::new(rng);
                extend_desert_cacti(
                    &mut bg.cacti,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
                bg.init_stars(rng, INITIAL_WIDTH, INITIAL_HEIGHT);
                TankBackground::Desert { bg }
            }
            TankKind::Rad => {
                let mut bg = RadBackground::new();
                extend_rad(
                    &mut bg.barrels,
                    &mut bg.floor,
                    INITIAL_WIDTH as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
                TankBackground::Rad { bg }
            }
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, width: u16, height: u16) {
        match self {
            TankBackground::Plain { plants } => {
                for plant in plants {
                    plant.tick(dt);
                }
            }
            TankBackground::Coral {
                corals,
                floor_algae,
            } => {
                for coral in corals {
                    coral.tick(dt, rng);
                }
                for fa in floor_algae {
                    fa.tick(dt);
                }
            }
            TankBackground::Hell { bg, plants } => {
                bg.tick(dt, rng, width, height);
                for plant in plants {
                    plant.tick(dt);
                }
            }
            TankBackground::Void { bg } => bg.tick(),
            TankBackground::Alien { bg } => bg.tick(dt, rng, width, height),
            TankBackground::Haunted { bg } => bg.tick(dt, rng, width, height),
            TankBackground::Candy { pink_plants, .. } => {
                for plant in pink_plants {
                    plant.tick(dt);
                }
            }
            TankBackground::Desert { bg } => bg.tick(dt, rng, width, height),
            TankBackground::Rad { bg } => bg.tick(dt, rng),
        }
    }

    pub fn extend(&mut self, width: u16, dead_names: &[String], rng: &mut impl RngExt) {
        match self {
            TankBackground::Plain { plants } => {
                extend_plants(plants, width as i32 + PLANT_SPAWN_LOOKAHEAD, rng);
            }
            TankBackground::Coral {
                corals,
                floor_algae,
            } => {
                extend_coral_reef(
                    corals,
                    floor_algae,
                    width as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
            }
            TankBackground::Hell { plants, .. } => {
                extend_hell_plants(plants, width as i32 + PLANT_SPAWN_LOOKAHEAD, rng);
            }
            TankBackground::Void { .. } => {}
            TankBackground::Alien { bg } => {
                let color = bg.color;
                extend_alien_tentacles(
                    &mut bg.tentacles,
                    width as i32 + PLANT_SPAWN_LOOKAHEAD,
                    color,
                    rng,
                );
                extend_alien_pyramids(&mut bg.pyramids, width as i32 + CORAL_SPAWN_LOOKAHEAD, rng);
            }
            TankBackground::Haunted { bg } => {
                extend_haunted(
                    &mut bg.graves,
                    &mut bg.pumpkins,
                    width as i32 + CORAL_SPAWN_LOOKAHEAD,
                    dead_names,
                    rng,
                );
            }
            TankBackground::Candy { bg, pink_plants } => {
                let target = width as i32 + CORAL_SPAWN_LOOKAHEAD;
                extend_candy_plants(&mut bg.plants, target, rng);
                extend_candy_decos(&mut bg.decos, target, rng);
                extend_candy_pink_plants(pink_plants, target, rng);
            }
            TankBackground::Desert { bg } => {
                extend_desert_cacti(&mut bg.cacti, width as i32 + CORAL_SPAWN_LOOKAHEAD, rng);
            }
            TankBackground::Rad { bg } => {
                extend_rad(
                    &mut bg.barrels,
                    &mut bg.floor,
                    width as i32 + CORAL_SPAWN_LOOKAHEAD,
                    rng,
                );
            }
        }
    }

    pub fn init_stars(&mut self, rng: &mut impl RngExt, width: u16, height: u16) {
        match self {
            TankBackground::Alien { bg } => bg.init_stars(rng, width, height),
            TankBackground::Desert { bg } => bg.init_stars(rng, width, height),
            _ => {}
        }
    }

    pub fn clear_grave_name(&mut self, name: &str) {
        if let TankBackground::Haunted { bg } = self {
            bg.clear_grave_name(name);
        }
    }

    pub fn bubble_color_override(&self) -> Option<Color> {
        match self {
            TankBackground::Alien { bg } => Some(bg.color.bubble_color()),
            TankBackground::Desert { bg } => Some(bg.bubble_color()),
            _ => None,
        }
    }

    pub fn is_night(&self) -> bool {
        match self {
            TankBackground::Desert { bg } => bg.is_night(),
            _ => false,
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
