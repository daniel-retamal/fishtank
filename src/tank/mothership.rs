use rand::RngExt;
use ratatui::style::Color;

use super::{Tank, TankEvent, ufo_mean_secs};
use crate::colors::LIGHT_GREEN;
use crate::util::exponential_event;

pub const ALIEN_TONGUE: &str = "GLORP VLERP!";
pub const ALIEN_INK: Color = LIGHT_GREEN;
pub const ALIEN_SOUNDS: &[&str] = &[
    "Glerp", "Glurp", "Glorp", "Vlerp", "Bzzs", "Gloop", "Vlorg", "Meep",
];
pub const ALIEN_NAME_WORDS_MIN: usize = 1;
pub const ALIEN_NAME_WORDS_MAX: usize = 3;

pub fn alien_name(rng: &mut impl RngExt) -> String {
    let words = rng.random_range(ALIEN_NAME_WORDS_MIN..=ALIEN_NAME_WORDS_MAX);
    (0..words)
        .map(|_| ALIEN_SOUNDS[rng.random_range(0..ALIEN_SOUNDS.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn speech_ink(alienated: bool, frame: Color) -> Color {
    if alienated { ALIEN_INK } else { frame }
}

#[derive(Clone, Copy)]
enum Caller {
    Fish(usize),
    Cow(usize),
}

impl Tank {
    pub(super) fn tick_calls_home(
        &mut self,
        dt: f32,
        rng: &mut impl RngExt,
        events: &mut Vec<TankEvent>,
    ) {
        let mean = ufo_mean_secs(self.kind);
        for caller in self.callers() {
            if exponential_event(rng, mean, dt) {
                self.call_home(caller);
                events.push(TankEvent::CallHome);
            }
        }
    }

    pub fn call_home_now(&mut self, rng: &mut impl RngExt) -> bool {
        let callers = self.callers();
        if callers.is_empty() {
            return false;
        }
        self.call_home(callers[rng.random_range(0..callers.len())]);
        true
    }

    pub fn answers_call_home(&self) -> bool {
        self.fish.len() + self.incoming_fish() < self.capacity()
    }

    fn callers(&self) -> Vec<Caller> {
        let fish = self
            .fish
            .iter()
            .enumerate()
            .filter(|(_, fish)| fish.is_alienated())
            .map(|(index, _)| Caller::Fish(index));
        let cows = self
            .cows
            .iter()
            .enumerate()
            .filter(|(_, cow)| cow.is_alienated() && cow.can_speak())
            .map(|(index, _)| Caller::Cow(index));
        fish.chain(cows).collect()
    }

    fn call_home(&mut self, caller: Caller) {
        match caller {
            Caller::Fish(index) => {
                self.fish[index].say(ALIEN_TONGUE.to_string());
            }
            Caller::Cow(index) => self.cows[index].say(ALIEN_TONGUE.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::mutations::{Mutation, apply_mutation};
    use crate::fishes::species::FishSpecies;
    use crate::tank::TankKind;

    const TANK_W: u16 = 80;
    const TANK_H: u16 = 24;
    const A_LIFETIME_SECS: f32 = 1.0e9;

    fn tank_with_one_fish(alien: bool) -> Tank {
        let mut rng = rand::rng();
        let mut tank = Tank::new("Test".to_string(), TankKind::Base, &[]);
        tank.resize(TANK_W, TANK_H, &[]);
        tank.spawn_fish(FishSpecies::Merluza, "Zed".to_string(), &mut rng);
        if alien {
            apply_mutation(&mut tank.fish[0], Mutation::Alienation, &mut rng);
        }
        tank
    }

    fn calls_in_a_lifetime(tank: &mut Tank) -> usize {
        let mut events = Vec::new();
        tank.tick_calls_home(A_LIFETIME_SECS, &mut rand::rng(), &mut events);
        events
            .iter()
            .filter(|event| matches!(event, TankEvent::CallHome))
            .count()
    }

    #[test]
    fn an_alien_calls_home_on_its_own_clock() {
        let mut tank = tank_with_one_fish(true);
        assert_eq!(calls_in_a_lifetime(&mut tank), 1);
        let bubble = tank.fish[0].speech.as_ref().map(|b| b.text.as_str());
        assert_eq!(bubble, Some(ALIEN_TONGUE));
    }

    #[test]
    fn an_earthly_fish_never_calls_home() {
        let mut tank = tank_with_one_fish(false);
        assert_eq!(calls_in_a_lifetime(&mut tank), 0);
        assert!(tank.fish[0].speech.is_none());
    }

    #[test]
    fn a_restored_alien_forgets_the_way_home() {
        use crate::restore::Restorable;
        let mut tank = tank_with_one_fish(true);
        tank.fish[0].restore();
        assert_eq!(calls_in_a_lifetime(&mut tank), 0);
    }
}
