use std::collections::{HashMap, HashSet};

use rand::RngExt;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    consumable::{BAIT_DURATION, COFFEE_DURATION, CONSUMABLE_STACK_BONUS},
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    loot::{ConsumableKind, ItemKind, LootKind, roll_loot, roll_loot_no_fish},
    names,
    settings::Settings,
    tank::{ActiveConsumable, Tank, TankEvent, TankKind},
    ui::{
        catch_overlay::{CatchOverlay, CatchState},
        command_bar::{self, CommandBar},
        fishing_overlay::{FishingOverlay, FishingState},
        fishtanks_overlay::{FishtanksOverlay, FishtanksState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
        shop_overlay::{NecroPopupWidget, ShopOverlay, ShopState},
        show_overlay::{ShowOverlay, ShowState},
        tank_view::TankView,
        text_input::TextInput,
    },
    util::sample_exponential,
    void_ritual::{self, VoidRitualState},
};

mod input;

const TERMINAL_HEIGHT_DEFAULT: u16 = 24;
const TERMINAL_WIDTH_DEFAULT: u16 = 80;

pub struct App {
    pub settings: Settings,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    used_tank_names: HashSet<String>,
    pub cash: u32,
    pub food_supply: u32,
    pub inventory: HashMap<String, u32>,
    pub active_consumables: Vec<ActiveConsumable>,
    pub command_input: String,
    pub running: bool,
    cursor_pos: usize,
    cursor_visible: bool,
    blink_counter: f32,
    history: Vec<String>,
    history_pos: Option<usize>,
    draft: String,
    pub index_state: Option<IndexState>,
    backed_index_state: Option<IndexState>,
    show_state: Option<ShowState>,
    inventory_state: Option<InventoryState>,
    fishing_state: Option<FishingState>,
    catch_state: Option<CatchState>,
    shop_state: Option<ShopState>,
    pub fishtanks_state: Option<FishtanksState>,
    necronomicon_popup: Option<TextInput>,
    terminal_height: u16,
    terminal_width: u16,
    pub graveyard: Vec<Fish>,
    pub void_ritual: VoidRitualState,
    pub next_prayer: usize,
    pub nothing_stacks: u32,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let mut rng = rand::rng();
        let initial_ritual_timer = sample_exponential(&mut rng, void_ritual::VOID_RITUAL_MEAN_SECS);

        let initial_name = names::unique_name_in(&HashSet::new(), "Fishtank");
        let mut used_tank_names = HashSet::new();
        used_tank_names.insert(initial_name.clone());
        let mut first_tank = Tank::new(initial_name, TankKind::Base);

        for (species, name) in [
            (FishSpecies::Merluza, "merluza"),
            (FishSpecies::Betta, "betta"),
            (FishSpecies::Salmon, "salmon"),
            (FishSpecies::Merluza, "merluza"),
            (FishSpecies::Betta, "betta"),
            (FishSpecies::Salmon, "salmon"),
            (FishSpecies::Merluza, "merluza"),
            (FishSpecies::Betta, "betta"),
            (FishSpecies::Salmon, "salmon"),
        ] {
            first_tank.spawn_fish(species, name.to_string(), &mut rng);
        }

        Self {
            settings,
            tanks: vec![first_tank],
            current_tank: 0,
            used_tank_names,
            cash: 40_000,
            food_supply: 0,
            inventory: {
                let mut inv = HashMap::new();
                inv.insert("Coffee".to_string(), 1);
                inv.insert("Bait".to_string(), 1);
                inv.insert("Necronomicon".to_string(), 1);
                inv
            },
            active_consumables: Vec::new(),
            command_input: String::new(),
            running: true,
            cursor_pos: 0,
            cursor_visible: true,
            blink_counter: 0.0,
            history: Vec::new(),
            history_pos: None,
            draft: String::new(),
            index_state: None,
            backed_index_state: None,
            show_state: None,
            inventory_state: None,
            fishing_state: None,
            catch_state: None,
            shop_state: None,
            fishtanks_state: None,
            necronomicon_popup: None,
            terminal_height: TERMINAL_HEIGHT_DEFAULT,
            terminal_width: TERMINAL_WIDTH_DEFAULT,
            graveyard: Vec::new(),
            void_ritual: VoidRitualState::Idle {
                timer: initial_ritual_timer,
            },
            next_prayer: 0,
            nothing_stacks: 0,
        }
    }

    fn tank(&self) -> &Tank {
        &self.tanks[self.current_tank]
    }

    fn tank_mut(&mut self) -> &mut Tank {
        &mut self.tanks[self.current_tank]
    }

    fn coffee_stacks(&self) -> u32 {
        self.active_consumables
            .iter()
            .filter(|c| matches!(c.kind, ConsumableKind::Coffee))
            .map(|c| c.stacks)
            .sum()
    }

    fn bait_stacks(&self) -> u32 {
        self.active_consumables
            .iter()
            .filter(|c| matches!(c.kind, ConsumableKind::Bait))
            .map(|c| c.stacks)
            .sum()
    }

    fn devils_luck(&self) -> u32 {
        if self.tank().kind == TankKind::Hell {
            self.tank().fish.len() as u32
        } else {
            0
        }
    }

    fn consume_item(&mut self, kind: ConsumableKind) {
        let duration = match kind {
            ConsumableKind::Coffee => COFFEE_DURATION,
            ConsumableKind::Bait => BAIT_DURATION,
        };
        if let Some(existing) = self.active_consumables.iter_mut().find(|c| c.kind == kind) {
            existing.stacks += 1;
            existing.time_remaining += CONSUMABLE_STACK_BONUS;
        } else {
            self.active_consumables.push(ActiveConsumable {
                kind,
                stacks: 1,
                time_remaining: duration,
            });
        }
    }

    pub fn tick(&mut self) {
        if let Some(ref mut catch) = self.catch_state {
            catch.tick(self.settings.fps);
            return;
        }

        if self.necronomicon_popup.is_some() {
            return;
        }

        if self.fishing_state.is_some() {
            let coffee = self.coffee_stacks();
            self.fishing_state
                .as_mut()
                .unwrap()
                .tick(self.settings.fps, coffee);
            let game_over = self.fishing_state.as_ref().unwrap().game_over;
            let captured = self.fishing_state.as_ref().unwrap().captured;
            if game_over {
                self.fishing_state = None;
            } else if captured {
                let mut rng = rand::rng();
                let bait = self.bait_stacks();
                let all_tanks_full = self.tanks.iter().all(|t| t.is_full());
                let devils_luck = self.devils_luck();
                let loot = if all_tanks_full {
                    roll_loot_no_fish(&mut rng, devils_luck)
                } else {
                    roll_loot(&mut rng, bait, devils_luck)
                };
                let item_qty = match &loot {
                    LootKind::Item(ItemKind::GoldBar) => 0,
                    LootKind::Item(item) => {
                        self.inventory.get(item.display_name()).copied().unwrap_or(0) + 1
                    }
                    _ => 0,
                };
                let mut cs = CatchState::new(loot, &mut rng);
                cs.item_qty = item_qty;
                self.catch_state = Some(cs);
                self.fishing_state = None;
            }
            return;
        }

        if let Some(ref mut show) = self.show_state {
            show.tick_animation(1.0 / self.settings.fps);
            return;
        }

        if let Some(ref mut idx) = self.index_state {
            idx.tick_animation(1.0 / self.settings.fps);
            return;
        }

        if self.inventory_state.is_some() || self.fishtanks_state.is_some() {
            return;
        }

        if let Some(ref mut shop) = self.shop_state {
            shop.tick(self.settings.fps);
            return;
        }

        let dt = 1.0 / self.settings.fps;

        self.tick_void_ritual(dt);
        if self.void_ritual.is_blocking() {
            self.tick_blink();
            return;
        }

        for ac in &mut self.active_consumables {
            ac.time_remaining -= dt;
        }
        self.active_consumables.retain(|ac| ac.time_remaining > 0.0);

        let coffee = self.coffee_stacks();
        for i in 0..self.tanks.len() {
            let events = self.tanks[i].tick(&self.settings, coffee);
            self.cash += self.tanks[i].pending_star_cash;
            self.tanks[i].pending_star_cash = 0;
            for event in events {
                match event {
                    TankEvent::PhantomCrossTank { fish_name } => {
                        self.handle_phantom_cross_tank(i, &fish_name);
                    }
                }
            }
        }
        self.tick_blink();
    }

    fn bar_height(&self) -> u16 {
        command_bar::height(
            self.settings.show_stats,
            self.terminal_width,
            &self.active_consumables,
            self.cash,
            self.food_supply,
            self.tank().fish.len(),
            self.tank().capacity(),
            &self.tank().name,
            self.devils_luck(),
        )
    }

    fn tank_height(&self) -> u16 {
        self.terminal_height.saturating_sub(self.bar_height())
    }

    fn show_visible_count(&self) -> usize {
        if let Some(ref s) = self.show_state {
            ShowState::visible_count(s.overlay_h(self.tank_height(), self.terminal_width))
        } else {
            0
        }
    }

    fn inventory_visible_rows(&self) -> usize {
        (self.tank_height() as usize).saturating_sub(6).max(1)
    }

    fn fishtanks_visible_rows(&self) -> usize {
        (self.tank_height() as usize).saturating_sub(6).max(1)
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let full_area = frame.area();
        self.terminal_height = full_area.height;
        self.terminal_width = full_area.width;

        let [tank_area, command_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(self.bar_height())])
                .areas(full_area);

        self.tanks[self.current_tank].resize(tank_area.width, tank_area.height);

        {
            let ritual_blocking = self.void_ritual.is_blocking()
                && self.tanks[self.current_tank].kind == TankKind::Void;
            let mut tv = TankView::new(self.tank(), self.settings.show_names);
            if ritual_blocking {
                let text = void_ritual::wish_display_text(&self.void_ritual, self.next_prayer);
                tv = tv.with_ritual(text);
            }
            frame.render_widget(tv, tank_area);
        }

        let ritual_blocking = self.void_ritual.is_blocking();
        let fish_names: Vec<&str> = self.tank().fish.iter().map(|f| f.name.as_str()).collect();
        let consumable_names: Vec<&str> = ["coffee", "bait"]
            .iter()
            .filter(|&&n| {
                let cap = n[..1].to_uppercase() + &n[1..];
                self.inventory.get(&cap).copied().unwrap_or(0) > 0
            })
            .copied()
            .collect();
        let tank_names: Vec<&str> = self.tanks.iter().map(|t| t.name.as_str()).collect();
        let fish_in_tanks: Vec<(&str, &str)> = self
            .tanks
            .iter()
            .flat_map(|t| {
                t.fish
                    .iter()
                    .map(move |f| (f.name.as_str(), t.name.as_str()))
            })
            .collect();
        let ghost = if ritual_blocking {
            String::new()
        } else {
            commands::autocomplete(
                &self.command_input,
                &fish_names,
                &consumable_names,
                &tank_names,
                self.tank().name.as_str(),
                &fish_in_tanks,
            )
            .map(|c| c.ghost)
            .unwrap_or_default()
        };
        let devils_luck = self.devils_luck();
        frame.render_widget(
            CommandBar {
                input: &self.command_input,
                cursor_pos: self.cursor_pos,
                cursor_visible: self.cursor_visible,
                ghost: &ghost,
                fish_count: self.tank().fish.len(),
                fish_capacity: self.tank().capacity(),
                food_supply: self.food_supply,
                cash: self.cash,
                show_stats: self.settings.show_stats,
                active_consumables: &self.active_consumables,
                tank_name: &self.tank().name,
                devils_luck,
            },
            command_area,
        );

        if let Some(ref state) = self.index_state {
            frame.render_widget(IndexOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.show_state {
            frame.render_widget(ShowOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.inventory_state {
            frame.render_widget(InventoryOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.fishtanks_state {
            frame.render_widget(FishtanksOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.fishing_state {
            frame.render_widget(FishingOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.catch_state {
            frame.render_widget(CatchOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.shop_state {
            frame.render_widget(ShopOverlay::new(state, self.cash), tank_area);
        }

        if let Some(ref input) = self.necronomicon_popup {
            frame.render_widget(
                NecroPopupWidget {
                    input,
                    cursor_visible: self.cursor_visible,
                },
                tank_area,
            );
        }
    }

    fn has_any_overlay(&self) -> bool {
        self.show_state.is_some()
            || self.index_state.is_some()
            || self.inventory_state.is_some()
            || self.fishing_state.is_some()
            || self.catch_state.is_some()
            || self.shop_state.is_some()
            || self.fishtanks_state.is_some()
            || self.necronomicon_popup.is_some()
    }

    fn tick_void_ritual(&mut self, dt: f32) {
        enum Tr {
            None,
            Abort,
            ResetMaybeStart(f32, bool),
        }

        let is_void = self.tanks[self.current_tank].kind == TankKind::Void;
        let has_overlay = self.has_any_overlay();

        let tr = match &mut self.void_ritual {
            VoidRitualState::Wish { .. } => Tr::None,
            VoidRitualState::Prayer { timeout, .. } | VoidRitualState::FinalPhrase { timeout } => {
                *timeout -= dt;
                if *timeout <= 0.0 { Tr::Abort } else { Tr::None }
            }
            VoidRitualState::Idle { timer } => {
                *timer -= dt;
                if *timer > 0.0 {
                    Tr::None
                } else {
                    let mean = void_ritual::ritual_mean_secs(self.nothing_stacks);
                    let new_t = sample_exponential(&mut rand::rng(), mean);
                    Tr::ResetMaybeStart(new_t, is_void && !has_overlay)
                }
            }
        };

        match tr {
            Tr::None => {}
            Tr::Abort => self.abort_void_ritual(),
            Tr::ResetMaybeStart(new_t, start) => {
                if let VoidRitualState::Idle { timer } = &mut self.void_ritual {
                    *timer = new_t;
                }
                if start {
                    self.start_void_ritual();
                }
            }
        }
    }

    fn start_void_ritual(&mut self) {
        self.command_input.clear();
        self.cursor_pos = 0;
        self.void_ritual = VoidRitualState::Prayer {
            prayer_idx: self.next_prayer,
            phrase_idx: 0,
            timeout: void_ritual::PRAYER_TIMEOUT_SECS,
        };
    }

    fn abort_void_ritual(&mut self) {
        let mean = void_ritual::ritual_mean_secs(self.nothing_stacks);
        let timer = sample_exponential(&mut rand::rng(), mean);
        self.void_ritual = VoidRitualState::Idle { timer };
        self.command_input.clear();
        self.cursor_pos = 0;
    }

    fn handle_phantom_cross_tank(&mut self, source_idx: usize, fish_name: &str) {
        let candidates: Vec<usize> = (0..self.tanks.len())
            .filter(|&i| i != source_idx && !self.tanks[i].is_full())
            .collect();
        if candidates.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let target_idx = candidates[rng.random_range(0..candidates.len())];
        let pos = match self.tanks[source_idx]
            .fish
            .iter()
            .position(|f| f.name == fish_name)
        {
            Some(p) => p,
            None => return,
        };
        let fish = self.tanks[source_idx].fish.remove(pos);
        let name = fish.name.clone();
        self.tanks[source_idx].used_names.remove(&name);
        self.tanks[target_idx].place_fish(fish, name, &mut rng);
    }

    fn tick_blink(&mut self) {
        if !self.settings.cursor_blink {
            self.cursor_visible = true;
            self.blink_counter = 0.0;
            return;
        }
        self.blink_counter += 1.0;
        let half_period = (self.settings.fps * 0.5).max(1.0);
        if self.blink_counter >= half_period {
            self.blink_counter = 0.0;
            self.cursor_visible = !self.cursor_visible;
        }
    }

    fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.blink_counter = 0.0;
        if let Some(ref mut s) = self.shop_state {
            s.reset_blink();
        }
        if let Some(ref mut s) = self.catch_state {
            s.reset_blink();
        }
    }
}
