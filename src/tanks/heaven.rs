use std::collections::VecDeque;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{CYAN, DARK_GRAY, LIGHT_YELLOW};
use crate::fishes::fish::{Direction, Fish};
use crate::sprite::opaque_line;
use crate::tanks::gate::{Gate, GateStyle};

pub const ANGEL_COLOR: Color = LIGHT_YELLOW;
pub const CLOUD_COLOR: Color = CYAN;
pub const SOUL_COLOR: Color = DARK_GRAY;
pub const SOUL_WALL_LIMIT: usize = 75;

const GATE_STYLE: GateStyle = GateStyle {
    bars: LIGHT_YELLOW,
    floor: LIGHT_YELLOW,
};

const ANGEL_SPAWN_MIN: f32 = 3.0;
const ANGEL_SPAWN_MAX: f32 = 7.0;
const ANGEL_RISE_MIN: f32 = 2.0;
const ANGEL_RISE_MAX: f32 = 4.5;

pub const ANGEL: [&str; 6] = [
    " _  -=-  _",
    r"( `\(_)/` )",
    r" ( /   \ )",
    r"  (\   /)",
    r"  `/   \`",
    "  '.___.'",
];

pub const CLOUDS_PER_HUNDRED_COLUMNS: f32 = 30.0;
pub const CLOUD_SKY_FRACTION: f32 = 0.6;
const CLOUD_DRIFT_MIN: f32 = 0.2;
const CLOUD_DRIFT_MAX: f32 = 0.9;

const CLOUD_SPRITES: [&[&str]; 6] = [
    &[
        "    ._",
        " .-(`  )",
        ":(      ))",
        "`(    )  ))",
        "  ` __.:'",
    ],
    &[
        "    +_",
        "  (`  ).",
        " (     ).",
        " (       '`.",
        "(      .   )",
        " (..__.:'-'",
    ],
    &["    .--", " .+(   )", " (   .  )", "(   (   ))", " `- __.'"],
    &[
        "     _",
        " .:(`  )`.",
        ":(   .    )",
        "`.  (    ) )",
        "  ` _`  ) )",
        "     (   )",
        "      `-'",
    ],
    &[" .')", "(_  )"],
    &[" ( )", "(_.'"],
];

const BIGGEST_CLOUD: usize = 3;

const SOUL_DRIFT_MIN: f32 = 0.6;
const SOUL_DRIFT_MAX: f32 = 1.8;
const SOUL_TOP_ROW: f32 = 1.0;
const SOUL_FLOOR_MARGIN: f32 = 2.0;

pub struct Angel {
    pub x: f32,
    pub y: f32,
    rise_speed: f32,
}

impl Angel {
    fn new(width: u16, height: u16, rng: &mut impl RngExt) -> Self {
        let widest = ANGEL
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        let room = (width as f32 - widest as f32).max(1.0);
        Self {
            x: rng.random_range(0.0..room),
            y: (height as f32 - 1.0).max(0.0),
            rise_speed: rng.random_range(ANGEL_RISE_MIN..ANGEL_RISE_MAX),
        }
    }

    pub fn at(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rise_speed: ANGEL_RISE_MIN,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.y -= self.rise_speed * dt;
    }

    fn is_gone(&self) -> bool {
        self.y < -(ANGEL.len() as f32)
    }

    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        ANGEL
            .iter()
            .map(|row| opaque_line(row, ANGEL_COLOR, |_, _| ANGEL_COLOR))
            .collect()
    }
}

pub struct Cloud {
    pub x: f32,
    altitude: f32,
    sprite: usize,
    drift: f32,
}

impl Cloud {
    fn new(width: u16, rng: &mut impl RngExt) -> Self {
        Self {
            x: rng.random_range(0.0..(width as f32).max(1.0)),
            altitude: rng.random_range(0.0..CLOUD_SKY_FRACTION),
            sprite: rng.random_range(0..CLOUD_SPRITES.len()),
            drift: rng.random_range(CLOUD_DRIFT_MIN..CLOUD_DRIFT_MAX),
        }
    }

    pub fn covering(x: f32, altitude: f32) -> Self {
        Self {
            x,
            altitude,
            sprite: BIGGEST_CLOUD,
            drift: 0.0,
        }
    }

    pub fn top(&self, height: u16) -> i32 {
        (self.altitude * height as f32).floor() as i32
    }

    fn width(&self) -> f32 {
        CLOUD_SPRITES[self.sprite]
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0) as f32
    }

    fn tick(&mut self, dt: f32, width: u16) {
        self.x += self.drift * dt;
        if self.x > width as f32 {
            self.x = -self.width();
        }
    }

    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        CLOUD_SPRITES[self.sprite]
            .iter()
            .map(|row| opaque_line(row, CLOUD_COLOR, |_, _| CLOUD_COLOR))
            .collect()
    }
}

pub fn cloud_count(width: u16) -> usize {
    (width as f32 * CLOUDS_PER_HUNDRED_COLUMNS / 100.0).ceil() as usize
}

pub struct Soul {
    pub fish: Fish,
    drift: f32,
}

impl Soul {
    fn new(mut fish: Fish, x: f32, height: u16, rng: &mut impl RngExt) -> Self {
        let drift = rng.random_range(SOUL_DRIFT_MIN..SOUL_DRIFT_MAX);
        fish.position.x = x;
        fish.position.y = soul_row(height, rng);
        Self { fish, drift }
    }

    fn arriving(fish: Fish, width: u16, height: u16, rng: &mut impl RngExt) -> Self {
        let room = (width as f32 - fish.display_width as f32).max(1.0);
        let x = rng.random_range(0.0..room);
        let mut soul = Self::new(fish, x, height, rng);
        soul.fish.facing = random_direction(rng);
        soul
    }

    fn returning(fish: Fish, width: u16, height: u16, rng: &mut impl RngExt) -> Self {
        let facing = random_direction(rng);
        let x = match facing {
            Direction::Right => -(fish.display_width as f32),
            Direction::Left => width as f32,
        };
        let mut soul = Self::new(fish, x, height, rng);
        soul.fish.facing = facing;
        soul
    }

    fn tick(&mut self, dt: f32, height: u16) {
        let step = self.drift * dt;
        match self.fish.facing {
            Direction::Left => self.fish.position.x -= step,
            Direction::Right => self.fish.position.x += step,
        }
        self.fish.position.y = self.fish.position.y.min(lowest_soul_row(height));
    }

    fn is_gone(&self, width: u16) -> bool {
        let x = self.fish.position.x;
        match self.fish.facing {
            Direction::Left => x + self.fish.display_width as f32 <= 0.0,
            Direction::Right => x >= width as f32,
        }
    }
}

fn random_direction(rng: &mut impl RngExt) -> Direction {
    if rng.random::<bool>() {
        Direction::Left
    } else {
        Direction::Right
    }
}

fn lowest_soul_row(height: u16) -> f32 {
    (height as f32 - SOUL_FLOOR_MARGIN).max(SOUL_TOP_ROW)
}

fn soul_row(height: u16, rng: &mut impl RngExt) -> f32 {
    let lowest = lowest_soul_row(height);
    if lowest <= SOUL_TOP_ROW {
        return SOUL_TOP_ROW;
    }
    rng.random_range(SOUL_TOP_ROW..=lowest).floor()
}

#[derive(Default)]
pub struct SoulWall {
    drifting: Vec<Soul>,
    waiting: VecDeque<Fish>,
}

impl SoulWall {
    pub fn receive(&mut self, fish: Fish, width: u16, height: u16, rng: &mut impl RngExt) {
        if self.drifting.len() >= SOUL_WALL_LIMIT {
            self.waiting.push_back(fish);
            return;
        }
        self.drifting.push(Soul::arriving(fish, width, height, rng));
    }

    pub fn release(&mut self, name: &str, width: u16, height: u16, rng: &mut impl RngExt) -> bool {
        if let Some(pos) = self.drifting.iter().position(|soul| soul.fish.name == name) {
            self.drifting.remove(pos);
            self.call_in(width, height, rng);
            return true;
        }
        let Some(pos) = self.waiting.iter().position(|fish| fish.name == name) else {
            return false;
        };
        self.waiting.remove(pos);
        true
    }

    pub fn tick(&mut self, dt: f32, width: u16, height: u16, rng: &mut impl RngExt) {
        for soul in &mut self.drifting {
            soul.tick(dt, height);
        }
        let (gone, staying): (Vec<Soul>, Vec<Soul>) = std::mem::take(&mut self.drifting)
            .into_iter()
            .partition(|soul| soul.is_gone(width));
        self.drifting = staying;
        self.waiting.extend(gone.into_iter().map(|soul| soul.fish));
        self.call_in(width, height, rng);
    }

    fn call_in(&mut self, width: u16, height: u16, rng: &mut impl RngExt) {
        while self.drifting.len() < SOUL_WALL_LIMIT {
            let Some(fish) = self.waiting.pop_front() else {
                return;
            };
            self.drifting
                .push(Soul::returning(fish, width, height, rng));
        }
    }

    pub fn drifting(&self) -> impl Iterator<Item = &Fish> {
        self.drifting.iter().map(|soul| &soul.fish)
    }

    pub fn drifting_mut(&mut self) -> impl Iterator<Item = &mut Fish> {
        self.drifting.iter_mut().map(|soul| &mut soul.fish)
    }

    pub fn all(&self) -> impl Iterator<Item = &Fish> {
        self.drifting().chain(self.waiting.iter())
    }

    pub fn len(&self) -> usize {
        self.drifting.len() + self.waiting.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct HeavenBackground {
    pub gate: Gate,
    pub clouds: Vec<Cloud>,
    pub angels: Vec<Angel>,
    pub souls: SoulWall,
    angel_timer: f32,
}

impl HeavenBackground {
    pub fn new(width: u16, rng: &mut impl RngExt) -> Self {
        let mut bg = Self {
            gate: Gate::whole(GATE_STYLE, rng),
            clouds: Vec::new(),
            angels: Vec::new(),
            souls: SoulWall::default(),
            angel_timer: rng.random_range(ANGEL_SPAWN_MIN..ANGEL_SPAWN_MAX),
        };
        bg.extend(width, rng);
        bg
    }

    pub fn extend(&mut self, width: u16, rng: &mut impl RngExt) {
        while self.clouds.len() < cloud_count(width) {
            self.clouds.push(Cloud::new(width, rng));
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, width: u16, height: u16) {
        if width == 0 || height == 0 {
            return;
        }
        for cloud in &mut self.clouds {
            cloud.tick(dt, width);
        }
        for angel in &mut self.angels {
            angel.tick(dt);
        }
        self.angels.retain(|angel| !angel.is_gone());
        self.angel_timer -= dt;
        if self.angel_timer <= 0.0 {
            self.angel_timer = rng.random_range(ANGEL_SPAWN_MIN..ANGEL_SPAWN_MAX);
            self.angels.push(Angel::new(width, height, rng));
        }
        self.souls.tick(dt, width, height, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::species::FishSpecies;
    use crate::sprite::TRANSPARENT;

    const WIDTH: u16 = 80;
    const HEIGHT: u16 = 24;

    fn dead(name: &str) -> Fish {
        Fish::new(
            FishSpecies::Salmon,
            name.to_string(),
            0.0,
            0.0,
            &mut rand::rng(),
        )
    }

    #[test]
    fn an_angel_rises_straight_up() {
        let mut rng = rand::rng();
        let mut angel = Angel::new(WIDTH, HEIGHT, &mut rng);
        let x = angel.x;
        let y = angel.y;
        angel.tick(1.0);
        assert_eq!(angel.x, x, "an angel never sways");
        assert!(angel.y < y, "an angel rises");
    }

    #[test]
    fn the_sky_is_as_cloudy_as_its_density_says_and_every_cloud_floats_in_the_upper_sky() {
        let bg = HeavenBackground::new(WIDTH, &mut rand::rng());
        assert_eq!(bg.clouds.len(), cloud_count(WIDTH));
        let highest_bottom = (HEIGHT as f32 * CLOUD_SKY_FRACTION).ceil() as i32;
        assert!(
            bg.clouds
                .iter()
                .all(|cloud| cloud.top(HEIGHT) < highest_bottom)
        );
    }

    #[test]
    fn an_angel_is_solid_from_wing_to_wing() {
        let rows = Angel::at(0.0, 0.0).rows();
        let wings = &rows[2];
        let first = wings.iter().position(|&(ch, _)| ch != TRANSPARENT).unwrap();
        let last = wings
            .iter()
            .rposition(|&(ch, _)| ch != TRANSPARENT)
            .unwrap();
        assert!(wings[first..=last].iter().all(|&(ch, _)| ch != TRANSPARENT));
        assert!(
            rows.iter()
                .flatten()
                .all(|&(_, color)| color == ANGEL_COLOR)
        );
    }

    #[test]
    fn the_wall_shows_at_most_its_limit_and_the_rest_wait_their_turn() {
        let mut rng = rand::rng();
        let mut wall = SoulWall::default();
        let total = SOUL_WALL_LIMIT + 5;
        for n in 0..total {
            wall.receive(dead(&format!("Soul{n}")), WIDTH, HEIGHT, &mut rng);
        }
        assert_eq!(wall.drifting().count(), SOUL_WALL_LIMIT);
        assert_eq!(wall.len(), total);
    }

    #[test]
    fn a_soul_that_drifts_off_the_wall_lets_a_waiting_one_in() {
        let mut rng = rand::rng();
        let mut wall = SoulWall::default();
        for n in 0..=SOUL_WALL_LIMIT {
            wall.receive(dead(&format!("Soul{n}")), WIDTH, HEIGHT, &mut rng);
        }
        let waiting = format!("Soul{SOUL_WALL_LIMIT}");
        assert!(wall.drifting().all(|fish| fish.name != waiting));
        wall.drifting[0].fish.position.x = WIDTH as f32 * 2.0;
        wall.drifting[0].fish.facing = Direction::Right;
        wall.tick(0.0, WIDTH, HEIGHT, &mut rng);
        assert!(wall.drifting().any(|fish| fish.name == waiting));
        assert_eq!(wall.drifting().count(), SOUL_WALL_LIMIT);
        assert_eq!(wall.len(), SOUL_WALL_LIMIT + 1);
    }

    #[test]
    fn a_lone_soul_that_drifts_off_comes_back() {
        let mut rng = rand::rng();
        let mut wall = SoulWall::default();
        wall.receive(dead("Ann"), WIDTH, HEIGHT, &mut rng);
        wall.drifting[0].fish.position.x = -100.0;
        wall.drifting[0].fish.facing = Direction::Left;
        wall.tick(0.0, WIDTH, HEIGHT, &mut rng);
        assert_eq!(wall.drifting().count(), 1);
        let fish = wall.drifting().next().unwrap();
        assert!(!wall.drifting[0].is_gone(WIDTH), "{}", fish.position.x);
    }

    #[test]
    fn a_released_soul_leaves_the_wall_wherever_it_was() {
        let mut rng = rand::rng();
        let mut wall = SoulWall::default();
        for n in 0..=SOUL_WALL_LIMIT {
            wall.receive(dead(&format!("Soul{n}")), WIDTH, HEIGHT, &mut rng);
        }
        assert!(wall.release("Soul0", WIDTH, HEIGHT, &mut rng));
        let called_in = format!("Soul{SOUL_WALL_LIMIT}");
        assert!(wall.drifting().any(|fish| fish.name == called_in));
        assert!(wall.release(&called_in, WIDTH, HEIGHT, &mut rng));
        assert_eq!(wall.len(), SOUL_WALL_LIMIT - 1);
        assert!(!wall.all().any(|fish| fish.name == "Soul0"));
        assert!(!wall.release("Nobody", WIDTH, HEIGHT, &mut rng));
    }
}
