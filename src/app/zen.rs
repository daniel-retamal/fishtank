use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::ui::input_action::{InputAction, classify};

use super::App;

impl App {
    pub fn in_zen(&self) -> bool {
        self.zen
    }

    pub(super) fn enter_zen(&mut self) -> bool {
        self.close_overlay();
        self.editor.clear();
        self.zen = true;
        true
    }

    pub(super) fn wake_from_zen(&mut self, event: &Event) {
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press || matches!(key.code, KeyCode::Modifier(_)) {
            return;
        }
        if matches!(classify(event), Some(InputAction::Quit)) {
            self.running = false;
            return;
        }
        self.zen = false;
    }
}
