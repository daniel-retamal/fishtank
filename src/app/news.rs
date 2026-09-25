use crate::update::UPDATE_LABEL;

use super::App;

impl App {
    pub fn announce_update(&mut self) {
        self.newer_release = true;
    }

    pub(super) fn update_label(&self) -> Option<&'static str> {
        self.newer_release.then_some(UPDATE_LABEL)
    }
}
