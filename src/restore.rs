use crate::colors::DARK_GRAY;
use crate::entities::cow::{Cow, cow_default_sway_speed, cow_display_width};
use crate::fishes::fish::{Fish, compute_display_width};
use crate::fishes::species::{BodyTemplate, FishSpecies};

pub trait Restorable {
    fn restore(&mut self);
}

impl Restorable for Fish {
    fn restore(&mut self) {
        self.mutant = None;
        self.mutations = None;
        self.devil_marked = false;
        self.sell_price_bonus_pct = 0;
        let config = self.species.config();
        self.sway_speed = config.sway_speed;
        if !matches!(config.body, BodyTemplate::Fixed { .. }) {
            self.body_size = config.sizes[self.size_category as usize];
        }
        if self.species == FishSpecies::Mutantfish {
            self.color = FishSpecies::mutant_color_for_seed(self.pattern_seed);
        } else if !config.palette.is_empty() {
            self.color = config.palette[0];
        }
        self.display_width = compute_display_width(self.species, self.body_size);
    }
}

impl Restorable for Cow {
    fn restore(&mut self) {
        self.mutant.color_patches.clear();
        self.mutant.glistening_color = None;
        self.mutant.is_double = false;
        self.mutant.double_head_eyes.clear();
        self.mutant.eye_color = Some(DARK_GRAY);
        self.color = self.variant.body_color();
        self.body_length = 0;
        self.sway_speed = cow_default_sway_speed();
        self.mutations = None;
        self.display_width = cow_display_width(self);
    }
}
