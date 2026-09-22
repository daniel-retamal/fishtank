use ratatui::style::Color;

use crate::colors::{CYAN, DARK_GRAY, LIGHT_CYAN, WHITE};
use crate::entities::cow::Cow;
use crate::entities::glistening::{GlisteningMode, color_for_glisten};
use crate::fishes::fish::Fish;

pub const UFO_CANVAS_W: usize = 22;
pub const UFO_SHIP_ROWS: usize = 6;
pub const UFO_CONE_ROWS_MAX: usize = 10;
pub const UFO_SPRITE_HEIGHT: usize = UFO_SHIP_ROWS + UFO_CONE_ROWS_MAX;
pub const UFO_SPRITE_WIDTH: usize = UFO_CANVAS_W;

pub const UFO_CENTER_COL: i32 = 11;
pub const UFO_PAYLOAD_CONE_ROW: usize = 8;

const DESCENT_SPEED: f32 = 13.0;
const ASCENT_SPEED: f32 = 13.0;
const CONE_ROW_REVEAL_INTERVAL: f32 = 0.12;
const HOLD_TIME_SECS: f32 = 2.0;
const O_TOGGLE_INTERVAL: f32 = 0.10;
const CONE_GLISTEN_SPEED: f32 = 25.0;

pub const UFO_SHIP_LINES: [&str; UFO_SHIP_ROWS] = [
    "         _____       ",
    "        (_____)      ",
    "   _.-~`       `~-._ ",
    "  /O O O O O O O O O\\",
    "  \\_________________/",
    "       \\_______/     ",
];

pub const UFO_SHIP_O_TOGGLED_ROW: &str = "  / O O O O O O O O \\";

pub const UFO_CONE_LINES: [&str; UFO_CONE_ROWS_MAX] = [
    "          / \\         ",
    "         /   \\        ",
    "        /     \\       ",
    "       /       \\      ",
    "      /         \\     ",
    "     /           \\    ",
    "    /             \\   ",
    "   /               \\  ",
    "  /        X        \\ ",
    " /                   \\",
];

pub const UFO_COW_BAY_LINES: [&str; 5] = [
    "     /-__- ___,  \\ ",
    "    / (oo)\\    )\\ \\ ",
    "   /  (__)//--///  \\",
    "  /       \\\\_ \\\\    \\",
    " /                   \\",
];

pub enum UfoPayload {
    AbductingFish { fish_name: String },
    DroppingFish(Box<Fish>),
    DroppingCow(Box<Cow>),
}

pub enum UfoPhase {
    Descending,
    GrowingCone { rows_shown: usize, timer: f32 },
    Holding { ticks_left: f32 },
    ShrinkingCone { rows_left: usize, timer: f32 },
    Ascending,
    Done,
}

pub struct Ufo {
    pub x: f32,
    pub y: f32,
    pub target_y: f32,
    pub phase: UfoPhase,
    pub payload: UfoPayload,
    pub o_toggle_phase: f32,
    pub o_toggled: bool,
    pub glisten_phase: f32,
}

impl Ufo {
    fn empty(x: f32, target_y: f32, payload: UfoPayload) -> Self {
        Self {
            x,
            y: -(UFO_SPRITE_HEIGHT as f32),
            target_y,
            phase: UfoPhase::Descending,
            payload,
            o_toggle_phase: 0.0,
            o_toggled: false,
            glisten_phase: 0.0,
        }
    }

    pub fn new_abduct(x: f32, target_y: f32, fish_name: String) -> Self {
        Self::empty(x, target_y, UfoPayload::AbductingFish { fish_name })
    }

    pub fn new_drop_fish(x: f32, target_y: f32, mut fish: Fish) -> Self {
        fish.abduction_lock = true;
        let mut ufo = Self::empty(x, target_y, UfoPayload::DroppingFish(Box::new(fish)));
        ufo.carry();
        ufo
    }

    fn carry(&mut self) {
        let (x, y) = self.payload_world_pos();
        if let UfoPayload::DroppingFish(fish) = &mut self.payload {
            fish.position.x = x - (fish.display_width / 2) as f32;
            fish.position.y = y;
        }
    }

    pub fn abductee(&self) -> Option<&str> {
        match &self.payload {
            UfoPayload::AbductingFish { fish_name } if !fish_name.is_empty() => Some(fish_name),
            _ => None,
        }
    }

    pub fn carried_fish(&self) -> Option<&Fish> {
        match &self.payload {
            UfoPayload::DroppingFish(fish) => Some(fish),
            _ => None,
        }
    }

    fn release_carried_fish(&mut self) -> Option<UfoTickResult> {
        let placeholder = UfoPayload::AbductingFish {
            fish_name: String::new(),
        };
        match std::mem::replace(&mut self.payload, placeholder) {
            UfoPayload::DroppingFish(mut fish) => {
                fish.abduction_lock = false;
                Some(UfoTickResult::ReleaseFish(*fish))
            }
            other => {
                self.payload = other;
                None
            }
        }
    }

    pub fn new_drop_cow(x: f32, target_y: f32, cow: Cow) -> Self {
        Self::empty(x, target_y, UfoPayload::DroppingCow(Box::new(cow)))
    }

    pub fn tick(&mut self, dt: f32) -> UfoTickResult {
        self.o_toggle_phase += dt;
        if self.o_toggle_phase >= O_TOGGLE_INTERVAL {
            self.o_toggle_phase -= O_TOGGLE_INTERVAL;
            self.o_toggled = !self.o_toggled;
        }
        self.glisten_phase = (self.glisten_phase + CONE_GLISTEN_SPEED * dt) % 1000.0;

        let result = self.advance(dt);
        self.carry();
        result
    }

    fn advance(&mut self, dt: f32) -> UfoTickResult {
        match &mut self.phase {
            UfoPhase::Descending => {
                self.y += DESCENT_SPEED * dt;
                if self.y >= self.target_y {
                    self.y = self.target_y;
                    if self.is_delivering() {
                        self.phase = UfoPhase::Holding {
                            ticks_left: HOLD_TIME_SECS,
                        };
                    } else {
                        self.phase = UfoPhase::GrowingCone {
                            rows_shown: 0,
                            timer: 0.0,
                        };
                    }
                }
            }
            UfoPhase::GrowingCone { rows_shown, timer } => {
                *timer += dt;
                if *timer >= CONE_ROW_REVEAL_INTERVAL {
                    *timer -= CONE_ROW_REVEAL_INTERVAL;
                    *rows_shown += 1;
                    if *rows_shown >= UFO_CONE_ROWS_MAX {
                        let result = match &self.payload {
                            UfoPayload::AbductingFish { fish_name } => {
                                UfoTickResult::LockFish(fish_name.clone())
                            }
                            _ => UfoTickResult::None,
                        };
                        self.phase = UfoPhase::Holding {
                            ticks_left: HOLD_TIME_SECS,
                        };
                        return result;
                    }
                }
            }
            UfoPhase::Holding { ticks_left } => {
                *ticks_left -= dt;
                if *ticks_left <= 0.0 {
                    let release = match std::mem::replace(
                        &mut self.payload,
                        UfoPayload::AbductingFish {
                            fish_name: String::new(),
                        },
                    ) {
                        UfoPayload::DroppingCow(c) => Some(UfoTickResult::ReleaseCow(*c)),
                        kept => {
                            self.payload = kept;
                            None
                        }
                    };
                    self.phase = UfoPhase::ShrinkingCone {
                        rows_left: UFO_CONE_ROWS_MAX,
                        timer: 0.0,
                    };
                    if let Some(r) = release {
                        return r;
                    }
                }
            }
            UfoPhase::ShrinkingCone { rows_left, timer } => {
                *timer += dt;
                if *timer >= CONE_ROW_REVEAL_INTERVAL {
                    *timer -= CONE_ROW_REVEAL_INTERVAL;
                    if *rows_left > 0 {
                        *rows_left -= 1;
                    }
                    let take_fish = matches!(&self.payload, UfoPayload::AbductingFish { fish_name } if !fish_name.is_empty())
                        && *rows_left == UFO_PAYLOAD_CONE_ROW;
                    if take_fish {
                        let name = if let UfoPayload::AbductingFish { fish_name } =
                            std::mem::replace(
                                &mut self.payload,
                                UfoPayload::AbductingFish {
                                    fish_name: String::new(),
                                },
                            ) {
                            fish_name
                        } else {
                            String::new()
                        };
                        if *rows_left == 0 {
                            self.phase = UfoPhase::Ascending;
                        }
                        return UfoTickResult::TakeFish(name);
                    }
                    if *rows_left == 0 {
                        self.phase = UfoPhase::Ascending;
                        if let Some(released) = self.release_carried_fish() {
                            return released;
                        }
                    }
                }
            }
            UfoPhase::Ascending => {
                self.y -= ASCENT_SPEED * dt;
                if self.y + UFO_SPRITE_HEIGHT as f32 <= 0.0 {
                    self.phase = UfoPhase::Done;
                    return UfoTickResult::Finished;
                }
            }
            UfoPhase::Done => return UfoTickResult::Finished,
        }
        UfoTickResult::None
    }

    pub fn is_done(&self) -> bool {
        matches!(self.phase, UfoPhase::Done)
    }

    pub fn cone_rows(&self) -> usize {
        match &self.phase {
            UfoPhase::Descending => {
                if self.is_delivering() {
                    UFO_CONE_ROWS_MAX
                } else {
                    0
                }
            }
            UfoPhase::Ascending | UfoPhase::Done => 0,
            UfoPhase::GrowingCone { rows_shown, .. } => *rows_shown,
            UfoPhase::Holding { .. } => UFO_CONE_ROWS_MAX,
            UfoPhase::ShrinkingCone { rows_left, .. } => *rows_left,
        }
    }

    pub fn cow_bay_colors(&self) -> Option<(Color, Color)> {
        match &self.payload {
            UfoPayload::DroppingCow(c) => Some((c.color, c.mutant.eye_color.unwrap_or(DARK_GRAY))),
            _ => None,
        }
    }

    pub fn is_delivering(&self) -> bool {
        matches!(
            self.payload,
            UfoPayload::DroppingCow(_) | UfoPayload::DroppingFish(_)
        )
    }

    pub fn is_following_phase(&self) -> bool {
        matches!(
            self.phase,
            UfoPhase::Descending | UfoPhase::GrowingCone { .. }
        )
    }

    pub fn payload_world_pos(&self) -> (f32, f32) {
        (
            self.x + UFO_CENTER_COL as f32,
            self.y + (UFO_SHIP_ROWS + UFO_PAYLOAD_CONE_ROW) as f32,
        )
    }
}

pub enum UfoTickResult {
    None,
    LockFish(String),
    TakeFish(String),
    ReleaseFish(Fish),
    ReleaseCow(Cow),
    Finished,
}

pub fn ufo_sprite(ufo: &Ufo) -> Vec<Vec<(char, Color)>> {
    let mut rows: Vec<Vec<(char, Color)>> = Vec::with_capacity(UFO_SPRITE_HEIGHT);
    let cone_rows_shown = ufo.cone_rows();
    let cow_bay_colors = ufo.cow_bay_colors();
    let interior_mask = cow_bay_colors.map(|_| cow_bay_interior_mask());

    for (idx, line) in UFO_SHIP_LINES.iter().enumerate() {
        let chosen = if idx == 3 && ufo.o_toggled {
            UFO_SHIP_O_TOGGLED_ROW
        } else {
            line
        };
        rows.push(color_ship_row(chosen));
    }

    let (palette_base, palette_mid, palette_peak) = (CYAN, CYAN, LIGHT_CYAN);
    for (cone_idx, &cone_line) in UFO_CONE_LINES.iter().enumerate().take(cone_rows_shown) {
        let wave_color = color_for_glisten(
            GlisteningMode::Wave,
            ufo.glisten_phase,
            cone_idx,
            UFO_CONE_ROWS_MAX,
            palette_base,
            palette_mid,
            palette_peak,
        );
        let cow_overlay_row = if cow_bay_colors.is_some()
            && cone_idx >= UFO_CONE_ROWS_MAX - UFO_COW_BAY_LINES.len()
        {
            Some(cone_idx - (UFO_CONE_ROWS_MAX - UFO_COW_BAY_LINES.len()))
        } else {
            None
        };
        let line = match cow_overlay_row {
            Some(row) => UFO_COW_BAY_LINES[row],
            None => cone_line,
        };
        let row_cow_colors = if cow_overlay_row.is_some() {
            cow_bay_colors
        } else {
            None
        };
        let interior =
            cow_overlay_row.and_then(|row| interior_mask.as_ref().map(|m| m[row].as_slice()));
        rows.push(color_cone_row(line, wave_color, row_cow_colors, interior));
    }

    rows
}

fn color_ship_row(line: &str) -> Vec<(char, Color)> {
    let chars: Vec<char> = line.chars().collect();
    let first = chars.iter().position(|&c| c != ' ');
    let last = chars.iter().rposition(|&c| c != ' ');
    chars
        .iter()
        .enumerate()
        .map(|(i, &ch)| match ch {
            'O' => (ch, CYAN),
            ' ' => {
                let interior = matches!((first, last), (Some(f), Some(l)) if i > f && i < l);
                if interior {
                    (' ', DARK_GRAY)
                } else {
                    ('\0', DARK_GRAY)
                }
            }
            _ => (ch, DARK_GRAY),
        })
        .collect()
}

fn color_cone_row(
    line: &str,
    wall_color: Color,
    cow_bay: Option<(Color, Color)>,
    interior: Option<&[bool]>,
) -> Vec<(char, Color)> {
    let (body_c, eye_c) = cow_bay.unwrap_or((WHITE, DARK_GRAY));
    let bay = cow_bay.is_some();
    let first_slash = line.char_indices().find(|(_, c)| *c == '/').map(|(i, _)| i);
    let last_backslash = line
        .char_indices()
        .rfind(|(_, c)| *c == '\\')
        .map(|(i, _)| i);
    line.char_indices()
        .map(|(idx, ch)| match ch {
            '/' if Some(idx) == first_slash => (ch, wall_color),
            '\\' if Some(idx) == last_backslash => (ch, wall_color),
            '/' | '\\' => (ch, if bay { body_c } else { wall_color }),
            'X' | ' ' => {
                if bay && interior.and_then(|m| m.get(idx)).copied().unwrap_or(false) {
                    (' ', body_c)
                } else {
                    ('\0', wall_color)
                }
            }
            'o' if bay => (ch, eye_c),
            _ if bay => (ch, body_c),
            _ => (ch, wall_color),
        })
        .collect()
}

fn cow_bay_interior_mask() -> Vec<Vec<bool>> {
    let width = UFO_COW_BAY_LINES
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let height = UFO_COW_BAY_LINES.len();
    let passable: Vec<Vec<bool>> = UFO_COW_BAY_LINES
        .iter()
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let mut row: Vec<bool> = chars.iter().map(|&c| c == ' ').collect();
            while row.len() < width {
                row.push(true);
            }
            row
        })
        .collect();
    let mut exterior = vec![vec![false; width]; height];
    let mut queue: std::collections::VecDeque<(usize, usize)> = std::collections::VecDeque::new();
    for r in 0..height {
        for c in 0..width {
            if (r == 0 || r + 1 == height || c == 0 || c + 1 == width)
                && passable[r][c]
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
            if !exterior[nr][nc] && passable[nr][nc] {
                exterior[nr][nc] = true;
                queue.push_back((nr, nc));
            }
        }
    }
    (0..height)
        .map(|r| {
            (0..width)
                .map(|c| passable[r][c] && !exterior[r][c])
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::species::FishSpecies;

    const FRAME_SECS: f32 = 1.0 / 30.0;
    const TICK_BUDGET: usize = 10_000;
    const TARGET_Y: f32 = 10.0;

    fn a_fish() -> Fish {
        Fish::new(
            FishSpecies::Merluza,
            "Zed".to_string(),
            0.0,
            0.0,
            &mut rand::rng(),
        )
    }

    #[test]
    fn a_fish_rides_down_an_open_beam_and_is_let_go_only_once_the_beam_is_back() {
        let mut ufo = Ufo::new_drop_fish(20.0, TARGET_Y, a_fish());
        for _ in 0..TICK_BUDGET {
            let carried = ufo
                .carried_fish()
                .map(|fish| (fish.abduction_lock, fish.position.clone()));
            let cone = ufo.cone_rows();
            let (ufo_x, ufo_y) = ufo.payload_world_pos();
            if let Some((locked, position)) = carried {
                assert!(locked, "a carried fish is held");
                assert_eq!(position.y, ufo_y, "it rides the payload row");
                assert!((position.x - ufo_x).abs() <= UFO_CENTER_COL as f32);
                if matches!(ufo.phase, UfoPhase::Descending) {
                    assert_eq!(cone, UFO_CONE_ROWS_MAX, "the beam is open on the way down");
                }
            }
            if let UfoTickResult::ReleaseFish(fish) = ufo.tick(FRAME_SECS) {
                assert_eq!(
                    ufo.cone_rows(),
                    0,
                    "the beam is back before the fish is let go"
                );
                assert!(!fish.abduction_lock, "a released fish is free");
                assert_eq!(
                    fish.position.y,
                    TARGET_Y + (UFO_SHIP_ROWS + UFO_PAYLOAD_CONE_ROW) as f32
                );
                return;
            }
        }
        panic!("the fish was never released");
    }
}
