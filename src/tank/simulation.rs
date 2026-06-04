use rand::RngExt;

use crate::colors::{LIGHT_YELLOW, PINK};
use crate::entities::bubble::{Bubble, BubblePhase};
use crate::fishes::fish::{Direction, EATING_DURATION, FishState};
use crate::fishes::species::FishSpecies;
use crate::fishes::unfish::{
    PHANTOM_CROSS_TANK_CHANCE, PHANTOM_TELEPORT_MEAN, SPAWNABLE_UNFISH, VOID_SPAWN_MEAN_SECS,
};
use crate::names;
use crate::settings::Settings;
use crate::util::sample_exponential;

use super::FOOD_WEIGHT_GAIN_G;
use super::{
    BUBBLE_ZOOMIE_CHANCE, SEEK_BOOST_GROWTH, SEEK_BOOST_INITIAL_MAX, SEEK_DX_DEADZONE,
    SEEK_DY_MULTIPLIER, SEEK_NORM_MIN,
};
use super::{Tank, TankEvent, TankKind};

fn sq(x: f32) -> f32 {
    x * x
}

const CANDYFISH_SCAN_INTERVAL_TICKS: u32 = 100;

impl Tank {
    pub(super) fn spawn_bubbles(&mut self, dt: f32) {
        if matches!(self.kind, TankKind::Void) {
            return;
        }
        if self.width == 0 || self.height == 0 {
            return;
        }
        let mut rng = rand::rng();

        let alien_bubble_color = self.alien_bg.as_ref().map(|bg| bg.color.bubble_color());
        let desert_bubble_color = self.desert_bg.as_ref().map(|bg| bg.bubble_color());
        let bubble_color = alien_bubble_color
            .or(desert_bubble_color)
            .unwrap_or(self.kind.config().bubble_color);

        let new_bubbles =
            self.bubble_spawner
                .tick(dt, self.width, self.height, bubble_color, &mut rng);
        self.bubbles.extend(new_bubbles);

        for fish in &self.fish {
            if !matches!(fish.state, FishState::Zoomie { .. }) {
                continue;
            }
            if rng.random::<f32>() < BUBBLE_ZOOMIE_CHANCE {
                let tail_x = match fish.facing {
                    Direction::Left => fish.position.x + fish.display_width as f32 - 1.0,
                    Direction::Right => fish.position.x,
                };
                let bubble = match fish.species {
                    FishSpecies::Goldenfish => {
                        let mut b = Bubble::new(
                            tail_x,
                            fish.position.y,
                            BubblePhase::rising(&mut rng),
                            LIGHT_YELLOW,
                            Some('☆'),
                            &mut rng,
                        );
                        b.cash_value = Some(5);
                        b
                    }
                    FishSpecies::Mutantfish => Bubble::new(
                        tail_x,
                        fish.position.y,
                        BubblePhase::rising(&mut rng),
                        fish.color,
                        Some('X'),
                        &mut rng,
                    ),
                    FishSpecies::Candyfish => Bubble::new(
                        tail_x,
                        fish.position.y,
                        BubblePhase::rising(&mut rng),
                        PINK,
                        None,
                        &mut rng,
                    ),
                    _ => Bubble::new(
                        tail_x,
                        fish.position.y,
                        BubblePhase::rising(&mut rng),
                        bubble_color,
                        None,
                        &mut rng,
                    ),
                };
                self.bubbles.push(bubble);
            }
        }
    }

    pub(super) fn steer_seeking_fish(&mut self) {
        for i in 0..self.fish.len() {
            let (idx, approach_right) = match self.fish[i].state {
                FishState::SeekingFood(idx, ar) => (idx, ar),
                _ => continue,
            };
            if idx >= self.food.len() {
                self.fish[i].cancel_seek();
                continue;
            }

            self.fish[i].seek_boost += SEEK_BOOST_GROWTH / (1.0 + self.fish[i].seek_boost);

            let food_x = self.food[idx].position.x;
            let food_y = self.food[idx].position.y;
            let fish_len = self.fish[i].display_width as f32;
            let fish_y = self.fish[i].position.y;

            let target_x = if approach_right {
                food_x - (fish_len - 1.0)
            } else {
                food_x
            };

            let raw_dx = target_x - self.fish[i].position.x;
            let dx = if raw_dx.abs() < SEEK_DX_DEADZONE {
                0.0
            } else {
                raw_dx
            };
            let raw_dy = food_y - fish_y;
            let dy = if self.food[idx].settled && fish_y as i32 >= food_y as i32 {
                0.0
            } else {
                raw_dy
            };
            let norm = (dx * dx + dy * dy).sqrt().max(SEEK_NORM_MIN);
            let seek_speed = self.fish[i].speed + self.fish[i].seek_boost;
            self.fish[i].velocity.dx = (dx / norm) * seek_speed;
            self.fish[i].velocity.dy = (dy / norm) * seek_speed * SEEK_DY_MULTIPLIER;
            self.fish[i].facing = if approach_right {
                Direction::Right
            } else {
                Direction::Left
            };
        }
    }

    pub(super) fn tick_fish(&mut self, settings: &Settings, coffee: u32) {
        for fish in &mut self.fish {
            fish.tick(settings, self.width, self.height, coffee);
        }
    }

    pub(super) fn check_eating_collisions(&mut self) {
        for i in 0..self.fish.len() {
            let idx = match self.fish[i].state {
                FishState::SeekingFood(idx, _) => idx,
                _ => continue,
            };
            if idx >= self.food.len() || self.food[idx].eaten {
                continue;
            }
            let head_x = self.fish[i].head_x();
            let head_y = self.fish[i].position.y as i32;
            let fx = self.food[idx].position.x as i32;
            let fy = self.food[idx].position.y as i32;
            if (head_x - fx).abs() <= 1 && head_y == fy {
                if self.fish[i].species == FishSpecies::Candyfish {
                    self.food[idx].is_candy = true;
                    self.fish[i].state = FishState::Eating {
                        time_remaining: EATING_DURATION,
                    };
                    continue;
                }
                self.food[idx].eaten = true;
                self.fish[i].state = FishState::Eating {
                    time_remaining: EATING_DURATION,
                };
                let cat = self.fish[i].size_category;
                let cap = if self.fish[i].is_weight_uncapped() {
                    0
                } else {
                    self.fish[i].species.config().weight_cap[cat as usize]
                };
                let gain = if self.food[idx].is_candy {
                    FOOD_WEIGHT_GAIN_G * 10
                } else {
                    FOOD_WEIGHT_GAIN_G
                };
                let new_w = self.fish[i].weight_g + gain;
                self.fish[i].weight_g = if cap == 0 { new_w } else { new_w.min(cap) };
            }
        }
    }

    pub(super) fn assign_food_to_idle_fish(&mut self) {
        if self.food.is_empty() {
            return;
        }
        for i in 0..self.fish.len() {
            if !matches!(
                self.fish[i].state,
                FishState::Idle | FishState::Zoomie { .. }
            ) {
                continue;
            }
            let fish_len = self.fish[i].display_width as f32;
            let head_x = self.fish[i].head_x() as f32;
            let fish_y = self.fish[i].position.y;
            let nearest_idx = self.food.iter().enumerate().min_by(|(_, a), (_, b)| {
                let da = sq(head_x - a.position.x) + sq(fish_y - a.position.y);
                let db = sq(head_x - b.position.x) + sq(fish_y - b.position.y);
                da.partial_cmp(&db).unwrap()
            });
            if let Some((idx, _)) = nearest_idx {
                let mut rng = rand::rng();
                let center_x = self.fish[i].position.x + (fish_len - 1.0) / 2.0;
                let food_x = self.food[idx].position.x;
                let approach_right = center_x <= food_x;
                self.fish[i].seek_boost = rng.random_range(0.0_f32..SEEK_BOOST_INITIAL_MAX);
                self.fish[i].facing = if approach_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                self.fish[i].state = FishState::SeekingFood(idx, approach_right);
            }
        }
    }

    pub(super) fn tick_candyfish_effects(&mut self) {
        let candyfish_bounds: Vec<(f32, f32, f32)> = self
            .fish
            .iter()
            .filter(|f| f.species == FishSpecies::Candyfish)
            .map(|cf| {
                (
                    cf.position.x,
                    cf.position.x + cf.display_width as f32,
                    cf.position.y,
                )
            })
            .collect();
        if candyfish_bounds.is_empty() {
            return;
        }
        for food in &mut self.food {
            if food.is_candy {
                continue;
            }
            let fx = food.position.x;
            let fy = food.position.y;
            for &(cx1, cx2, cy) in &candyfish_bounds {
                if fx >= cx1 - 1.0 && fx <= cx2 + 1.0 && (fy - cy).abs() <= 1.0 {
                    food.is_candy = true;
                    break;
                }
            }
        }
        if !self
            .candy_tick
            .is_multiple_of(CANDYFISH_SCAN_INTERVAL_TICKS)
        {
            return;
        }
        for i in 0..self.fish.len() {
            if self.fish[i].species == FishSpecies::Candyfish {
                continue;
            }
            let fx1 = self.fish[i].position.x;
            let fx2 = fx1 + self.fish[i].display_width as f32;
            let fy = self.fish[i].position.y;
            for &(cx1, cx2, cy) in &candyfish_bounds {
                if fx1 < cx2 && fx2 > cx1 && (fy - cy).abs() <= 1.0 {
                    self.fish[i].weight_g += 1;
                    break;
                }
            }
        }
    }

    pub fn spawn_unfish(&mut self, rng: &mut impl RngExt) {
        if self.is_full() {
            return;
        }
        let kind = SPAWNABLE_UNFISH[rng.random_range(0..SPAWNABLE_UNFISH.len())];
        let actual_name = if kind == crate::fishes::unfish::UnfishKind::Doppleganger {
            names::unique_name_in(&self.used_names, "Doppleganger")
        } else {
            names::unique_roman_in(&self.used_names)
        };
        let x_max = (self.width as f32 - 15.0).max(6.0);
        let y_max = (self.height as f32 - 5.0).max(3.0);
        let x = rng.random_range(5.0_f32..x_max);
        let y = rng.random_range(3.0_f32..y_max);
        let fish = crate::fishes::fish::Fish::new_unfish(kind, actual_name.clone(), x, y, rng);
        self.used_names.insert(actual_name);
        self.fish.push(fish);
    }

    pub(super) fn tick_void_spawn(&mut self, dt: f32, rng: &mut impl RngExt) {
        self.void_spawn_timer -= dt;
        if self.void_spawn_timer <= 0.0 {
            self.void_spawn_timer = sample_exponential(rng, VOID_SPAWN_MEAN_SECS);
            self.spawn_unfish(rng);
        }
    }

    pub(super) fn tick_dopplegangers(&mut self) {
        let doppleganger_idx = self.fish.iter().position(|f| {
            f.unfish_state.as_ref().is_some_and(|us| {
                us.kind == crate::fishes::unfish::UnfishKind::Doppleganger
                    && !us.doppleganger_cloned
            })
        });
        let Some(d_idx) = doppleganger_idx else {
            return;
        };

        let target_idx = self.fish.iter().position(|f| f.unfish_state.is_none());
        let Some(t_idx) = target_idx else { return };

        let target_name = self.fish[t_idx].name.clone();
        let target_clone = self.fish[t_idx].clone();

        let old_unfish_state = self.fish[d_idx].unfish_state.take();
        let old_weight = self.fish[d_idx].weight_g;

        self.fish[d_idx] = target_clone;
        self.fish[d_idx].weight_g = old_weight;
        self.fish[d_idx].unfish_state = old_unfish_state;
        if let Some(ref mut us) = self.fish[d_idx].unfish_state {
            us.doppleganger_cloned = true;
        }

        let new_name = names::unique_name_in(&self.used_names, &format!("{}?", target_name));
        self.used_names.insert(new_name.clone());
        self.fish[d_idx].name = new_name;
    }

    pub(super) fn tick_phantoms(&mut self, dt: f32, rng: &mut impl RngExt) -> Vec<TankEvent> {
        let mut events = Vec::new();
        for fish in &mut self.fish {
            let Some(ref mut us) = fish.unfish_state else {
                continue;
            };
            if us.kind != crate::fishes::unfish::UnfishKind::Phantom {
                continue;
            }
            us.phantom_timer -= dt;
            if us.phantom_timer <= 0.0 {
                us.phantom_timer = sample_exponential(rng, PHANTOM_TELEPORT_MEAN);
                if rng.random::<f32>() < PHANTOM_CROSS_TANK_CHANCE {
                    events.push(TankEvent::PhantomCrossTank {
                        fish_name: fish.name.clone(),
                    });
                } else {
                    let max_x = (self.width as f32 - fish.display_width as f32).max(0.0);
                    let max_y = (self.height as f32 - 1.0).max(0.0);
                    fish.position.x = rng.random_range(0.0..=max_x);
                    fish.position.y = rng.random_range(0.0..=max_y);
                }
            }
        }
        events
    }
}
