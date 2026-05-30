use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use crate::colors::{
    COBALT, COBALT_DARK, COBALT_LIGHT, LIGHT_GREEN, PURPLE, PURPLE_DARK, PURPLE_LIGHT,
};
use crate::entities::components::{
    BlinkTimer, EyeRow, SwayState, extend_spaced, sway_x_offset, tick_sway,
};
use crate::entities::glistening::GlisteningMode;
use crate::entities::plant::Seaweed;

pub const ALIEN_STAR_RATIO: f32 = 0.06;

const STAR_ZERO_FRACTION: f32 = 0.6;
const STAR_TWINKLE_LIT_MIN: f32 = 0.05;
const STAR_TWINKLE_LIT_MAX: f32 = 0.25;
const STAR_TWINKLE_DARK_MIN: f32 = 3.0;
const STAR_TWINKLE_DARK_MAX: f32 = 10.0;
const ALIEN_STAR_CHARS: &[char] = &['.', '*', '\'', ':', '⋆', '⟡', '✶', '˚'];

const SWAY_SPEED: f32 = 0.02;
const WAVE_SPREAD: f32 = 0.5;
const SWAY_AMOUNT: f32 = 2.0;
pub const ALIEN_TENTACLE_WIDTH: i32 = 1;
const ALIEN_TENTACLE_GAP_MIN: i32 = 6;
const ALIEN_TENTACLE_GAP_MAX: i32 = 12;
const ALIEN_TENTACLE_HEIGHT_MIN: usize = 8;
const ALIEN_TENTACLE_HEIGHT_MAX: usize = 27;
const ALIEN_EYE_SPACING_MIN: usize = 1;
const ALIEN_EYE_SPACING_MAX: usize = 2;
const ALIEN_EYE_OPEN_CHAR: char = '0';
const ALIEN_EYE_CLOSED_CHAR: char = '-';
const ALIEN_GLISTEN_SPEED: f32 = 1.8;
const ALIEN_GLISTEN_THRESHOLD: f32 = 0.45;

const ALIEN_PYRAMID_GAP_MIN: i32 = 26;
const ALIEN_PYRAMID_GAP_MAX: i32 = 30;
const ALIEN_PYRAMID_LEFT_MIN: i32 = 1;
const ALIEN_PYRAMID_LEFT_MAX: i32 = 20;

const PYRAMID_VARIANT_COUNT: u8 = 4;
const PYRAMID_D_VARIANT: u8 = 3;
const PYRAMID_D_EYE_ROW: usize = 4;

const TENTACLE_COLORS_BLUE: &[Color] = &[COBALT, COBALT_DARK, COBALT_LIGHT];
const TENTACLE_COLORS_PURPLE: &[Color] = &[PURPLE, PURPLE_DARK, PURPLE_LIGHT];
const PYRAMID_COLOR_BLUE: Color = COBALT_LIGHT;
const PYRAMID_COLOR_PURPLE: Color = PURPLE_LIGHT;

pub const PYRAMID_A_LINES: &[&str] = &[
    "                                _/L          _L/L",
    "                              _T/_lL_      _LT/l_l_",
    "                            _lT/_l__LL_  _TLl/Ll__lL_",
    "                      _T/L _Ll/_LT__L_ _l_lL/_T__L___L_",
    "                    _lT/L_T_ /_T__LL _TLl_T/_L_|__T__|_l_",
    "                  _l_T/T___T__ __L _TL_lTl/_|__l___L__L_TL_",
    "                _TlT_/L_T____TL_ _TL_Tl_l/_l_|_|__l__L___|_L_",
    "              _TLl_l/l_L_T__L_ _l__LlTL_/__T__L__l__l_l_l__T_T_",
    "            _l_T_lL/_T___L_l _L_l__l__L/T__T____T___l___L__l_l_T_",
    "          _T_T_lL_/_L__T_l _L_TL_l_T_l/_l__l_l__T__T_L___L___T___l_",
    "        _L_l_lL_T/_l__L_ _Tl_L_TL_l_L/l__l__T__L_T___T___l___T__L__L_",
    "      _Tll_lL_TT/__l_l T_L__T_l_L_TL/l__l__L_T____T___T___T__L__T__L_T_",
    "    _Ll__lL_TT_/_T_l _l_lT_L__T_l_L/TL_l___l__l__T__l_T___T__L__T__L__TL_",
    "  _TL__lL_TT_l/_L_ _T_TLl_lT_LT_T_/_L__l__T___l__l__L___T___T__L__T__L__TL_",
    "_TLT_lL_TT_lT/_T __Ll_TT_Ll_lT_L_/T_l__T__L_l___l__l_T___T__l__T__L__T__L__L_",
];

pub const PYRAMID_B_LINES: &[&str] = &[
    "                                             _L/L",
    "                                           _LT/l_l_",
    "                                         _TLl/Ll__lL_",
    "                      _T/L             _l_lL/_T__L___L_",
    "                    _lT/L_T_         _TLl_T/_L_|__T__|_l_",
    "                  _l_T/T___T__     _TL_lTl/_|__l___L__L_TL_",
    "                _TlT_/L_T____TL_ _TL_Tl_l/_l_|_|__l__L___|_L_",
    "              _TLl_l/l_L_T__L_ _l__LlTL_/__T__L__l__l___l__T_T_",
    "            _l_T_lL/_T___L_l _L_l__l__L/T__T____T___l___L__l_l_T_",
    "          _T_T_lL_/_L__T_l _L_TL_l_T_l/_l__l_l__T__T_L___L___T___l_",
    "        _L_l_lL_T/_l__L_ _Tl_L_TL_l_L/l__l__T__L_T___T__l___T__L__TL_",
    "      _Tll_lL_TT/__l_l T_Ll_T_l_L_TL/l__l__L_T___T___T__T__L__T__L__TL_",
    "    _Ll__lL_TT_/_T_l _l_lT_L__T_l_L/TL_l___l__l__T__l_T___T__L__T__L__TL_",
    "  _TL__lL_TT_l/_L_ _T_TLl_lT_LT_T_/_L___l__T___l__l__L_T___T__T__L__T___TL_",
    "_TLT_lL_TT_lT/_T __Ll_TT_Ll_lT_L_/T_l__T__L_l___l__l_T__L_T___T__L__T__L__TL_",
];

pub const PYRAMID_C_LINES: &[&str] = &[
    "                                  _L/L",
    "                                _LT/l_l_",
    "                              _TLl/Ll__lL_",
    "                            _l_lL/_T__L___L_",
    "                          _TLl_T/_L_|__T__|_l_",
    "                        _TL_lTl/_|__l___L__L_TL_",
    "                      _TL_Tl_l/_l_|_|__l__L___|_L_",
    "                    _lT_Ll_L_/__T__L__l__l___l__T_T_",
    "                  _L_l__ll_L/T__T____T___l___L__l_l_T_",
    "                _L_TL_l_T_l/_l__l____T__T_L___L___T___l_",
    "              _Tl_L_T_ll_L/l__l__T__L_T___T__l___T__L__TL_",
    "            _TLl_T_l_L_TL/l__l__L_T____T___T_T__L__T__L__TL_",
    "          _l_lT_Ll_T_l_L/TL_l___l_T__T___T__l__T__L__T__L__TL_",
    "        _T_T_l_lT_LT_T_/_L___l__T___l__l____T_____T__L__T__L__T_",
    "      _TLl_Tl_Ll_lT_L_/T_l__T__L_l___l__l_T__L___l__T__L__T__L_TL_",
    "    _TLL_LTl_LT_LLT__/_l__T__l___L____T__l__T_____L____T____L__T__L_",
    "  _L_lT_TL_LTL__lTl_/_T___T__L____T__T____L___T___L____T__l__L___T__T_",
    "_TTT__T_LTLT_T_LT_T/_L_T_TL____L__l__T____L__l__T___L___T__l__T_____T_T_",
];

pub const PYRAMID_D_LINES: &[&str] = &[
    "                                  _L/L",
    "                                _LT/l_l_",
    "                              _TLl/Ll__lL_",
    "                            _l_lL/_T__   _L_",
    "                          _TLl_T/_L_ <(0)> |l_",
    "                        _TL_lTl/_|__l_    _L_TL_",
    "                      _TL_Tl_l/_l_|_|__l__L___|_L_",
    "                    _lT_Ll_L_/__T__L__l__l___l__T_T_",
    "                  _L_l__ll_L/T__T____T___l___L__l_l_T_",
    "                _L_TL_l_T_l/_l__l____T__T_L___L___T___l_",
    "              _Tl_L_T_ll_L/l__l__T__L_T___T__l___T__L__TL_",
    "            _TLl_T_l_L_TL/l__l__L_T____T___T_T__L__T__L__TL_",
    "          _l_lT_Ll_T_l_L/TL_l___l_T__T___T__l__T__L__T__L__TL_",
    "        _T_T_l_lT_LT_T_/_L___l__T___l__l____T_____T__L__T__L__T_",
    "      _TLl_Tl_Ll_lT_L_/T_l__T__L_l___l__l_T__L___l__T__L__T__L_TL_",
    "    _TLL_LTl_LT_LLT__/_l__T__l___L____T__l__T_____L____T____L__T__L_",
    "  _L_lT_TL_LTL__lTl_/_T___T__L____T__T____L___T___L____T__l__L___T__T_",
    "_TTT__T_LTLT_T_LT_T/_L_T_TL____L__l__T____L__l__T___L___T__l__T_____T_T_",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlienColor {
    Blue,
    Purple,
}

impl AlienColor {
    pub fn bubble_color(self) -> Color {
        LIGHT_GREEN
    }

    pub fn pyramid_color(self) -> Color {
        match self {
            AlienColor::Blue => PYRAMID_COLOR_BLUE,
            AlienColor::Purple => PYRAMID_COLOR_PURPLE,
        }
    }

    fn tentacle_colors(self) -> &'static [Color] {
        match self {
            AlienColor::Blue => TENTACLE_COLORS_BLUE,
            AlienColor::Purple => TENTACLE_COLORS_PURPLE,
        }
    }
}

pub fn pyramid_lines(variant: u8) -> &'static [&'static str] {
    match variant {
        0 => PYRAMID_A_LINES,
        1 => PYRAMID_B_LINES,
        2 => PYRAMID_C_LINES,
        _ => PYRAMID_D_LINES,
    }
}

pub fn pyramid_canvas_w(variant: u8) -> i32 {
    pyramid_lines(variant)
        .iter()
        .map(|l| l.len() as i32)
        .max()
        .unwrap_or(0)
}

pub struct AlienTentacle {
    pub x: i32,
    pub height: usize,
    pub sway: SwayState,
    pub color: Color,
    glisten_phase: f32,
    eyes: Vec<EyeRow>,
}

impl AlienTentacle {
    pub fn new(x: i32, height: usize, color: AlienColor, rng: &mut impl RngExt) -> Self {
        let colors = color.tentacle_colors();
        let body_color = colors[rng.random_range(0..colors.len())];
        let mut eyes = Vec::new();
        let mut next_eye_height = rng.random_range(ALIEN_EYE_SPACING_MIN..=ALIEN_EYE_SPACING_MAX);
        while next_eye_height < height {
            eyes.push(EyeRow::new(next_eye_height, rng));
            next_eye_height += rng.random_range(ALIEN_EYE_SPACING_MIN..=ALIEN_EYE_SPACING_MAX);
        }
        Self {
            x,
            height,
            sway: SwayState {
                phase: rng.random::<f32>() * TAU,
            },
            color: body_color,
            glisten_phase: rng.random::<f32>() * TAU,
            eyes,
        }
    }

    fn is_eye_at(&self, row: usize) -> bool {
        self.eyes.iter().any(|e| e.height == row)
    }
}

impl Seaweed for AlienTentacle {
    fn x(&self) -> i32 {
        self.x
    }

    fn height(&self) -> usize {
        self.height
    }

    fn segment_at(&self, row: usize) -> (i32, char) {
        let x_offset = sway_x_offset(self.sway.phase, row, self.height, WAVE_SPREAD, SWAY_AMOUNT);
        if let Some(eye) = self.eyes.iter().find(|e| e.height == row) {
            let ch = if eye.blink.is_open {
                ALIEN_EYE_OPEN_CHAR
            } else {
                ALIEN_EYE_CLOSED_CHAR
            };
            return (x_offset, ch);
        }
        let ch = if x_offset > 0 {
            ')'
        } else if x_offset < 0 {
            '('
        } else {
            '|'
        };
        (x_offset, ch)
    }

    fn color_at(&self, row: usize) -> Color {
        if self.is_eye_at(row) {
            let wave = GlisteningMode::Wave.glistening_value(self.glisten_phase, row, self.height);
            if wave > ALIEN_GLISTEN_THRESHOLD {
                LIGHT_GREEN
            } else {
                self.color
            }
        } else {
            self.color
        }
    }

    fn tick(&mut self, delta_time: f32) {
        tick_sway(&mut self.sway, SWAY_SPEED);
        self.glisten_phase =
            (self.glisten_phase + ALIEN_GLISTEN_SPEED * delta_time).rem_euclid(TAU);
        for eye in &mut self.eyes {
            eye.blink.tick(delta_time);
        }
    }
}

pub struct AlienStar {
    pub x: u16,
    pub y: u16,
    pub ch: char,
    pub twinkle_lit: bool,
    pub twinkle_timer: f32,
}

impl AlienStar {
    fn new(rng: &mut impl RngExt, w: u16, h: u16) -> Self {
        let zero_row = ((h as f32 * STAR_ZERO_FRACTION) as u16).max(1);
        let u: f32 = rng.random();
        let y = (zero_row as f32 * (1.0 - (1.0 - u).sqrt())) as u16;
        let y = y.min(zero_row.saturating_sub(1));
        Self {
            x: rng.random_range(0..w),
            y,
            ch: ALIEN_STAR_CHARS[rng.random_range(0..ALIEN_STAR_CHARS.len())],
            twinkle_lit: false,
            twinkle_timer: rng.random_range(STAR_TWINKLE_DARK_MIN..STAR_TWINKLE_DARK_MAX),
        }
    }
}

fn star_target(w: u16, h: u16) -> usize {
    if w == 0 || h == 0 {
        return 0;
    }
    let zero_row = (h as f32 * STAR_ZERO_FRACTION) as usize;
    if zero_row == 0 {
        return 0;
    }
    ((w as f32 * zero_row as f32 * ALIEN_STAR_RATIO / 2.0) as usize).max(1)
}

pub struct AlienPyramid {
    pub base_x: i32,
    pub mirrored: bool,
    pub variant: u8,
    eye: Option<BlinkTimer>,
}

impl AlienPyramid {
    pub fn new(base_x: i32, mirrored: bool, variant: u8, rng: &mut impl RngExt) -> Self {
        let eye = if variant == PYRAMID_D_VARIANT {
            Some(BlinkTimer::entity_eye(rng))
        } else {
            None
        };
        Self {
            base_x,
            mirrored,
            variant,
            eye,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if let Some(eye) = &mut self.eye {
            eye.tick(dt);
        }
    }

    pub fn eye_char_for_row(&self, row_idx: usize) -> Option<char> {
        if self.variant == PYRAMID_D_VARIANT && row_idx == PYRAMID_D_EYE_ROW {
            let open = self.eye.as_ref().is_none_or(|e| e.is_open);
            Some(if open { '0' } else { '-' })
        } else {
            None
        }
    }
}

pub struct AlienBackground {
    pub color: AlienColor,
    pub tentacles: Vec<AlienTentacle>,
    pub stars: Vec<AlienStar>,
    pub pyramids: Vec<AlienPyramid>,
}

impl AlienBackground {
    pub fn new(rng: &mut impl RngExt) -> Self {
        let color = if rng.random::<bool>() {
            AlienColor::Blue
        } else {
            AlienColor::Purple
        };
        Self {
            color,
            tentacles: Vec::new(),
            stars: Vec::new(),
            pyramids: Vec::new(),
        }
    }

    pub fn init_stars(&mut self, rng: &mut impl RngExt, w: u16, h: u16) {
        let target = star_target(w, h);
        while self.stars.len() < target {
            self.stars.push(AlienStar::new(rng, w, h));
        }
    }

    pub fn tick(&mut self, dt: f32, rng: &mut impl RngExt, w: u16, h: u16) {
        if w == 0 || h == 0 {
            return;
        }
        for t in &mut self.tentacles {
            t.tick(dt);
        }
        for star in &mut self.stars {
            star.twinkle_timer -= dt;
            if star.twinkle_timer <= 0.0 {
                star.twinkle_lit = !star.twinkle_lit;
                star.twinkle_timer = if star.twinkle_lit {
                    rng.random_range(STAR_TWINKLE_LIT_MIN..STAR_TWINKLE_LIT_MAX)
                } else {
                    rng.random_range(STAR_TWINKLE_DARK_MIN..STAR_TWINKLE_DARK_MAX)
                };
            }
        }
        let target = star_target(w, h);
        while self.stars.len() < target {
            self.stars.push(AlienStar::new(rng, w, h));
        }
        for p in &mut self.pyramids {
            p.tick(dt);
        }
    }
}

pub fn extend_alien_tentacles(
    tentacles: &mut Vec<AlienTentacle>,
    to_width: i32,
    color: AlienColor,
    rng: &mut impl RngExt,
) {
    extend_spaced(
        tentacles,
        to_width,
        ALIEN_TENTACLE_WIDTH,
        ALIEN_TENTACLE_GAP_MIN..=ALIEN_TENTACLE_GAP_MAX,
        rng,
        |t| t.x,
        |x, rng| {
            AlienTentacle::new(
                x,
                rng.random_range(ALIEN_TENTACLE_HEIGHT_MIN..=ALIEN_TENTACLE_HEIGHT_MAX),
                color,
                rng,
            )
        },
    );
}

pub fn extend_alien_pyramids(
    pyramids: &mut Vec<AlienPyramid>,
    to_width: i32,
    rng: &mut impl RngExt,
) {
    let mut next_x = if pyramids.is_empty() {
        rng.random_range(ALIEN_PYRAMID_LEFT_MIN..=ALIEN_PYRAMID_LEFT_MAX)
    } else {
        let last = pyramids.last().unwrap();
        last.base_x
            + pyramid_canvas_w(last.variant)
            + rng.random_range(ALIEN_PYRAMID_GAP_MIN..=ALIEN_PYRAMID_GAP_MAX)
    };
    while next_x < to_width {
        let variant = rng.random_range(0..PYRAMID_VARIANT_COUNT);
        let mirrored = rng.random::<bool>();
        pyramids.push(AlienPyramid::new(next_x, mirrored, variant, rng));
        let canvas_w = pyramid_canvas_w(variant);
        next_x += canvas_w + rng.random_range(ALIEN_PYRAMID_GAP_MIN..=ALIEN_PYRAMID_GAP_MAX);
    }
}
