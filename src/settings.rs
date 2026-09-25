use serde::{Deserialize, Serialize};
pub const DEFAULT_FPS: f32 = 30.0;
pub const FPS_MIN: f32 = 1.0;
pub const FPS_MAX: f32 = 120.0;

pub const DEFAULT_STAGES_PER_TICK: u32 = 1;
pub const STAGES_PER_TICK_MIN: u32 = 1;
pub const STAGES_PER_TICK_MAX: u32 = 100_000;

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fps: f32,
    pub stages_per_tick: u32,
    pub cursor_blink: bool,
    pub show_names: bool,
    pub show_nets: bool,
    pub show_stats: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: DEFAULT_FPS,
            stages_per_tick: DEFAULT_STAGES_PER_TICK,
            cursor_blink: true,
            show_names: false,
            show_nets: false,
            show_stats: true,
        }
    }
}
