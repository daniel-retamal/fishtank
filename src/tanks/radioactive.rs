use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{LIGHT_GREEN, LIGHT_YELLOW};
use crate::sprite::{TRANSPARENT, mirror_grid};

pub const FLUID_COLOR: Color = LIGHT_GREEN;
const BARREL_COLOR: Color = LIGHT_YELLOW;

const FLUID_CHARS: &[char] = &['^', '.', '`', ';', '=', '~', '-', ',', '\''];
const FLUID_SWAP_MIN: f32 = 0.3;
const FLUID_SWAP_MAX: f32 = 2.2;

const BARREL_GAP_MIN: i32 = 5;
const BARREL_GAP_MAX: i32 = 16;
const BARREL_START_MIN: i32 = 2;
const BARREL_START_MAX: i32 = 10;

const SPURT_INTERVAL_MEAN: f32 = 15.0;
const SPURT_LIFESPAN_MIN: f32 = 6.0;
const SPURT_LIFESPAN_MAX: f32 = 11.0;
const SPURT_BURST_FRACTION: f32 = 0.33;
const SPURT_SETTLE_FRACTION: f32 = 0.75;
const SPURT_BURST_RATE: f32 = 42.0;
const SPURT_FLOW_RATE: f32 = 13.0;
const SPURT_GRAVITY: f32 = 18.0;
const SPURT_FALL_SPEED_MIN: f32 = 1.0;
const SPURT_FALL_SPEED_MAX: f32 = 3.0;
const SPURT_SPREAD_SPEED: f32 = 5.0;
const SPURT_SPREAD_MIN_SCALE: f32 = 0.3;
const SPURT_ORIGIN_ROW_MARGIN: i32 = 1;

const CAGED_ART: &[&str] = &[
    "  ___________",
    r#" /=//==//=/  \"#,
    "|=||==||=|    |",
    "|=||==||=|~-, |",
    "|=||==||=|^.`;|",
    r#" \=\\==\\=\`=.:"#,
];
const CAGED_MASK: &[&str] = &[
    "  ___________",
    r#" /=//==//=/  \"#,
    "|=||==||=|    |",
    "|=||==||=|    |",
    "|=||==||=|    |",
    r#" \=\\==\\=\"#,
];

const ROUND_ART: &[&str] = &[
    ",.--'```'--.,",
    "|'-.,___,.-'|",
    "|           |",
    "|'-.,___,.-'|",
    "|           |",
    "|'-.,___,.-'|",
    "`.         .`",
];

const TALL_ART: &[&str] = &[
    ",.--'````'--.,",
    "|'-.,____,.-'|",
    "|            |",
    "|            |",
    "|'-.,____,.-'|`'-.,",
    "|            |_,-'|",
    "|'-.,____,.-'|    |",
    "|            |    |",
    "`.          .`   .`",
];

#[derive(Clone, Copy)]
enum BarrelVariant {
    Caged,
    Round,
    Tall,
}

impl BarrelVariant {
    fn cycle(index: usize) -> Self {
        match index % 3 {
            0 => BarrelVariant::Caged,
            1 => BarrelVariant::Round,
            _ => BarrelVariant::Tall,
        }
    }

    fn art(self) -> &'static [&'static str] {
        match self {
            BarrelVariant::Caged => CAGED_ART,
            BarrelVariant::Round => ROUND_ART,
            BarrelVariant::Tall => TALL_ART,
        }
    }

    fn mask(self) -> Option<&'static [&'static str]> {
        match self {
            BarrelVariant::Caged => Some(CAGED_MASK),
            BarrelVariant::Round | BarrelVariant::Tall => None,
        }
    }

    fn leaks(self) -> bool {
        matches!(self, BarrelVariant::Round | BarrelVariant::Tall)
    }
}

fn random_fluid_char(rng: &mut impl RngExt) -> char {
    FLUID_CHARS[rng.random_range(0..FLUID_CHARS.len())]
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn next_spurt_delay(rng: &mut impl RngExt) -> f32 {
    -SPURT_INTERVAL_MEAN * (1.0 - rng.random::<f32>()).ln()
}

pub struct FluidChar {
    pub ch: char,
    timer: f32,
}

impl FluidChar {
    fn new(rng: &mut impl RngExt) -> Self {
        Self {
            ch: random_fluid_char(rng),
            timer: rng.random_range(FLUID_SWAP_MIN..FLUID_SWAP_MAX),
        }
    }

    fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.ch = random_fluid_char(rng);
            self.timer = rng.random_range(FLUID_SWAP_MIN..FLUID_SWAP_MAX);
        }
    }
}

#[derive(Clone, Copy)]
enum SpurtSide {
    Left,
    Right,
}

impl SpurtSide {
    fn random(rng: &mut impl RngExt) -> Self {
        if rng.random::<bool>() {
            SpurtSide::Left
        } else {
            SpurtSide::Right
        }
    }

    fn outward(self) -> f32 {
        match self {
            SpurtSide::Left => -1.0,
            SpurtSide::Right => 1.0,
        }
    }

    fn column(self, width: i32) -> i32 {
        match self {
            SpurtSide::Left => -1,
            SpurtSide::Right => width,
        }
    }
}

struct Droplet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ch: char,
}

impl Droplet {
    fn spawn(side: SpurtSide, burst: f32, rng: &mut impl RngExt) -> Self {
        let spread = side.outward()
            * burst
            * SPURT_SPREAD_SPEED
            * rng.random_range(SPURT_SPREAD_MIN_SCALE..=1.0);
        Self {
            x: 0.0,
            y: 0.0,
            vx: spread,
            vy: rng.random_range(SPURT_FALL_SPEED_MIN..=SPURT_FALL_SPEED_MAX),
            ch: random_fluid_char(rng),
        }
    }

    fn advance(&mut self, dt: f32) {
        self.vy += SPURT_GRAVITY * dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

pub struct SpurtCell {
    pub col: i32,
    pub row: i32,
    pub ch: char,
}

struct Spurt {
    side: SpurtSide,
    origin_row: i32,
    fall_limit: f32,
    age: f32,
    lifespan: f32,
    emit_accumulator: f32,
    droplets: Vec<Droplet>,
}

impl Spurt {
    fn new(barrel_height: i32, rng: &mut impl RngExt) -> Self {
        let origin_row =
            rng.random_range(SPURT_ORIGIN_ROW_MARGIN..=barrel_height - 1 - SPURT_ORIGIN_ROW_MARGIN);
        Self {
            side: SpurtSide::random(rng),
            origin_row,
            fall_limit: (barrel_height - origin_row) as f32,
            age: 0.0,
            lifespan: rng.random_range(SPURT_LIFESPAN_MIN..=SPURT_LIFESPAN_MAX),
            emit_accumulator: 0.0,
            droplets: Vec::new(),
        }
    }

    fn progress(&self) -> f32 {
        (self.age / self.lifespan).min(1.0)
    }

    fn burst_intensity(&self) -> f32 {
        ((SPURT_BURST_FRACTION - self.progress()) / SPURT_BURST_FRACTION).clamp(0.0, 1.0)
    }

    fn emission_rate(&self) -> f32 {
        let t = self.progress();
        if t < SPURT_BURST_FRACTION {
            return lerp(SPURT_BURST_RATE, SPURT_FLOW_RATE, t / SPURT_BURST_FRACTION);
        }
        if t < SPURT_SETTLE_FRACTION {
            return SPURT_FLOW_RATE;
        }
        let tail = (t - SPURT_SETTLE_FRACTION) / (1.0 - SPURT_SETTLE_FRACTION);
        lerp(SPURT_FLOW_RATE, 0.0, tail)
    }

    fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.age += dt;
        let limit = self.fall_limit;
        self.droplets.retain_mut(|d| {
            d.advance(dt);
            d.y < limit
        });
        self.emit_accumulator += self.emission_rate() * dt;
        while self.emit_accumulator >= 1.0 {
            self.emit_accumulator -= 1.0;
            let burst = self.burst_intensity();
            self.droplets.push(Droplet::spawn(self.side, burst, rng));
        }
    }

    fn is_finished(&self) -> bool {
        self.progress() >= 1.0 && self.droplets.is_empty()
    }

    fn cells(&self, width: i32) -> impl Iterator<Item = SpurtCell> + '_ {
        let base_col = self.side.column(width);
        self.droplets.iter().map(move |d| SpurtCell {
            col: base_col + d.x.round() as i32,
            row: self.origin_row + d.y.round() as i32,
            ch: d.ch,
        })
    }
}

enum BarrelCell {
    Empty,
    Solid(char),
    Fluid(FluidChar),
}

pub struct RadBarrel {
    pub x: i32,
    width: i32,
    mirrored: bool,
    leaks: bool,
    grid: Vec<Vec<BarrelCell>>,
    spurts: Vec<Spurt>,
    spurt_timer: f32,
}

impl RadBarrel {
    fn new(variant: BarrelVariant, x: i32, mirrored: bool, rng: &mut impl RngExt) -> Self {
        let art = variant.art();
        let mask = variant.mask();
        let width = art
            .iter()
            .map(|l| l.chars().count() as i32)
            .max()
            .unwrap_or(0);
        let grid = art
            .iter()
            .enumerate()
            .map(|(row, line)| {
                line.chars()
                    .enumerate()
                    .map(|(col, ch)| {
                        if ch == ' ' {
                            BarrelCell::Empty
                        } else if is_fluid_cell(mask, row, col) {
                            BarrelCell::Fluid(FluidChar::new(rng))
                        } else {
                            BarrelCell::Solid(ch)
                        }
                    })
                    .collect()
            })
            .collect();
        Self {
            x,
            width,
            mirrored,
            leaks: variant.leaks(),
            grid,
            spurts: Vec::new(),
            spurt_timer: next_spurt_delay(rng),
        }
    }

    fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.grid.len() as i32
    }

    fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        for row in &mut self.grid {
            for cell in row {
                if let BarrelCell::Fluid(fluid) = cell {
                    fluid.tick(dt, rng);
                }
            }
        }
        if self.leaks {
            self.tick_spurts(dt, rng);
        }
    }

    fn tick_spurts(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.spurt_timer -= dt;
        if self.spurt_timer <= 0.0 {
            self.spurts.push(Spurt::new(self.height(), rng));
            self.spurt_timer = next_spurt_delay(rng);
        }
        for spurt in &mut self.spurts {
            spurt.tick(dt, rng);
        }
        self.spurts.retain(|spurt| !spurt.is_finished());
    }

    pub fn spurt_cells(&self) -> impl Iterator<Item = SpurtCell> + '_ {
        let width = self.width;
        self.spurts.iter().flat_map(move |spurt| spurt.cells(width))
    }

    pub fn rows(&self) -> Vec<Vec<(char, Color)>> {
        let grid: Vec<Vec<(char, Color)>> = self
            .grid
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| match cell {
                        BarrelCell::Empty => (TRANSPARENT, BARREL_COLOR),
                        BarrelCell::Solid(ch) => (*ch, BARREL_COLOR),
                        BarrelCell::Fluid(fluid) => (fluid.ch, FLUID_COLOR),
                    })
                    .collect()
            })
            .collect();
        if self.mirrored {
            mirror_grid(&grid)
        } else {
            grid
        }
    }
}

fn is_fluid_cell(mask: Option<&[&str]>, row: usize, col: usize) -> bool {
    let Some(mask) = mask else {
        return false;
    };
    mask.get(row)
        .and_then(|line| line.chars().nth(col))
        .is_none_or(|c| c == ' ')
}

pub struct RadBackground {
    pub floor: Vec<FluidChar>,
    pub barrels: Vec<RadBarrel>,
}

impl Default for RadBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl RadBackground {
    pub fn new() -> Self {
        Self {
            floor: Vec::new(),
            barrels: Vec::new(),
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt) {
        for cell in &mut self.floor {
            cell.tick(dt, rng);
        }
        for barrel in &mut self.barrels {
            barrel.tick(dt, rng);
        }
    }
}

pub fn extend_rad(
    barrels: &mut Vec<RadBarrel>,
    floor: &mut Vec<FluidChar>,
    to_width: i32,
    rng: &mut impl RngExt,
) {
    while (floor.len() as i32) < to_width {
        floor.push(FluidChar::new(rng));
    }
    let frontier = barrels.iter().map(|b| b.x + b.width()).max();
    let mut next_x = match frontier {
        Some(edge) => edge + rng.random_range(BARREL_GAP_MIN..=BARREL_GAP_MAX),
        None => rng.random_range(BARREL_START_MIN..=BARREL_START_MAX),
    };
    while next_x < to_width {
        let variant = BarrelVariant::cycle(barrels.len());
        let mirrored = rng.random::<bool>();
        let barrel = RadBarrel::new(variant, next_x, mirrored, rng);
        let width = barrel.width();
        barrels.push(barrel);
        next_x += width + rng.random_range(BARREL_GAP_MIN..=BARREL_GAP_MAX);
    }
}
