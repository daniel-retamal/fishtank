use crate::fishes::fish::Fish;
use crate::names;
use crate::tank::{Exile, Tank, TankKind, WorldSignal};

use super::App;

impl App {
    pub(super) fn bury(&mut self, fish: Fish) {
        if !fish.devil_marked {
            let heaven = self.ensure_heaven();
            self.tanks[heaven].receive_soul(fish.clone());
        }
        self.graveyard.push(fish);
    }

    pub(super) fn exhume(&mut self, grave: usize) -> Fish {
        let fish = self.graveyard.remove(grave);
        for tank in &mut self.tanks {
            tank.release_soul(&fish.name);
            tank.clear_grave_name(&fish.name);
        }
        fish
    }

    pub(super) fn kill_fish(&mut self, name: &str) -> bool {
        let Some((tank, index)) = self.fish_location(name) else {
            return false;
        };
        let fish = self.tanks[tank].take_fish(index);
        self.tanks[tank].signal(WorldSignal::Death);
        self.bury(fish);
        true
    }

    fn ensure_heaven(&mut self) -> usize {
        if let Some(heaven) = self.tanks.iter().position(|t| t.kind == TankKind::Heaven) {
            return heaven;
        }
        let name = names::unique_name_in(&self.used_tank_names, TankKind::Heaven.display_name());
        self.used_tank_names.insert(name.clone());
        let mut heaven = Tank::new(name, TankKind::Heaven, &[]);
        heaven.resize(self.terminal_width, self.tank_height(), &[]);
        self.tanks.push(heaven);
        self.tanks.len() - 1
    }

    pub(super) fn landing_tank(&self, from: usize, welcomes: impl Fn(&Tank) -> bool) -> usize {
        let count = self.tanks.len();
        let onwards = || (0..count).map(|step| (from + step) % count);
        onwards()
            .find(|&i| welcomes(&self.tanks[i]) && !self.tanks[i].is_full())
            .or_else(|| onwards().find(|&i| welcomes(&self.tanks[i])))
            .unwrap_or(from)
    }

    pub(super) fn land_fish(&mut self, from: usize, fish: Fish, name: String) -> usize {
        let to = self.landing_tank(from, |tank| tank.welcomes(&fish));
        self.tanks[to].place_fish(fish, name, &mut rand::rng());
        to
    }

    pub(super) fn settle_exiles(&mut self) {
        for from in 0..self.tanks.len() {
            for exile in std::mem::take(&mut self.tanks[from].pending_exiles) {
                match exile {
                    Exile::Fish(fish) => {
                        let name = fish.name.clone();
                        self.land_fish(from, *fish, name);
                    }
                    Exile::Cow(cow) => {
                        let to = self.landing_tank(from, Tank::welcomes_cows);
                        self.tanks[to].place_cow(*cow, &mut rand::rng());
                    }
                }
            }
        }
    }

    pub(super) fn can_sell_tank(&self, index: usize) -> bool {
        let tank = &self.tanks[index];
        let another_home = self
            .tanks
            .iter()
            .enumerate()
            .any(|(i, other)| i != index && !other.kind.config().holy_only);
        !tank.kind.config().unique && tank.fish.is_empty() && another_home
    }
}
