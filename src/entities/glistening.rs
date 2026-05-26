use std::f32::consts::PI;

use rand::RngExt;
use ratatui::style::Color;

const CHAR_SPREAD: f32 = 1.0;
const GLISTEN_BASE_FACTOR: f32 = 0.70;
const GLISTEN_PEAK_FACTOR: f32 = 0.65;
pub const GLISTEN_PEAK_THRESHOLD: f32 = 0.6;
pub const GLISTEN_MID_THRESHOLD: f32 = 0.1;

#[derive(Clone, Copy, PartialEq)]
pub enum GlisteningMode {
    Wave,
    FullGlow,
    HalfHalf,
    CenterOut,
    OutsideIn,
}

impl GlisteningMode {
    pub fn random_other(self, rng: &mut impl RngExt) -> Self {
        const ALL: [GlisteningMode; 5] = [
            GlisteningMode::Wave,
            GlisteningMode::FullGlow,
            GlisteningMode::HalfHalf,
            GlisteningMode::CenterOut,
            GlisteningMode::OutsideIn,
        ];
        loop {
            let m = ALL[rng.random_range(0..ALL.len())];
            if m != self {
                return m;
            }
        }
    }

    pub fn glistening_value(self, phase: f32, i: usize, total: usize) -> f32 {
        let center = total as f32 / 2.0;
        let dist = (i as f32 - center).abs();
        match self {
            GlisteningMode::Wave     => (phase - i as f32 * CHAR_SPREAD).sin(),
            GlisteningMode::FullGlow => phase.sin(),
            GlisteningMode::HalfHalf => {
                if i < total / 2 { phase.sin() } else { (phase + PI).sin() }
            }
            GlisteningMode::CenterOut  => (phase - dist * CHAR_SPREAD).sin(),
            GlisteningMode::OutsideIn  => (phase + dist * CHAR_SPREAD).sin(),
        }
    }
}

pub fn derive_glistening_palette(color: Color) -> (Color, Color, Color) {
    match color {
        Color::Rgb(r, g, b) => {
            let base = Color::Rgb(
                (r as f32 * GLISTEN_BASE_FACTOR) as u8,
                (g as f32 * GLISTEN_BASE_FACTOR) as u8,
                (b as f32 * GLISTEN_BASE_FACTOR) as u8,
            );
            let peak = Color::Rgb(
                r.saturating_add(((255u16 - r as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
                g.saturating_add(((255u16 - g as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
                b.saturating_add(((255u16 - b as u16) as f32 * GLISTEN_PEAK_FACTOR) as u8),
            );
            (base, color, peak)
        }
        _ => (Color::DarkGray, color, Color::White),
    }
}

pub fn color_for_glisten(
    mode: GlisteningMode,
    phase: f32,
    i: usize,
    total: usize,
    base: Color,
    mid: Color,
    peak: Color,
) -> Color {
    let v = mode.glistening_value(phase, i, total);
    if v > GLISTEN_PEAK_THRESHOLD {
        peak
    } else if v > GLISTEN_MID_THRESHOLD {
        mid
    } else {
        base
    }
}
