pub const DEFAULT_FPS: f32 = 30.0;
pub const FPS_MIN: f32 = 1.0;
pub const FPS_MAX: f32 = 120.0;

pub struct Settings {
    pub fps: f32,
    pub cursor_blink: bool,
    pub show_names: bool,
    pub show_stats: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: DEFAULT_FPS,
            cursor_blink: true,
            show_names: false,
            show_stats: true,
        }
    }
}
