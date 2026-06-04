use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{BROWN, DARK_GRAY, LIGHT_GREEN, LIGHT_YELLOW, PINK, WHITE};
use crate::entities::components::{Position, SwayState, tick_sway};
use crate::fishes::mutant::{EyeState, MutantState, MutantTail, MutationRecord};
use crate::fishes::mutations::Mutatable;

pub const COW_SPRITE_ROWS: u16 = 5;
pub const COW_BASE_TORSO: usize = 7;
const COW_HEAD_CAP: usize = 5;
const COW_HEAD_TOP_W: usize = 4;
pub const COW_DEFAULT_SWAY_SPEED: f32 = 0.05;
pub const COW_TRANSPARENT: char = crate::sprite::TRANSPARENT;
pub const COW_ANTENNA_ROWS: u16 = 1;
const ANTENNA_BALL: char = 'o';
const ANTENNA_LEFT_STEM: char = '\\';
const ANTENNA_RIGHT_STEM: char = '/';
const ANTENNA_STEM_FILL: char = '_';
const ALIENATION_TAG: &str = "alienation";

pub fn cow_default_sway_speed() -> f32 {
    COW_DEFAULT_SWAY_SPEED
}
const COW_MIN_TORSO: usize = 3;
const COW_MAX_TORSO: usize = 12;
const SPEECH_BUBBLE_TTL: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CowVariant {
    Brown,
    WhiteBlack,
    Pink,
    LightYellow,
    LightGreen,
}

impl CowVariant {
    pub const ALL: &'static [CowVariant] = &[
        CowVariant::Brown,
        CowVariant::WhiteBlack,
        CowVariant::Pink,
        CowVariant::LightYellow,
        CowVariant::LightGreen,
    ];

    pub fn body_color(self) -> Color {
        match self {
            CowVariant::Brown => BROWN,
            CowVariant::WhiteBlack => WHITE,
            CowVariant::Pink => PINK,
            CowVariant::LightYellow => LIGHT_YELLOW,
            CowVariant::LightGreen => LIGHT_GREEN,
        }
    }

    pub fn patches_color(self) -> Option<Color> {
        match self {
            CowVariant::WhiteBlack => Some(DARK_GRAY),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            CowVariant::Brown => "brown",
            CowVariant::WhiteBlack => "white",
            CowVariant::Pink => "pink",
            CowVariant::LightYellow => "yellow",
            CowVariant::LightGreen => "alien",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "brown" => Some(CowVariant::Brown),
            "white" | "whiteblack" | "white-black" => Some(CowVariant::WhiteBlack),
            "pink" => Some(CowVariant::Pink),
            "yellow" | "lightyellow" => Some(CowVariant::LightYellow),
            "alien" | "lightgreen" | "green" => Some(CowVariant::LightGreen),
            _ => None,
        }
    }

    pub fn random(rng: &mut impl RngExt) -> Self {
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }
}

#[derive(Clone)]
pub struct SpeechBubble {
    pub text: String,
    pub ttl: f32,
}

impl SpeechBubble {
    pub fn new(text: String) -> Self {
        Self {
            text,
            ttl: SPEECH_BUBBLE_TTL,
        }
    }
}

#[derive(Clone)]
pub struct Cow {
    pub name: String,
    pub position: Position,
    pub variant: CowVariant,
    pub color: Color,
    pub body_length: usize,
    pub sway: SwayState,
    pub sway_speed: f32,
    pub mutant: Box<MutantState>,
    pub mutations: Option<Box<MutationRecord>>,
    pub speech: Option<SpeechBubble>,
    pub display_width: usize,
}

impl Cow {
    pub fn new(name: String, variant: CowVariant, x: f32, y: f32, rng: &mut impl RngExt) -> Self {
        let mut mutant = MutantState::new_for_standard(MutantTail::Wide, rng);
        mutant.left_eyes.clear();
        mutant.right_eyes.clear();
        for _ in 0..2 {
            mutant.left_eyes.push(EyeState::new(rng));
        }
        mutant.eye_color = Some(DARK_GRAY);
        let mut cow = Self {
            name,
            position: Position { x, y },
            variant,
            color: variant.body_color(),
            body_length: 0,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            sway_speed: COW_DEFAULT_SWAY_SPEED,
            mutant: Box::new(mutant),
            mutations: None,
            speech: None,
            display_width: 0,
        };
        cow.display_width = cow_display_width(&cow);
        cow
    }

    pub fn tick(&mut self, dt: f32) {
        tick_sway(&mut self.sway, self.sway_speed);
        self.mutant.tick_eyes(dt);
        if let Some(b) = &mut self.speech {
            b.ttl -= dt;
            if b.ttl <= 0.0 {
                self.speech = None;
            }
        }
    }

    pub fn say(&mut self, text: String) {
        if text.trim().is_empty() {
            return;
        }
        self.speech = Some(SpeechBubble::new(text));
    }

    pub fn torso_width(&self) -> usize {
        COW_BASE_TORSO + self.body_length
    }

    pub fn eye_count(&self) -> usize {
        self.mutant.left_eyes.len()
    }

    pub fn is_alienated(&self) -> bool {
        self.mutations
            .as_ref()
            .is_some_and(|m| m.history.iter().any(|h| h == ALIENATION_TAG))
    }

    pub fn sprite_top_offset(&self) -> u16 {
        if self.is_alienated() {
            COW_ANTENNA_ROWS
        } else {
            0
        }
    }
}

impl Cow {
    pub const fn sprite_height() -> u16 {
        COW_SPRITE_ROWS
    }
}

impl Mutatable for Cow {
    fn body_size(&self) -> usize {
        self.body_length + COW_BASE_TORSO
    }
    fn set_body_size(&mut self, n: usize) {
        let clamped = n.clamp(COW_MIN_TORSO, COW_MAX_TORSO);
        self.body_length = clamped.saturating_sub(COW_BASE_TORSO);
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, c: Color) {
        self.color = c;
    }
    fn sway_speed(&self) -> f32 {
        self.sway_speed
    }
    fn set_sway_speed(&mut self, s: f32) {
        self.sway_speed = s;
    }
    fn default_sway_speed(&self) -> f32 {
        COW_DEFAULT_SWAY_SPEED
    }
    fn mutant(&self) -> &MutantState {
        &self.mutant
    }
    fn mutant_mut(&mut self) -> &mut MutantState {
        &mut self.mutant
    }
    fn mutations_record_mut(&mut self) -> &mut MutationRecord {
        self.mutations
            .get_or_insert_with(|| Box::new(MutationRecord::default()))
    }
    fn doublefish_eye_count<R: RngExt>(&self, _rng: &mut R) -> usize {
        self.eye_count().clamp(2, COW_HEAD_CAP)
    }
    fn allows_tail_variant(&self) -> bool {
        false
    }
    fn recompute_display_width(&mut self) {
        self.display_width = cow_display_width(self);
    }
    fn color_patch_range(&self) -> usize {
        cow_paintable_cell_count(self)
    }
}

pub fn cow_paintable_cell_count(cow: &Cow) -> usize {
    cow_sprite(cow)
        .iter()
        .flatten()
        .filter(|(c, _)| *c != ' ' && *c != COW_TRANSPARENT)
        .count()
}

pub fn cow_display_width(cow: &Cow) -> usize {
    let torso = cow.torso_width();
    let head_eye_render = cow_head_render_count(cow);
    let head_w = (head_eye_render.max(2)) + 2;
    if cow.mutant.is_double {
        let right_head_w = (cow.mutant.double_head_eyes.len().max(1)) + 2;
        head_w + torso + 1 + right_head_w
    } else {
        let tail = 4;
        let row3_w = head_w + torso + tail;
        let row2_w = head_w + 1 + torso;
        row3_w.max(row2_w)
    }
}

fn cow_head_render_count(cow: &Cow) -> usize {
    let eyes = cow.eye_count();
    eyes.clamp(2, COW_HEAD_CAP)
}

pub fn cow_sprite(cow: &Cow) -> Vec<Vec<(char, Color)>> {
    let body = cow.color;
    let eye_color = cow.mutant.eye_color.unwrap_or(DARK_GRAY);
    let patch_color = cow.variant.patches_color();
    let phase = cow.sway.phase;
    let glisten_color = cow.mutant.glistening_color;
    let glisten_mode = cow.mutant.glistening_mode;

    let total_eyes = cow.eye_count();
    let head_render = cow_head_render_count(cow);
    let head_eye_render = head_render.min(total_eyes);
    let head_w = head_render + 2;
    let torso = cow.torso_width();
    let body_overflow = total_eyes.saturating_sub(head_eye_render);
    let alienated = cow.is_alienated();

    if cow.mutant.is_double {
        let mut rows = double_cow_sprite(cow, head_render, torso, alienated);
        if let Some(pc) = patch_color {
            apply_random_patches(
                &mut rows,
                pc,
                cow.position.x as u64 ^ (cow.name.len() as u64) << 8,
            );
        }
        if let Some(peak) = glisten_color {
            crate::sprite::apply_glisten(&mut rows, phase, glisten_mode, cow.color, peak);
        }
        for &(pos, color) in &cow.mutant.color_patches {
            apply_color_patch(&mut rows, pos, color);
        }
        return rows;
    }

    let row1 = build_row1(head_w, body, alienated);

    let mut row2: Vec<(char, Color)> = Vec::new();
    row2.push(('(', body));
    for i in 0..head_render {
        let open = cow.mutant.left_eyes.get(i).is_none_or(|e| e.is_open());
        let ch = if i < head_eye_render && open {
            'o'
        } else if i < head_eye_render {
            '-'
        } else {
            ' '
        };
        row2.push((ch, eye_color));
    }
    row2.push((')', body));
    row2.push(('\\', body));
    for _ in 0..torso {
        row2.push(('_', body));
    }

    let mut row3: Vec<(char, Color)> = Vec::new();
    for _ in 0..(head_w.saturating_sub(4)) {
        row3.push((COW_TRANSPARENT, body));
    }
    row3.push(('(', body));
    row3.push(('_', body));
    row3.push(('_', body));
    row3.push((')', body));
    row3.push(('\\', body));
    for i in 0..torso {
        let eye_index = head_eye_render + i;
        let is_eye = i < body_overflow;
        let open = cow
            .mutant
            .left_eyes
            .get(eye_index)
            .is_none_or(|e| e.is_open());
        let ch = if is_eye && open {
            'o'
        } else if is_eye {
            '-'
        } else {
            ' '
        };
        let color = if is_eye { eye_color } else { body };
        row3.push((ch, color));
    }
    row3.push((')', body));
    row3.push(('\\', body));
    row3.push(('/', body));
    row3.push(('\\', body));

    let leg_indent = head_w.saturating_sub(1);
    let mut row4: Vec<(char, Color)> = Vec::new();
    for _ in 0..leg_indent {
        row4.push((COW_TRANSPARENT, body));
    }
    row4.push(('|', body));
    row4.push(('|', body));
    for _ in 0..(torso.saturating_sub(3)) {
        row4.push(('-', body));
    }
    row4.push(('-', body));
    row4.push(('w', body));
    row4.push((' ', body));
    row4.push(('|', body));

    let mut row5: Vec<(char, Color)> = Vec::new();
    for _ in 0..leg_indent {
        row5.push((COW_TRANSPARENT, body));
    }
    row5.push(('|', body));
    row5.push(('|', body));
    for _ in 1..torso {
        row5.push((COW_TRANSPARENT, body));
    }
    row5.push(('|', body));
    row5.push(('|', body));

    let mut rows = vec![row1, row2, row3, row4, row5];
    if alienated {
        rows.insert(0, build_antenna_row(head_w, body));
    }

    if let Some(pc) = patch_color {
        apply_random_patches(
            &mut rows,
            pc,
            cow.position.x as u64 ^ (cow.name.len() as u64) << 8,
        );
    }

    if let Some(peak) = glisten_color {
        crate::sprite::apply_glisten(&mut rows, phase, glisten_mode, cow.color, peak);
    }

    for &(pos, color) in &cow.mutant.color_patches {
        apply_color_patch(&mut rows, pos, color);
    }

    rows
}

fn build_row1(head_w: usize, body: Color, alienated: bool) -> Vec<(char, Color)> {
    let mut row = Vec::new();
    let pad = head_w.saturating_sub(COW_HEAD_TOP_W) / 2;
    for _ in 0..pad {
        row.push((COW_TRANSPARENT, body));
    }
    if alienated {
        row.push((ANTENNA_LEFT_STEM, body));
        row.push((ANTENNA_STEM_FILL, body));
        row.push((ANTENNA_STEM_FILL, body));
        row.push((ANTENNA_RIGHT_STEM, body));
    } else {
        row.push(('^', body));
        row.push(('_', body));
        row.push(('_', body));
        row.push(('^', body));
    }
    row
}

fn build_antenna_row(head_w: usize, body: Color) -> Vec<(char, Color)> {
    let mut row = Vec::new();
    let pad = head_w.saturating_sub(COW_HEAD_TOP_W) / 2;
    for _ in 0..pad {
        row.push((COW_TRANSPARENT, body));
    }
    row.push((ANTENNA_BALL, body));
    row.push((COW_TRANSPARENT, body));
    row.push((COW_TRANSPARENT, body));
    row.push((ANTENNA_BALL, body));
    row
}

fn double_cow_sprite(
    cow: &Cow,
    head_render: usize,
    torso: usize,
    alienated: bool,
) -> Vec<Vec<(char, Color)>> {
    let body = cow.color;
    let eye_color = cow.mutant.eye_color.unwrap_or(DARK_GRAY);
    let head_w = head_render + 2;
    let right_eyes = cow.mutant.double_head_eyes.len().max(1);
    let right_head_w = right_eyes + 2;

    let left_pad = head_w.saturating_sub(COW_HEAD_TOP_W) / 2;
    let right_head_col = head_w + torso + 2;
    let right_pad = right_head_w.saturating_sub(COW_HEAD_TOP_W) / 2;
    let right_top_col = right_head_col + right_pad;

    let (left_a, left_b, left_c, left_d) = if alienated {
        (
            ANTENNA_LEFT_STEM,
            ANTENNA_STEM_FILL,
            ANTENNA_STEM_FILL,
            ANTENNA_RIGHT_STEM,
        )
    } else {
        ('^', '_', '_', '^')
    };

    let mut row1 = Vec::new();
    for _ in 0..left_pad {
        row1.push((COW_TRANSPARENT, body));
    }
    row1.push((left_a, body));
    row1.push((left_b, body));
    row1.push((left_c, body));
    row1.push((left_d, body));
    while row1.len() < right_top_col {
        row1.push((COW_TRANSPARENT, body));
    }
    row1.push((left_a, body));
    row1.push((left_b, body));
    row1.push((left_c, body));
    row1.push((left_d, body));

    let mut row2 = Vec::new();
    row2.push(('(', body));
    for i in 0..head_render {
        let open = cow.mutant.left_eyes.get(i).is_none_or(|e| e.is_open());
        row2.push((if open { 'o' } else { '-' }, eye_color));
    }
    row2.push((')', body));
    row2.push(('\\', body));
    for _ in 0..torso {
        row2.push(('_', body));
    }
    row2.push(('/', body));
    row2.push(('(', body));
    for i in 0..right_eyes {
        let open = cow
            .mutant
            .double_head_eyes
            .get(i)
            .is_none_or(|e| e.is_open());
        row2.push((if open { 'o' } else { '-' }, eye_color));
    }
    row2.push((')', body));

    let mut row3 = vec![
        ('(', body),
        ('_', body),
        ('_', body),
        (')', body),
        ('\\', body),
    ];
    for _ in 0..torso {
        row3.push((' ', body));
    }
    row3.push(('/', body));
    row3.push(('(', body));
    row3.push(('_', body));
    row3.push(('_', body));
    row3.push((')', body));

    let leg_indent = head_w.saturating_sub(1);
    let mut row4 = Vec::new();
    for _ in 0..leg_indent {
        row4.push((COW_TRANSPARENT, body));
    }
    row4.push(('|', body));
    row4.push(('|', body));
    row4.push(('w', body));
    for _ in 0..(torso.saturating_sub(3)) {
        row4.push(('-', body));
    }
    row4.push(('-', body));
    row4.push(('w', body));
    row4.push(('|', body));
    row4.push(('|', body));

    let mut row5 = Vec::new();
    for _ in 0..leg_indent {
        row5.push((COW_TRANSPARENT, body));
    }
    row5.push(('|', body));
    row5.push(('|', body));
    for _ in 0..torso {
        row5.push((COW_TRANSPARENT, body));
    }
    row5.push(('|', body));
    row5.push(('|', body));

    let mut rows = vec![row1, row2, row3, row4, row5];
    if alienated {
        let mut antenna = Vec::new();
        for _ in 0..left_pad {
            antenna.push((COW_TRANSPARENT, body));
        }
        antenna.push((ANTENNA_BALL, body));
        antenna.push((COW_TRANSPARENT, body));
        antenna.push((COW_TRANSPARENT, body));
        antenna.push((ANTENNA_BALL, body));
        while antenna.len() < right_top_col {
            antenna.push((COW_TRANSPARENT, body));
        }
        antenna.push((ANTENNA_BALL, body));
        antenna.push((COW_TRANSPARENT, body));
        antenna.push((COW_TRANSPARENT, body));
        antenna.push((ANTENNA_BALL, body));
        rows.insert(0, antenna);
    }
    rows
}

fn apply_color_patch(rows: &mut [Vec<(char, Color)>], linear_pos: usize, color: Color) {
    let mut idx = 0usize;
    for row in rows.iter_mut() {
        for cell in row.iter_mut() {
            if cell.0 != ' ' && cell.0 != COW_TRANSPARENT {
                if idx == linear_pos {
                    cell.1 = color;
                    return;
                }
                idx += 1;
            }
        }
    }
}

fn apply_random_patches(rows: &mut [Vec<(char, Color)>], patch: Color, seed: u64) {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    let mut rng = SmallRng::seed_from_u64(seed);
    let n_underscores: usize = rows.iter().flatten().filter(|(c, _)| *c == '_').count();
    let n_patch = (n_underscores / 3).max(2);
    for _ in 0..n_patch {
        let mut target = rng.random_range(0..n_underscores.max(1));
        for row in rows.iter_mut() {
            for cell in row.iter_mut() {
                if cell.0 == '_' {
                    if target == 0 {
                        cell.1 = patch;
                        break;
                    }
                    target -= 1;
                }
            }
        }
    }
}

pub fn build_speech_bubble(text: &str) -> Vec<String> {
    let inner = text.len();
    let top: String = std::iter::repeat_n('_', inner + 2).collect();
    let bot: String = std::iter::repeat_n('-', inner + 2).collect();
    vec![
        format!(" {} ", top),
        format!("< {} >", text),
        format!(" {} ", bot),
    ]
}

pub fn random_cow_color(rng: &mut impl RngExt) -> CowVariant {
    CowVariant::random(rng)
}
