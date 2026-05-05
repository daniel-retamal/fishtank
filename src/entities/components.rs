use std::f32::consts::TAU;

#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Clone)]
pub struct SwayState {
    pub phase: f32,
}

pub fn tick_sway(sway: &mut SwayState, sway_speed: f32) {
    sway.phase = (sway.phase + sway_speed).rem_euclid(TAU);
}
