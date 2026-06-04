use std::f32::consts::{FRAC_PI_2, PI, TAU};

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    AMBER, AMBER_LIGHT, BLUE, BROWN, GOLD, GREEN, GREEN_DARK, LIGHT_GREEN, LIGHT_RED, ORANGE,
    ORANGE_LIGHT, RED, YELLOW,
};
use crate::entities::star::{StarTwinkle, star_field_target};
use crate::tanks::coral::mirror_char;

const TRANSPARENT: char = '\0';

pub const CYCLE_SECS: f32 = 7.5 * 60.0;
const SKY_SPEED: f32 = TAU / CYCLE_SECS;

pub const SUN_ORBIT_RY: f32 = 24.0;
pub const SUN_H_WAVE_AMPLITUDE: f32 = 3.0;
pub const SUN_H_WAVE_SPREAD: f32 = 0.3;
const SUN_H_WAVE_SPEED: f32 = 0.8;

const RING_RIPPLE_SECS: f32 = 0.18;
const SUN_X_SCALE: f32 = 0.5;
const SKULL_RADIUS: f32 = 5.0;
const RING_THICKNESS: f32 = 1.5;

pub const SUN_RING_SKULL: i32 = -1;
pub const SUN_RING_EYE: i32 = -2;
const SUN_RING_EMPTY: i32 = -3;

const SUN_EYE_CHAR: char = '@';
const SUN_SKULL_COLOR: Color = RED;
const SUN_EYE_COLOR: Color = YELLOW;

const DAY_BUBBLE_COLOR: Color = YELLOW;
const NIGHT_BUBBLE_COLOR: Color = BLUE;

const SUN_PALETTE: &[Color] = &[
    ORANGE,
    ORANGE_LIGHT,
    AMBER,
    AMBER_LIGHT,
    GOLD,
    YELLOW,
    RED,
    LIGHT_RED,
];

const STAR_RADIUS_MIN_FRAC: f32 = 0.3;
const STAR_RADIUS_MAX_FRAC: f32 = 1.0;

const STALK_CHARS: &[char] = &['/', '\\', '|', '_'];
const STALK_SHADES: &[Color] = &[GREEN, LIGHT_GREEN];

const TUMBLEWEED_ROWS: usize = 4;
const TUMBLEWEED_FRAME_COUNT: usize = 4;
const TUMBLEWEED_SPAWN_MIN_SECS: f32 = 10.0;
const TUMBLEWEED_SPAWN_MAX_SECS: f32 = 30.0;
const TUMBLEWEED_SPEED: f32 = 22.0;
const TUMBLEWEED_BOUNCE_SPEED: f32 = 5.7;
const TUMBLEWEED_BOUNCE_HEIGHT: f32 = 2.5;
const TUMBLEWEED_FRAME_SECS: f32 = 0.1;
const TUMBLEWEED_COLOR: Color = BROWN;

const TUMBLEWEED_FRAMES: [[&str; TUMBLEWEED_ROWS]; TUMBLEWEED_FRAME_COUNT] = [
    [r#"\ `. .'//"#, r#"\`.-|-.|"#, r#" |-||'-/"#, r#" \/\`-/"#],
    [
        r#"  / _/..'"#,
        r#"\/_--.'.-"#,
        r#"/\.-'`--."#,
        r#" .\'_/--_"#,
    ],
    [r#"   /\\|'"#, r#"  |`.'/|"#, r#" \.`.|./."#, r#"  \ /`/.\"#],
    [
        r#"   -\.\-\"#,
        r#" -.`.-'`.\"#,
        r#"-..__`-.//-"#,
        r#" .-'-`/''"#,
    ],
];

const CACTUS_HEIGHT_MAX: usize = 20;
const CACTUS_GAP_MIN: i32 = 4;
const CACTUS_GAP_MAX: i32 = 10;
const CACTUS_LEFT_MIN: i32 = 1;
const CACTUS_LEFT_MAX: i32 = 12;
const CACTUS_VARIANT_COUNT: u8 = 5;

pub const SUN_LINES: &[&str] = &[
    r"                           _.------------.",
    r"                     ,---''               `----.",
    r"                  ,-'                           `-.",
    r"               ,-'         _.------------.         `-.",
    r"            ,-'       ,--''               `---.       `-.",
    r"           /       ,-'                         `-.       \",
    r"         ,'     ,-'         _.----------.         `-.     `.",
    r"       ,'     ,'       ,--''             `---.       `.     `.",
    r"      /      /      ,-'      _.--------.      `-.      \      \",
    r"     /     ,'     ,'     ,-''           `--.     `.     `.     \",
    r"    /     /     ,'    ,-'                   `-.    `.     \     \",
    r"   /     /     /     /        _.------.        \     \     \     \",
    r"  ;     /     /    ,'     ,-''     |   `--.     `.    \     \     :",
    r"  ;    /     /    /     ,'  \            / `.     \    \     \    :",
    r" ;    ;     /    /     /       ,------.      \     \    \     :    :",
    r" ;    ;    /    ;     /  -.   |        |  .-  \     :    \    :    :",
    r";    ;    ;     ;    /       |`        `|      \    :     :    :    :",
    r"|    |    |    ;    ;        |  _    _  |       :    :    |    |    |",
    r"|    |    |    |    | --=    |  @    @  |   =-- |    |    |    |    |",
    r"|    |    |    :    :        |:   '`   :|       ;    ;    |    |    |",
    r":    :    :     :    \        \:      :/       /    ;     ;    ;    ;",
    r" :    :    \    :     \  -'    \`++++'/   '-  /     ;    /    ;    ;",
    r" :    :     \    \     \        `-__-`       /     /    /     ;    ;",
    r"  :    \     \    \     `.  /            \ ,'     /    /     /    ;",
    r"  :     \     \    `.     '--.    |    _.-'     ,'    /     /     ;",
    r"   \     \     \     \        `------''        /     /     /     /",
    r"    \     \     `.    `-.                   ,-'    ,'     /     /",
    r"     \     `.     `.     `--.           _.-'     ,'     ,'     /",
    r"      \      \      `-.      `--------''      ,-'      /      /",
    r"       `.     `.       `---.             _.--'       ,'     ,'",
    r"         `.     '-.         `----------''         ,-'     ,'",
    r"           \       `-.                         ,-'       /",
    r"            `-.       `---.               _.--'       ,-'",
    r"               `-.         `------------''         ,-'",
    r"                  `-.                           ,-'",
    r"                     `----.               _.---'",
    r"                           `------------''",
];

const CACTUS_A: &[&str] = &[
    r"        _    _",
    r"       | |  |.|",
    r"      -| |  | |-",
    r"  _    |.|- | |",
    r"-| |   | |  |.|-",
    r" |.|  -| ||/  |",
    r" | |-  |  ___/",
    r"-|.|   | | |",
    r" |  \_|| |",
    r"  \____  |",
    r"   |   | |-",
    r"       |.|",
    r"      -| |",
    r"       |_|",
];
const CACTUS_A_TRUNK: &str = r"       | |";

const CACTUS_B: &[&str] = &[
    r"         _",
    r"  _     | |-",
    r"-| |    |.|",
    r" |.|   -| |",
    r" | |-   | |",
    r"-|.|    | |-",
    r" |  \__|| |",
    r"  \_____  |",
    r"   |    |.|-",
    r"        | |",
    r"       -| |",
    r"        |.|",
    r"        | |",
    r"        | |",
    r"        |.|",
    r"        |_|",
];
const CACTUS_B_TRUNK: &str = r"        | |";

const CACTUS_C: &[&str] = &[
    r"          _",
    r"  _      | |-",
    r"-| |     |.|",
    r" | |-   -| |",
    r" |.|     | |     _",
    r" | |     | |-  -| |",
    r"-| |     | |    | |",
    r" |.|-    | |    | |-",
    r" | |     |.|-  -| |",
    r" | |     | |    |.|",
    r"-|.|    -| |    | |-",
    r" |  \___||.||__/  |",
    r"  \______| |_____/",
    r"   |     | |   |",
    r"        -|.|",
    r"         |_|",
];
const CACTUS_C_TRUNK: &str = r"         | |";

const CACTUS_D: &[&str] = &[
    r"     ,-,,",
    r" ,-\/|`| \",
    r" \'  | |'| -,",
    r"  \ `| | |/ )",
    r"   | |'| , /",
    r"   |'| |, /",
    r"   |_|_|_|",
];

const CACTUS_E: &[&str] = &[
    r"     _,__.__,",
    r"   /`- - | , \-",
    r"  `  ` | |-|  ,",
    r"  | `| . | |, .-",
    r" -|  ` | - ,  |",
    r"  -\_-_|_-_|_/",
];

fn mirror_cactus_char(ch: char) -> char {
    match ch {
        '`' => '\'',
        '\'' => '`',
        other => mirror_char(other),
    }
}

#[derive(Clone, Copy)]
enum CactusVariant {
    A,
    B,
    C,
    D,
    E,
}

impl CactusVariant {
    fn random(rng: &mut impl RngExt) -> Self {
        match rng.random_range(0..CACTUS_VARIANT_COUNT) {
            0 => CactusVariant::A,
            1 => CactusVariant::B,
            2 => CactusVariant::C,
            3 => CactusVariant::D,
            _ => CactusVariant::E,
        }
    }

    fn art(self) -> &'static [&'static str] {
        match self {
            CactusVariant::A => CACTUS_A,
            CactusVariant::B => CACTUS_B,
            CactusVariant::C => CACTUS_C,
            CactusVariant::D => CACTUS_D,
            CactusVariant::E => CACTUS_E,
        }
    }

    fn trunk_row(self) -> Option<&'static str> {
        match self {
            CactusVariant::A => Some(CACTUS_A_TRUNK),
            CactusVariant::B => Some(CACTUS_B_TRUNK),
            CactusVariant::C => Some(CACTUS_C_TRUNK),
            CactusVariant::D | CactusVariant::E => None,
        }
    }
}

pub struct Cactus {
    pub x: i32,
    pub width: i32,
    pub grid: Vec<Vec<(char, Color)>>,
}

impl Cactus {
    fn new(x: i32, rng: &mut impl RngExt) -> Self {
        let variant = CactusVariant::random(rng);
        let art = variant.art();
        let mut lines: Vec<String> = art.iter().map(|l| l.to_string()).collect();
        if let Some(trunk) = variant.trunk_row() {
            let target = rng.random_range(lines.len()..=CACTUS_HEIGHT_MAX);
            let extra = target.saturating_sub(lines.len());
            let base_idx = lines.len() - 1;
            for _ in 0..extra {
                lines.insert(base_idx, trunk.to_string());
            }
        }
        let max_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let grid = build_cactus_grid(&lines, max_w, rng.random::<bool>(), rng);
        Self {
            x,
            width: max_w as i32,
            grid,
        }
    }
}

fn build_cactus_grid(
    lines: &[String],
    width: usize,
    mirrored: bool,
    rng: &mut impl RngExt,
) -> Vec<Vec<(char, Color)>> {
    let height = lines.len();
    let mut grid: Vec<Vec<(char, Color)>> = lines
        .iter()
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let mut row: Vec<(char, Color)> = chars
                .iter()
                .map(|&ch| {
                    if ch == ' ' {
                        (TRANSPARENT, Color::Reset)
                    } else if STALK_CHARS.contains(&ch) {
                        (ch, STALK_SHADES[rng.random_range(0..STALK_SHADES.len())])
                    } else {
                        (ch, GREEN_DARK)
                    }
                })
                .collect();
            while row.len() < width {
                row.push((TRANSPARENT, Color::Reset));
            }
            row
        })
        .collect();
    flood_fill_interior(&mut grid, width, height);
    if mirrored {
        for row in &mut grid {
            row.reverse();
            for cell in row.iter_mut() {
                cell.0 = mirror_cactus_char(cell.0);
            }
        }
    }
    grid
}

fn flood_fill_interior(grid: &mut [Vec<(char, Color)>], width: usize, height: usize) {
    use std::collections::VecDeque;
    let mut exterior = vec![vec![false; width]; height];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    for r in 0..height {
        for c in 0..width {
            if (r == 0 || r + 1 == height || c == 0 || c + 1 == width)
                && grid[r][c].0 == TRANSPARENT
                && !exterior[r][c]
            {
                exterior[r][c] = true;
                queue.push_back((r, c));
            }
        }
    }
    while let Some((r, c)) = queue.pop_front() {
        for (nr, nc) in [
            r.checked_sub(1).map(|x| (x, c)),
            (r + 1 < height).then_some((r + 1, c)),
            c.checked_sub(1).map(|x| (r, x)),
            (c + 1 < width).then_some((r, c + 1)),
        ]
        .into_iter()
        .flatten()
        {
            if !exterior[nr][nc] && grid[nr][nc].0 == TRANSPARENT {
                exterior[nr][nc] = true;
                queue.push_back((nr, nc));
            }
        }
    }
    for r in 0..height {
        for c in 0..width {
            if grid[r][c].0 == TRANSPARENT && !exterior[r][c] {
                grid[r][c].0 = ' ';
            }
        }
    }
}

pub fn extend_desert_cacti(cacti: &mut Vec<Cactus>, to_width: i32, rng: &mut impl RngExt) {
    let frontier = cacti.iter().map(|c| c.x + c.width).max();
    let mut next_x = match frontier {
        Some(edge) => edge + rng.random_range(CACTUS_GAP_MIN..=CACTUS_GAP_MAX),
        None => rng.random_range(CACTUS_LEFT_MIN..=CACTUS_LEFT_MAX),
    };
    while next_x < to_width {
        let cactus = Cactus::new(next_x, rng);
        next_x = cactus.x + cactus.width + rng.random_range(CACTUS_GAP_MIN..=CACTUS_GAP_MAX);
        cacti.push(cactus);
    }
}

pub struct Tumbleweed {
    pub x: f32,
    bounce_phase: f32,
    frame: usize,
    frame_timer: f32,
}

impl Tumbleweed {
    fn new() -> Self {
        let width = TUMBLEWEED_FRAMES
            .iter()
            .flatten()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as f32;
        Self {
            x: -width,
            bounce_phase: 0.0,
            frame: 0,
            frame_timer: TUMBLEWEED_FRAME_SECS,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.x += TUMBLEWEED_SPEED * dt;
        self.bounce_phase = (self.bounce_phase + TUMBLEWEED_BOUNCE_SPEED * dt).rem_euclid(TAU);
        self.frame_timer -= dt;
        if self.frame_timer <= 0.0 {
            self.frame_timer += TUMBLEWEED_FRAME_SECS;
            self.frame = (self.frame + 1) % TUMBLEWEED_FRAME_COUNT;
        }
    }

    pub fn hop_height(&self) -> i32 {
        (self.bounce_phase.sin().abs() * TUMBLEWEED_BOUNCE_HEIGHT).round() as i32
    }

    pub fn grid(&self) -> Vec<Vec<(char, Color)>> {
        let frame = &TUMBLEWEED_FRAMES[self.frame];
        let width = frame.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        frame
            .iter()
            .map(|line| {
                let mut row: Vec<(char, Color)> = line
                    .chars()
                    .map(|ch| {
                        if ch == ' ' {
                            (TRANSPARENT, Color::Reset)
                        } else {
                            (ch, TUMBLEWEED_COLOR)
                        }
                    })
                    .collect();
                while row.len() < width {
                    row.push((TRANSPARENT, Color::Reset));
                }
                row
            })
            .collect()
    }
}

pub struct DesertStar {
    pub angle: f32,
    pub radius_frac: f32,
    pub twinkle: StarTwinkle,
}

impl DesertStar {
    fn new(rng: &mut impl RngExt) -> Self {
        Self {
            angle: rng.random_range(-PI..PI),
            radius_frac: rng.random_range(STAR_RADIUS_MIN_FRAC..STAR_RADIUS_MAX_FRAC),
            twinkle: StarTwinkle::new(rng),
        }
    }
}

fn random_palette(rng: &mut impl RngExt) -> Color {
    SUN_PALETTE[rng.random_range(0..SUN_PALETTE.len())]
}

fn build_sun_rings() -> Vec<Vec<i32>> {
    let n_rows = SUN_LINES.len();
    let max_w = SUN_LINES
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let center_y = (n_rows as f32 - 1.0) / 2.0;
    let center_x = (max_w as f32 - 1.0) / 2.0;
    SUN_LINES
        .iter()
        .enumerate()
        .map(|(row, line)| {
            line.chars()
                .enumerate()
                .map(|(col, ch)| {
                    if ch == ' ' {
                        return SUN_RING_EMPTY;
                    }
                    if ch == SUN_EYE_CHAR {
                        return SUN_RING_EYE;
                    }
                    let dx = (col as f32 - center_x) * SUN_X_SCALE;
                    let dy = row as f32 - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < SKULL_RADIUS {
                        SUN_RING_SKULL
                    } else {
                        ((dist - SKULL_RADIUS) / RING_THICKNESS) as i32
                    }
                })
                .collect()
        })
        .collect()
}

pub fn sun_cell_color(ring: i32, ring_colors: &[Color]) -> Color {
    match ring {
        SUN_RING_EYE => SUN_EYE_COLOR,
        SUN_RING_SKULL => SUN_SKULL_COLOR,
        i if i >= 0 => {
            let idx = (i as usize).min(ring_colors.len().saturating_sub(1));
            ring_colors.get(idx).copied().unwrap_or(SUN_SKULL_COLOR)
        }
        _ => SUN_SKULL_COLOR,
    }
}

pub struct DesertSky {
    pub sky_angle: f32,
    pub h_phase: f32,
    pub ring_colors: Vec<Color>,
    pub sun_rings: Vec<Vec<i32>>,
    pub stars: Vec<DesertStar>,
    pub cacti: Vec<Cactus>,
    pub tumbleweed: Option<Tumbleweed>,
    tumbleweed_timer: f32,
    ripple_timer: f32,
}

impl DesertSky {
    pub fn new(rng: &mut impl RngExt) -> Self {
        let sun_rings = build_sun_rings();
        let ring_count = sun_rings
            .iter()
            .flatten()
            .copied()
            .filter(|&r| r >= 0)
            .max()
            .map(|m| m as usize + 1)
            .unwrap_or(1);
        let ring_colors = (0..ring_count).map(|_| random_palette(rng)).collect();
        Self {
            sky_angle: FRAC_PI_2,
            h_phase: rng.random::<f32>() * TAU,
            ring_colors,
            sun_rings,
            stars: Vec::new(),
            cacti: Vec::new(),
            tumbleweed: None,
            tumbleweed_timer: rng
                .random_range(TUMBLEWEED_SPAWN_MIN_SECS..TUMBLEWEED_SPAWN_MAX_SECS),
            ripple_timer: RING_RIPPLE_SECS,
        }
    }

    pub fn is_night(&self) -> bool {
        self.sky_angle.sin() < 0.0
    }

    pub fn bubble_color(&self) -> Color {
        if self.is_night() {
            NIGHT_BUBBLE_COLOR
        } else {
            DAY_BUBBLE_COLOR
        }
    }

    pub fn night_progress(&self) -> f32 {
        self.sky_angle - PI
    }

    pub fn init_stars(&mut self, rng: &mut impl RngExt, w: u16, h: u16) {
        let target = star_field_target(w, h);
        while self.stars.len() < target {
            self.stars.push(DesertStar::new(rng));
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, w: u16, h: u16) {
        if w == 0 || h == 0 {
            return;
        }
        self.sky_angle = (self.sky_angle + SKY_SPEED * dt).rem_euclid(TAU);
        self.h_phase = (self.h_phase + SUN_H_WAVE_SPEED * dt).rem_euclid(TAU);
        self.ripple_timer -= dt;
        if self.ripple_timer <= 0.0 {
            self.ripple_timer += RING_RIPPLE_SECS;
            let n = self.ring_colors.len();
            for i in (1..n).rev() {
                self.ring_colors[i] = self.ring_colors[i - 1];
            }
            if n > 0 {
                self.ring_colors[0] = random_palette(rng);
            }
        }
        for star in &mut self.stars {
            star.twinkle.tick(dt, rng);
        }
        self.tick_tumbleweed(dt, rng, w);
        self.init_stars(rng, w, h);
    }

    fn tick_tumbleweed(&mut self, dt: f32, rng: &mut impl RngExt, w: u16) {
        let mut despawn = false;
        if let Some(tw) = &mut self.tumbleweed {
            tw.tick(dt);
            despawn = tw.x > w as f32;
        }
        if despawn {
            self.tumbleweed = None;
        }
        if self.tumbleweed.is_some() {
            return;
        }
        self.tumbleweed_timer -= dt;
        if self.tumbleweed_timer <= 0.0 {
            self.tumbleweed_timer =
                rng.random_range(TUMBLEWEED_SPAWN_MIN_SECS..TUMBLEWEED_SPAWN_MAX_SECS);
            self.tumbleweed = Some(Tumbleweed::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_and_night_are_equal_halves_over_one_cycle() {
        let mut rng = rand::rng();
        let mut sky = DesertSky::new(&mut rng);
        sky.init_stars(&mut rng, 80, 24);
        assert!(!sky.is_night(), "cycle starts at noon (sun at apex)");

        let dt = 1.0 / 30.0;
        let steps = (CYCLE_SECS / dt).round() as usize;
        let mut night_steps = 0usize;
        for _ in 0..steps {
            sky.tick(dt, &mut rng, 80, 24);
            if sky.is_night() {
                night_steps += 1;
            }
            for row in &sky.sun_rings {
                for &ring in row {
                    let _ = sun_cell_color(ring, &sky.ring_colors);
                }
            }
        }
        let night_fraction = night_steps as f32 / steps as f32;
        assert!(
            (night_fraction - 0.5).abs() < 0.02,
            "night should fill half the cycle, got {night_fraction}"
        );
        assert!(!sky.ring_colors.is_empty());
    }

    #[test]
    fn sun_eyes_skull_and_many_rings() {
        let mut rng = rand::rng();
        let sky = DesertSky::new(&mut rng);
        let mut eye_cells = 0;
        let mut skull_cells = 0;
        for (row, line) in SUN_LINES.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let ring = sky.sun_rings[row][col];
                if ch == '@' {
                    assert_eq!(ring, SUN_RING_EYE, "@ at {row},{col} must be an eye");
                    eye_cells += 1;
                }
                if ring == SUN_RING_SKULL {
                    skull_cells += 1;
                }
            }
        }
        assert_eq!(eye_cells, 2, "two eyes");
        assert!(skull_cells > 0, "face must map to the skull ring");
        assert!(
            sky.ring_colors.len() >= 7,
            "bigger sprite should yield many rings, got {}",
            sky.ring_colors.len()
        );
    }

    #[test]
    fn cactus_interior_spaces_are_opaque_not_transparent() {
        let mut rng = rand::rng();
        let mut cacti = Vec::new();
        extend_desert_cacti(&mut cacti, 600, &mut rng);
        let opaque_blanks: usize = cacti
            .iter()
            .flat_map(|c| c.grid.iter())
            .flatten()
            .filter(|&&(ch, _)| ch == ' ')
            .count();
        assert!(
            opaque_blanks > 0,
            "cacti with enclosed gaps should produce opaque interior blanks"
        );
    }

    #[test]
    fn cacti_extend_and_keep_base_at_bottom() {
        let mut rng = rand::rng();
        let mut cacti = Vec::new();
        extend_desert_cacti(&mut cacti, 400, &mut rng);
        assert!(!cacti.is_empty());
        for c in &cacti {
            assert!(c.width > 0);
            assert!(c.grid.len() <= CACTUS_HEIGHT_MAX);
            for row in &c.grid {
                assert_eq!(row.len(), c.width as usize);
            }
        }
    }
}
