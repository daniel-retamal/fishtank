use rand::RngExt;

use crate::colors::{LIGHT_YELLOW, PINK};
use crate::entities::bubble::{Bubble, BubblePhase};
use crate::entities::cow::Cow;
use crate::fishes::fish::{Direction, EATING_DURATION, Fish, FishState};
use crate::fishes::mutations::{MutantBacked, Mutatable, Mutation, apply_mutation};
use crate::fishes::species::FishSpecies;
use crate::fishes::unfish::{
    PHANTOM_CROSS_TANK_CHANCE, PHANTOM_TELEPORT_MEAN, SPAWNABLE_UNFISH, UnfishKind,
    VOID_SPAWN_MEAN_SECS, is_multi_row,
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
const GOLDENFISH_ZOOMIE_CASH: u32 = 5;
const CANDYFISH_TOUCH_WEIGHT_G: u32 = 1;
const ENGULF_REACH: f32 = 3.0;
const ENGULF_FISH_Y_TOLERANCE: f32 = 2.0;
const ENGULF_COW_Y_TOLERANCE: f32 = 5.0;

fn horizontally_near(ax: f32, aw: usize, bx: f32, bw: usize) -> bool {
    let ax2 = ax + aw as f32;
    let bx2 = bx + bw as f32;
    ax <= bx2 + ENGULF_REACH && bx <= ax2 + ENGULF_REACH
}

fn is_worm(fish: &Fish) -> bool {
    fish.unfish_state
        .as_ref()
        .is_some_and(|us| us.kind == UnfishKind::Worm)
}

fn engulf_compatible(receiver: &Fish, candidate: &Fish) -> bool {
    let receiver_is_worm = is_worm(receiver);
    match candidate.unfish_state.as_ref() {
        Some(us) if is_multi_row(us.kind) => false,
        Some(us) if us.kind == UnfishKind::Worm => receiver_is_worm,
        _ => !receiver_is_worm,
    }
}

fn fish_near(a: &Fish, b: &Fish) -> bool {
    horizontally_near(a.position.x, a.display_width, b.position.x, b.display_width)
        && (a.position.y - b.position.y).abs() <= ENGULF_FISH_Y_TOLERANCE
}

fn cow_near(a: &Cow, b: &Cow) -> bool {
    horizontally_near(a.position.x, a.display_width, b.position.x, b.display_width)
        && (a.position.y - b.position.y).abs() <= ENGULF_COW_Y_TOLERANCE
}

impl Tank {
    pub(super) fn spawn_bubbles(&mut self, dt: f32) {
        if matches!(self.kind, TankKind::Void) {
            return;
        }
        if self.width == 0 || self.height == 0 {
            return;
        }
        let mut rng = rand::rng();

        let bubble_color = self
            .background
            .bubble_color_override()
            .unwrap_or(self.kind.config().bubble_color);

        let spawn_dt = dt * self.kind.config().bubble_rate_mult;
        let new_bubbles =
            self.bubble_spawner
                .tick(spawn_dt, self.width, self.height, bubble_color, &mut rng);
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
                let golden_stacks = fish.ability_stacks(FishSpecies::Goldenfish);
                let mut bubble = if golden_stacks > 0 {
                    let mut b = Bubble::new(
                        tail_x,
                        fish.position.y,
                        BubblePhase::rising(&mut rng),
                        LIGHT_YELLOW,
                        Some('☆'),
                        &mut rng,
                    );
                    b.cash_value = Some(GOLDENFISH_ZOOMIE_CASH * golden_stacks);
                    b
                } else {
                    match fish.species {
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
                    }
                };
                if let Some(c) = fish.bubble_color() {
                    bubble.color = c;
                }
                self.bubbles.push(bubble);
            }
        }
    }

    pub(super) fn steer_seeking_fish(&mut self) {
        for i in 0..self.fish.len() {
            let (idx, approach_right) = match self.fish[i].state {
                FishState::SeekingFood {
                    food_idx,
                    approach_right,
                } => (food_idx, approach_right),
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
                FishState::SeekingFood { food_idx, .. } => food_idx,
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
                self.fish[i].state = FishState::SeekingFood {
                    food_idx: idx,
                    approach_right,
                };
            }
        }
    }

    pub(super) fn tick_candyfish_effects(&mut self) {
        let candyfish_bounds: Vec<(f32, f32, f32, u32)> = self
            .fish
            .iter()
            .filter(|f| f.ability_stacks(FishSpecies::Candyfish) > 0)
            .map(|cf| {
                (
                    cf.position.x,
                    cf.position.x + cf.display_width as f32,
                    cf.position.y,
                    cf.ability_stacks(FishSpecies::Candyfish),
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
            for &(cx1, cx2, cy, _) in &candyfish_bounds {
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
            if self.fish[i].ability_stacks(FishSpecies::Candyfish) > 0 {
                continue;
            }
            let fx1 = self.fish[i].position.x;
            let fx2 = fx1 + self.fish[i].display_width as f32;
            let fy = self.fish[i].position.y;
            for &(cx1, cx2, cy, stacks) in &candyfish_bounds {
                if fx1 < cx2 && fx2 > cx1 && (fy - cy).abs() <= 1.0 {
                    self.fish[i].weight_g += CANDYFISH_TOUCH_WEIGHT_G * stacks;
                    break;
                }
            }
        }
    }

    pub(super) fn tick_engulfment(&mut self) {
        if let Some((receiver, engulfed)) = self.find_engulf_pair_fish() {
            self.fuse_fish(receiver, engulfed);
        }
        if let Some((receiver, engulfed)) = self.find_engulf_pair_cow() {
            self.fuse_cow(receiver, engulfed);
        }
    }

    fn find_engulf_pair_fish(&self) -> Option<(usize, usize)> {
        for r in 0..self.fish.len() {
            if self.fish[r].engulf_timer <= 0.0 || self.fish[r].is_double() {
                continue;
            }
            for e in 0..self.fish.len() {
                if e == r || self.fish[e].is_double() {
                    continue;
                }
                if engulf_compatible(&self.fish[r], &self.fish[e])
                    && fish_near(&self.fish[r], &self.fish[e])
                {
                    return Some((r, e));
                }
            }
        }
        None
    }

    fn find_engulf_pair_cow(&self) -> Option<(usize, usize)> {
        for r in 0..self.cows.len() {
            if self.cows[r].engulf_timer <= 0.0 || self.cows[r].mutant.is_double {
                continue;
            }
            for e in 0..self.cows.len() {
                if e == r || self.cows[e].mutant.is_double {
                    continue;
                }
                if cow_near(&self.cows[r], &self.cows[e]) {
                    return Some((r, e));
                }
            }
        }
        None
    }

    fn fuse_fish(&mut self, receiver: usize, engulfed: usize) {
        let mut rng = rand::rng();
        let receiver_snapshot = self.fish[receiver].clone();
        let engulfed_snapshot = self.fish[engulfed].clone();
        let receiver_component = self.fish[receiver]
            .fused_self_component()
            .with_snapshot(receiver_snapshot);
        let mut engulfed_component = self.fish[engulfed]
            .fused_self_component()
            .with_snapshot(engulfed_snapshot);
        engulfed_component.persona = self.fish[engulfed].capture_persona();
        let engulfed_name = self.fish[engulfed].name.clone();
        let receiver_weight = self.fish[receiver].weight_g;
        let engulfed_weight = self.fish[engulfed].weight_g;

        apply_mutation(&mut self.fish[receiver], Mutation::Telophase, &mut rng);
        {
            let fish = &mut self.fish[receiver];
            fish.set_fused(vec![receiver_component, engulfed_component]);
            fish.name = format!("{} / {}", fish.name, engulfed_name);
            fish.weight_g = (receiver_weight + engulfed_weight) / 2;
            fish.engulf_timer = 0.0;
            fish.recompute_display_width();
        }
        self.fish.remove(engulfed);
    }

    fn fuse_cow(&mut self, receiver: usize, engulfed: usize) {
        let mut rng = rand::rng();
        let receiver_component = self.cows[receiver]
            .self_component()
            .with_cow_snapshot(self.cows[receiver].clone());
        let engulfed_component = self.cows[engulfed]
            .self_component()
            .with_cow_snapshot(self.cows[engulfed].clone());
        let engulfed_name = self.cows[engulfed].name.clone();

        apply_mutation(&mut self.cows[receiver], Mutation::Telophase, &mut rng);
        {
            let cow = &mut self.cows[receiver];
            cow.mutant.fused = vec![receiver_component, engulfed_component];
            cow.name = format!("{} / {}", cow.name, engulfed_name);
            cow.engulf_timer = 0.0;
            cow.recompute_display_width();
        }
        self.cows.remove(engulfed);
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
        let mut fish = crate::fishes::fish::Fish::new_unfish(kind, actual_name.clone(), x, y, rng);
        self.mark_if_hell(&mut fish);
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
        self.tick_standalone_doppleganger();
        self.tick_fused_doppleganger();
    }

    fn tick_standalone_doppleganger(&mut self) {
        let doppleganger_idx = self.fish.iter().position(|f| {
            f.unfish_state
                .as_ref()
                .is_some_and(|us| us.kind == UnfishKind::Doppleganger && !us.doppleganger_cloned)
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

    fn tick_fused_doppleganger(&mut self) {
        let host_idx = self.fish.iter().position(|f| {
            f.unfish_state.is_none()
                && f.fused_components().iter().any(|c| {
                    c.persona.as_deref().is_some_and(|us| {
                        us.kind == UnfishKind::Doppleganger && !us.doppleganger_cloned
                    })
                })
        });
        let Some(h_idx) = host_idx else { return };

        let target_name = self
            .fish
            .iter()
            .enumerate()
            .find(|(i, f)| *i != h_idx && f.unfish_state.is_none())
            .map(|(_, f)| f.name.clone());
        let Some(target_name) = target_name else {
            return;
        };

        let new_name = names::unique_name_in(&self.used_names, &format!("{target_name}?"));
        self.used_names.insert(new_name.clone());

        let Some(mutant) = self.fish[h_idx].mutant.as_mut() else {
            return;
        };
        for component in &mut mutant.fused {
            let Some(persona) = component.persona.as_deref_mut() else {
                continue;
            };
            if persona.kind == UnfishKind::Doppleganger && !persona.doppleganger_cloned {
                persona.doppleganger_cloned = true;
                component.name = new_name;
                break;
            }
        }
    }

    pub(super) fn tick_phantoms(&mut self, dt: f32, rng: &mut impl RngExt) -> Vec<TankEvent> {
        let mut events = Vec::new();
        for fish in &mut self.fish {
            let mut teleport = false;
            let mut cross_tank = false;
            for us in fish.personas_mut() {
                if us.kind != UnfishKind::Phantom {
                    continue;
                }
                us.phantom_timer -= dt;
                if us.phantom_timer <= 0.0 {
                    us.phantom_timer = sample_exponential(rng, PHANTOM_TELEPORT_MEAN);
                    if rng.random::<f32>() < PHANTOM_CROSS_TANK_CHANCE {
                        cross_tank = true;
                    } else {
                        teleport = true;
                    }
                }
            }
            if cross_tank {
                events.push(TankEvent::PhantomCrossTank {
                    fish_name: fish.name.clone(),
                });
            } else if teleport {
                let max_x = (self.width as f32 - fish.display_width as f32).max(0.0);
                let max_y = (self.height as f32 - 1.0).max(0.0);
                fish.position.x = rng.random_range(0.0..=max_x);
                fish.position.y = rng.random_range(0.0..=max_y);
            }
        }
        events
    }
}

#[cfg(test)]
mod engulfment_tests {
    use super::*;
    use crate::entities::cow::CowVariant;
    use ratatui::style::Color;

    fn tank_with_two_merluza() -> Tank {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new(FishSpecies::Merluza, "Ann".to_string(), 10.0, 5.0, &mut rng);
        let b = Fish::new(FishSpecies::Merluza, "Bob".to_string(), 12.0, 5.0, &mut rng);
        a.weight_g = 100;
        tank.used_names.insert("Ann".to_string());
        tank.used_names.insert("Bob".to_string());
        tank.fish.push(a);
        tank.fish.push(b);
        tank
    }

    #[test]
    fn engulfment_fuses_nearby_same_species() {
        let mut tank = tank_with_two_merluza();
        tank.fish[1].weight_g = 300;
        tank.fish[0].engulf_timer = 5.0;
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "the engulfed body is absorbed");
        let survivor = &tank.fish[0];
        assert!(survivor.is_double(), "the receiver becomes a telophase");
        assert_eq!(survivor.fused_components().len(), 2);
        assert_eq!(survivor.name, "Ann / Bob", "combined name");
        assert_eq!(survivor.weight_g, 200, "stats are averaged");
        assert!(
            survivor.engulf_timer <= 0.0,
            "the window closes after fusing"
        );
    }

    #[test]
    fn engulfment_resplit_restores_both_identities() {
        let mut tank = tank_with_two_merluza();
        tank.fish[0].engulf_timer = 5.0;
        tank.tick_engulfment();
        assert!(tank.apply_named_mutation("Ann / Bob", "cytokinesis"));
        assert_eq!(tank.fish.len(), 2);
        let names: Vec<&str> = tank.fish.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Ann"), "receiver keeps its identity");
        assert!(names.contains(&"Bob"), "engulfed regains its identity");
        for f in &tank.fish {
            assert!(!f.is_double(), "each half is single again");
        }
    }

    #[test]
    fn engulfment_ignores_far_entities() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new(FishSpecies::Merluza, "Ann".to_string(), 10.0, 5.0, &mut rng);
        let far = Fish::new(FishSpecies::Salmon, "Sal".to_string(), 80.0, 5.0, &mut rng);
        a.engulf_timer = 5.0;
        tank.fish.push(a);
        tank.fish.push(far);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 2, "a far entity is out of reach");
    }

    #[test]
    fn engulfment_fuses_across_species_and_stacks_both_abilities() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut gold = Fish::new(
            FishSpecies::Goldenfish,
            "Au".to_string(),
            10.0,
            5.0,
            &mut rng,
        );
        let mutant = Fish::new(
            FishSpecies::Mutantfish,
            "Goo".to_string(),
            12.0,
            5.0,
            &mut rng,
        );
        gold.engulf_timer = 5.0;
        tank.fish.push(gold);
        tank.fish.push(mutant);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "different species fuse");
        let s = &tank.fish[0];
        assert_eq!(
            s.ability_stacks(FishSpecies::Goldenfish),
            1,
            "keeps the money zoomies"
        );
        assert_eq!(s.auto_mutate_stacks(), 1, "gains the auto-mutation");
    }

    #[test]
    fn endocytosis_after_cross_fusion_keeps_both_abilities_on_one_body() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut gold = Fish::new(
            FishSpecies::Goldenfish,
            "Au".to_string(),
            10.0,
            5.0,
            &mut rng,
        );
        let mutant = Fish::new(
            FishSpecies::Mutantfish,
            "Goo".to_string(),
            12.0,
            5.0,
            &mut rng,
        );
        gold.engulf_timer = 5.0;
        tank.fish.push(gold);
        tank.fish.push(mutant);
        tank.tick_engulfment();
        assert!(tank.apply_named_mutation("Au / Goo", "endocytosis"));
        assert_eq!(tank.fish.len(), 1, "one body remains");
        let s = &tank.fish[0];
        assert!(!s.is_double(), "collapsed to a single body");
        assert_eq!(s.ability_stacks(FishSpecies::Goldenfish), 1);
        assert_eq!(s.auto_mutate_stacks(), 1, "both abilities live on one fish");
    }

    #[test]
    fn cross_species_resplit_restores_each_species() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut gold = Fish::new(
            FishSpecies::Goldenfish,
            "Au".to_string(),
            10.0,
            5.0,
            &mut rng,
        );
        let mutant = Fish::new(
            FishSpecies::Mutantfish,
            "Goo".to_string(),
            12.0,
            5.0,
            &mut rng,
        );
        gold.engulf_timer = 5.0;
        tank.fish.push(gold);
        tank.fish.push(mutant);
        tank.tick_engulfment();
        assert!(tank.apply_named_mutation("Au / Goo", "cytokinesis"));
        assert_eq!(tank.fish.len(), 2);
        let species: Vec<FishSpecies> = tank.fish.iter().map(|f| f.species).collect();
        assert!(species.contains(&FishSpecies::Goldenfish));
        assert!(species.contains(&FishSpecies::Mutantfish));
    }

    fn engulf_two(a_body: usize, b_body: usize) -> Tank {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new(FishSpecies::Merluza, "Ann".to_string(), 10.0, 5.0, &mut rng);
        a.body_size = a_body;
        a.recompute_display_width();
        let mut b = Fish::new(FishSpecies::Salmon, "Bob".to_string(), 12.0, 5.0, &mut rng);
        b.body_size = b_body;
        b.recompute_display_width();
        a.engulf_timer = 5.0;
        tank.fish.push(a);
        tank.fish.push(b);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "the two fish fuse into one body");
        tank
    }

    #[test]
    fn fused_body_renders_both_halves_at_their_own_widths() {
        let tank = engulf_two(7, 3);
        let fused = &tank.fish[0];
        let sprite = fused.line_sprite();
        assert_eq!(
            sprite.rows[sprite.body_row].len(),
            fused.display_width,
            "the drawn fused body is exactly display_width wide"
        );
        let solo = Fish::new(
            FishSpecies::Merluza,
            "x".to_string(),
            0.0,
            0.0,
            &mut rand::rng(),
        );
        assert!(
            fused.display_width > solo.display_width,
            "a two-fish merge is wider than a single fish"
        );
    }

    #[test]
    fn cytokinesis_restores_each_halfs_visual_state() {
        let mut tank = engulf_two(7, 3);
        assert!(tank.apply_named_mutation("Ann / Bob", "cytokinesis"));
        assert_eq!(tank.fish.len(), 2);
        let ann = tank.fish.iter().find(|f| f.name == "Ann").unwrap();
        let bob = tank.fish.iter().find(|f| f.name == "Bob").unwrap();
        assert_eq!(ann.body_size, 7, "the receiver keeps its own body");
        assert_eq!(bob.body_size, 3, "the engulfed fish keeps its own body");
    }

    #[test]
    fn a_mutation_while_merged_lands_on_both_halves() {
        let mut tank = engulf_two(6, 6);
        assert!(tank.apply_named_mutation("Ann / Bob", "feet"));
        let comps = tank.fish[0].fused_components();
        assert!(
            comps[0].fish_snapshot().unwrap().feet().is_some(),
            "the receiver half grows feet"
        );
        assert!(
            comps[1].fish_snapshot().unwrap().feet().is_some(),
            "the engulfed half grows feet too"
        );
    }

    #[test]
    fn endocytosis_collapses_to_the_heavier_halfs_body() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new(FishSpecies::Merluza, "Ann".to_string(), 10.0, 5.0, &mut rng);
        a.weight_g = 10;
        let mut b = Fish::new(FishSpecies::Salmon, "Bob".to_string(), 12.0, 5.0, &mut rng);
        b.weight_g = 40;
        a.engulf_timer = 5.0;
        tank.fish.push(a);
        tank.fish.push(b);
        tank.tick_engulfment();
        assert!(tank.apply_named_mutation("Ann / Bob", "endocytosis"));
        assert_eq!(tank.fish.len(), 1);
        let merged = &tank.fish[0];
        assert!(!merged.is_double(), "endocytosis collapses to one body");
        assert_eq!(
            merged.species,
            FishSpecies::Salmon,
            "the heavier half's body wins the conflict"
        );
        assert_eq!(
            merged.fused_components().len(),
            2,
            "both abilities still stack on the single body"
        );
    }

    #[test]
    fn cows_engulf_across_variants_and_stack_milk() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let a_name = tank.spawn_cow(CowVariant::Pink, &mut rng);
        let b_name = tank.spawn_cow(CowVariant::LightYellow, &mut rng);
        tank.cows[0].position.x = 5.0;
        tank.cows[0].position.y = 5.0;
        tank.cows[1].position.x = 6.0;
        tank.cows[1].position.y = 5.0;
        tank.cows[0].engulf_timer = 5.0;
        tank.tick_engulfment();
        assert_eq!(tank.cows.len(), 1, "one cow absorbs the other");
        let survivor = &tank.cows[0];
        assert!(survivor.mutant.is_double);
        assert_eq!(survivor.name, format!("{a_name} / {b_name}"));
        let milks = survivor.milk_components();
        assert!(milks.contains(&CowVariant::Pink), "keeps strawberry milk");
        assert!(
            milks.contains(&CowVariant::LightYellow),
            "gains vanilla milk"
        );
    }

    #[test]
    fn standard_fish_engulfs_a_slime_unfish() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut salmon = Fish::new(FishSpecies::Salmon, "Sal".to_string(), 10.0, 5.0, &mut rng);
        let mut phantom =
            Fish::new_unfish(UnfishKind::Phantom, "Ph".to_string(), 12.0, 5.0, &mut rng);
        phantom.position.y = 5.0;
        salmon.engulf_timer = 5.0;
        tank.fish.push(salmon);
        tank.fish.push(phantom);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "a salmon can engulf a phantom");
        assert_eq!(tank.fish[0].name, "Sal / Ph");
        assert!(tank.apply_named_mutation("Sal / Ph", "cytokinesis"));
        assert!(
            tank.fish.iter().any(|f| f
                .unfish_state
                .as_ref()
                .is_some_and(|us| us.kind == UnfishKind::Phantom)),
            "the phantom is restored as a real phantom on re-split"
        );
    }

    fn salmon_engulfing(kind: UnfishKind, engulfed_name: &str) -> Tank {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut salmon = Fish::new(FishSpecies::Salmon, "Sal".to_string(), 10.0, 5.0, &mut rng);
        let mut unfish = Fish::new_unfish(kind, engulfed_name.to_string(), 12.0, 5.0, &mut rng);
        unfish.position.y = 5.0;
        salmon.engulf_timer = 5.0;
        tank.used_names.insert("Sal".to_string());
        tank.used_names.insert(engulfed_name.to_string());
        tank.fish.push(salmon);
        tank.fish.push(unfish);
        tank.tick_engulfment();
        tank
    }

    #[test]
    fn engulfed_phantom_teleports_its_host() {
        let mut tank = salmon_engulfing(UnfishKind::Phantom, "Ph");
        assert_eq!(tank.fish.len(), 1);
        for persona in tank.fish[0].personas_mut() {
            persona.phantom_timer = 0.0;
        }
        let mut rng = rand::rng();
        tank.tick_phantoms(1.0 / 60.0, &mut rng);
        let fired = tank.fish[0]
            .personas_mut()
            .iter()
            .any(|us| us.phantom_timer > 0.0);
        assert!(fired, "the fused phantom rearms after teleporting its host");
    }

    #[test]
    fn engulfed_blinker_makes_its_host_invisible() {
        let mut tank = salmon_engulfing(UnfishKind::Blinker, "Bl");
        assert!(!tank.fish[0].is_invisible(), "starts visible");
        for persona in tank.fish[0].personas_mut() {
            persona.blinker_phase = crate::fishes::unfish::BlinkerPhase::Invisible;
        }
        assert!(
            tank.fish[0].is_invisible(),
            "the host vanishes while its blinker passenger is invisible"
        );
    }

    #[test]
    fn engulfed_doppleganger_impersonates_on_resplit() {
        let mut tank = salmon_engulfing(UnfishKind::Doppleganger, "Dop");
        let mut rng = rand::rng();
        let victim = Fish::new(FishSpecies::Merluza, "Vic".to_string(), 40.0, 5.0, &mut rng);
        tank.used_names.insert("Vic".to_string());
        tank.fish.push(victim);
        tank.tick_dopplegangers();
        assert!(tank.apply_named_mutation("Sal / Dop", "cytokinesis"));
        assert!(
            tank.fish.iter().any(|f| f.name == "Vic?"),
            "the fused doppleganger emerges impersonating its victim"
        );
    }

    #[test]
    fn worms_engulf_and_resplit() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new_unfish(UnfishKind::Worm, "Wa".to_string(), 10.0, 5.0, &mut rng);
        let mut b = Fish::new_unfish(UnfishKind::Worm, "Wb".to_string(), 11.0, 5.0, &mut rng);
        a.position.y = 5.0;
        b.position.y = 5.0;
        a.weight_g = 100;
        b.weight_g = 300;
        a.engulf_timer = 5.0;
        tank.used_names.insert("Wa".to_string());
        tank.used_names.insert("Wb".to_string());
        tank.fish.push(a);
        tank.fish.push(b);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "one worm absorbs the other");
        assert!(
            tank.fish[0].is_double(),
            "the receiver becomes a double worm"
        );
        assert_eq!(tank.fish[0].name, "Wa / Wb");
        assert_eq!(tank.fish[0].weight_g, 200, "weights are averaged");
        assert!(tank.apply_named_mutation("Wa / Wb", "cytokinesis"));
        assert_eq!(tank.fish.len(), 2);
        let names: Vec<&str> = tank.fish.iter().map(|f| f.name.as_str()).collect();
        assert!(
            names.contains(&"Wa") && names.contains(&"Wb"),
            "both worms return"
        );
    }

    #[test]
    fn worms_do_not_engulf_standard_fish() {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut worm = Fish::new_unfish(UnfishKind::Worm, "Wa".to_string(), 10.0, 5.0, &mut rng);
        let mut salmon = Fish::new(FishSpecies::Salmon, "Sal".to_string(), 11.0, 5.0, &mut rng);
        worm.position.y = 5.0;
        salmon.position.y = 5.0;
        worm.engulf_timer = 5.0;
        tank.fish.push(worm);
        tank.fish.push(salmon);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 2, "a worm only fuses with another worm");
    }

    fn engulf_two_worms(a_color: Color, b_color: Color) -> Tank {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let mut a = Fish::new_unfish(UnfishKind::Worm, "Wa".to_string(), 10.0, 5.0, &mut rng);
        let mut b = Fish::new_unfish(UnfishKind::Worm, "Wb".to_string(), 11.0, 5.0, &mut rng);
        a.position.y = 5.0;
        b.position.y = 5.0;
        a.unfish_state.as_mut().unwrap().slime_body_color = Some(a_color);
        b.unfish_state.as_mut().unwrap().slime_body_color = Some(b_color);
        a.engulf_timer = 5.0;
        tank.used_names.insert("Wa".to_string());
        tank.used_names.insert("Wb".to_string());
        tank.fish.push(a);
        tank.fish.push(b);
        tank.tick_engulfment();
        assert_eq!(tank.fish.len(), 1, "the two worms fuse into one body");
        tank
    }

    #[test]
    fn fused_worm_renders_each_half_in_its_own_color() {
        let tank = engulf_two_worms(Color::Red, Color::Blue);
        let cells = tank.fish[0]
            .fused_worm_cells()
            .expect("a fused worm renders per-half, not as a uniform telophase");
        let colors: std::collections::HashSet<Color> = cells.iter().map(|c| c.1).collect();
        assert!(
            colors.contains(&Color::Red),
            "the left half keeps its color"
        );
        assert!(
            colors.contains(&Color::Blue),
            "the right half keeps its color"
        );
    }

    #[test]
    fn worm_carries_mutations_through_fusion_and_resplit() {
        let mut tank = engulf_two_worms(Color::Red, Color::Blue);
        assert!(tank.apply_named_mutation("Wa / Wb", "ear"));
        assert!(tank.apply_named_mutation("Wa / Wb", "cytokinesis"));
        assert_eq!(tank.fish.len(), 2);
        for f in &tank.fish {
            assert!(
                f.unfish_state.as_ref().unwrap().ear_count > 0,
                "each separated worm keeps the ears it grew while fused"
            );
        }
    }

    fn engulf_two_cows(a: CowVariant, b: CowVariant) -> (Tank, String) {
        let mut tank = Tank::new("T".to_string(), TankKind::Base, &[]);
        let mut rng = rand::rng();
        let a_name = tank.spawn_cow(a, &mut rng);
        let b_name = tank.spawn_cow(b, &mut rng);
        tank.cows[0].position.x = 5.0;
        tank.cows[0].position.y = 5.0;
        tank.cows[1].position.x = 6.0;
        tank.cows[1].position.y = 5.0;
        tank.cows[0].engulf_timer = 5.0;
        tank.tick_engulfment();
        assert_eq!(tank.cows.len(), 1, "one cow absorbs the other");
        (tank, format!("{a_name} / {b_name}"))
    }

    #[test]
    fn fused_cow_renders_each_half_in_its_own_color() {
        let (tank, _) = engulf_two_cows(CowVariant::Pink, CowVariant::LightYellow);
        let rows = crate::entities::cow::cow_sprite(&tank.cows[0]);
        let colors: std::collections::HashSet<Color> = rows.iter().flatten().map(|c| c.1).collect();
        assert!(
            colors.contains(&CowVariant::Pink.body_color()),
            "the left half keeps strawberry color"
        );
        assert!(
            colors.contains(&CowVariant::LightYellow.body_color()),
            "the right half keeps vanilla color"
        );
    }

    #[test]
    fn cow_carries_mutations_through_fusion_and_resplit() {
        let (mut tank, fused_name) = engulf_two_cows(CowVariant::Pink, CowVariant::LightYellow);
        let mut rng = rand::rng();
        let baseline =
            crate::entities::cow::Cow::new("Ref".to_string(), CowVariant::Pink, 0.0, 0.0, &mut rng)
                .eye_count();
        assert!(tank.apply_named_mutation(&fused_name, "eyeincrease"));
        assert!(tank.apply_named_mutation(&fused_name, "cytokinesis"));
        assert_eq!(tank.cows.len(), 2);
        for cow in &tank.cows {
            assert!(
                cow.eye_count() > baseline,
                "each separated cow keeps the eye it grew while fused"
            );
        }
    }
}
