use crate::fishes::fish::Fish;
use crate::names;
use crate::tank::{Afterlife, Exile, Tank, TankKind, WorldSignal};

use super::App;

impl App {
    pub(super) fn bury(&mut self, fish: Fish) {
        let wall = match Afterlife::of(&fish) {
            Afterlife::Blessed => Some(self.ensure_heaven()),
            Afterlife::Damned => self.afterlife_tank(Afterlife::Damned),
        };
        if let Some(tank) = wall {
            self.tanks[tank].receive_soul(fish.clone());
        }
        self.graveyard.push(fish);
    }

    fn afterlife_tank(&self, afterlife: Afterlife) -> Option<usize> {
        self.tanks
            .iter()
            .position(|tank| tank.kind.config().afterlife == Some(afterlife))
    }

    pub(super) fn gather_the_dead(&self, tank: &mut Tank) {
        let Some(afterlife) = tank.kind.config().afterlife else {
            return;
        };
        if self.afterlife_tank(afterlife).is_some() {
            return;
        }
        for grave in self
            .graveyard
            .iter()
            .filter(|grave| Afterlife::of(grave) == afterlife)
        {
            tank.receive_soul(grave.clone());
        }
    }

    pub(super) fn demolish_tank(&mut self, index: usize) {
        let mut tank = self.tanks.remove(index);
        self.used_tank_names.remove(&tank.name);
        if self.current_tank >= index && self.current_tank > 0 {
            self.current_tank -= 1;
        }
        let Some(afterlife) = tank.kind.config().afterlife else {
            return;
        };
        let Some(heir) = self.afterlife_tank(afterlife) else {
            return;
        };
        for soul in tank.take_souls() {
            self.tanks[heir].receive_soul(soul);
        }
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
        if let Some(heaven) = self.afterlife_tank(Afterlife::Blessed) {
            return heaven;
        }
        let name = names::unique_name_in(&self.used_tank_names, TankKind::Heaven.display_name());
        self.used_tank_names.insert(name.clone());
        let mut heaven = Tank::new(name, TankKind::Heaven, &[]);
        heaven.resize(self.terminal_width, self.tank_height(), &[]);
        self.found_tank(heaven)
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
        tank.kind.config().sellable && tank.fish.is_empty() && another_home
    }
}
