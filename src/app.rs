use std::collections::{HashMap, HashSet};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use rand::RngExt;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    consumable::{BAIT_DURATION, COFFEE_DURATION, CONSUMABLE_STACK_BONUS},
    entities::fish::Fish,
    entities::food,
    entities::species::FishSpecies,
    loot::{ConsumableKind, LootKind, roll_loot, roll_loot_no_fish},
    names,
    settings::{FPS_MAX, FPS_MIN, Settings},
    tank::{ActiveConsumable, Tank, TankEvent, TankKind},
    ui::{
        catch_overlay::{CatchOverlay, CatchState},
        command_bar::{self, CommandBar},
        fishing_overlay::{FishingOverlay, FishingState},
        fishtanks_overlay::{FishtanksOverlay, FishtanksState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
        shop_overlay::{
            BAIT_BUY_PRICE, BuyCategoryPopup, BuyTankPopup, COFFEE_BUY_PRICE, FISH_CATALOG,
            FishListState, FishNamePopup, HELL_TANK_SELL_PRICE, NecroPopupWidget, SellConfirm,
            SellEntry, SellMenuState, ShopOverlay, ShopPage, ShopState, TANK_CATALOG,
            TANK_SELL_PRICE, TankListState, buy_cat_first_available, buy_cat_next,
            list_visible_rows,
        },
        show_overlay::{ShowOverlay, ShowSource, ShowState},
        tank_view::TankView,
        text_input::TextInput,
    },
    util::sample_exponential,
    void_ritual::{self, GiveTarget, VoidRitualState, WishAction},
};

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
                    LootKind::Junk(_) => self.inventory.get("Junk").copied().unwrap_or(0) + 1,
                    LootKind::Consumable(ConsumableKind::Coffee) => {
                        self.inventory.get("Coffee").copied().unwrap_or(0) + 1
                    }
                    LootKind::Consumable(ConsumableKind::Bait) => {
                        self.inventory.get("Bait").copied().unwrap_or(0) + 1
                    }
                    LootKind::Necronomicon => {
                        self.inventory.get("Necronomicon").copied().unwrap_or(0) + 1
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

    pub fn handle_input(&mut self, event: Event) {
        if self.void_ritual.is_blocking() {
            self.handle_void_ritual_input(event);
            return;
        }
        if self.catch_state.is_some() {
            self.handle_catch_input(event);
            return;
        }
        if self.necronomicon_popup.is_some() {
            self.handle_necro_popup_input(event);
            return;
        }
        if self.fishing_state.is_some() {
            self.handle_fishing_input(event);
            return;
        }
        if self.show_state.is_some() {
            self.handle_show_input(event);
            return;
        }
        if self.index_state.is_some() {
            self.handle_index_input(event);
            return;
        }
        if self.inventory_state.is_some() {
            self.handle_inventory_input(event);
            return;
        }
        if self.fishtanks_state.is_some() {
            self.handle_fishtanks_input(event);
            return;
        }
        if self.shop_state.is_some() {
            self.handle_shop_input(event);
            return;
        }
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.running = false;
                }
                KeyCode::Enter => {
                    let fish_names_for_parse: Vec<&str> = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter().map(|f| f.name.as_str()))
                        .collect();
                    let tank_names_for_parse: Vec<&str> =
                        self.tanks.iter().map(|t| t.name.as_str()).collect();
                    let action = commands::parse(
                        &self.command_input,
                        &fish_names_for_parse,
                        &tank_names_for_parse,
                    );
                    if !self.command_input.trim().is_empty() {
                        self.history.push(self.command_input.clone());
                    }
                    self.apply(action);
                    self.command_input.clear();
                    self.cursor_pos = 0;
                    self.history_pos = None;
                    self.draft.clear();
                    self.reset_blink();
                }
                KeyCode::Up => {
                    if self.history.is_empty() {
                        return;
                    }
                    match self.history_pos {
                        None => {
                            self.draft = self.command_input.clone();
                            self.history_pos = Some(self.history.len() - 1);
                            self.command_input = self.history[self.history.len() - 1].clone();
                        }
                        Some(0) => {}
                        Some(i) => {
                            self.history_pos = Some(i - 1);
                            self.command_input = self.history[i - 1].clone();
                        }
                    }
                    self.cursor_pos = self.command_input.len();
                    self.reset_blink();
                }
                KeyCode::Down => {
                    match self.history_pos {
                        None => {}
                        Some(i) if i + 1 < self.history.len() => {
                            self.history_pos = Some(i + 1);
                            self.command_input = self.history[i + 1].clone();
                        }
                        Some(_) => {
                            self.history_pos = None;
                            self.command_input = self.draft.clone();
                        }
                    }
                    self.cursor_pos = self.command_input.len();
                    self.reset_blink();
                }
                KeyCode::Left => {
                    self.cursor_pos = prev_char_boundary(&self.command_input, self.cursor_pos);
                    self.reset_blink();
                }
                KeyCode::Right => {
                    self.cursor_pos = next_char_boundary(&self.command_input, self.cursor_pos);
                    self.reset_blink();
                }
                KeyCode::Home => {
                    self.cursor_pos = 0;
                    self.reset_blink();
                }
                KeyCode::End => {
                    self.cursor_pos = self.command_input.len();
                    self.reset_blink();
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        let prev = prev_char_boundary(&self.command_input, self.cursor_pos);
                        self.command_input.drain(prev..self.cursor_pos);
                        self.cursor_pos = prev;
                    }
                    self.reset_blink();
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.command_input.len() {
                        let next = next_char_boundary(&self.command_input, self.cursor_pos);
                        self.command_input.drain(self.cursor_pos..next);
                    }
                    self.reset_blink();
                }
                KeyCode::Tab => {
                    let fish_names: Vec<&str> =
                        self.tank().fish.iter().map(|f| f.name.as_str()).collect();
                    let consumable_name_strings: Vec<String> = ConsumableKind::all()
                        .iter()
                        .filter(|k| self.inventory.get(k.display_name()).copied().unwrap_or(0) > 0)
                        .map(|k| k.lowercase_name())
                        .collect();
                    let consumable_names: Vec<&str> =
                        consumable_name_strings.iter().map(String::as_str).collect();
                    let tank_names: Vec<&str> =
                        self.tanks.iter().map(|t| t.name.as_str()).collect();
                    let fish_in_tanks: Vec<(&str, &str)> = self
                        .tanks
                        .iter()
                        .flat_map(|t| {
                            t.fish
                                .iter()
                                .map(move |f| (f.name.as_str(), t.name.as_str()))
                        })
                        .collect();
                    if let Some(new_input) = commands::tab_complete(
                        &self.command_input,
                        &fish_names,
                        &consumable_names,
                        &tank_names,
                        self.tank().name.as_str(),
                        &fish_in_tanks,
                    ) {
                        self.command_input = new_input;
                        self.cursor_pos = self.command_input.len();
                    }
                    self.reset_blink();
                }
                KeyCode::Char(c) => {
                    self.command_input.insert(self.cursor_pos, c);
                    self.cursor_pos += c.len_utf8();
                    self.reset_blink();
                }
                _ => {}
            },
            Event::Resize(w, h) => {
                self.terminal_height = h;
                self.terminal_width = w;
                let bh = self.bar_height();
                self.tanks[self.current_tank].resize(w, h.saturating_sub(bh));
            }
            _ => {}
        }
    }

    fn handle_show_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                if self
                    .show_state
                    .as_ref()
                    .is_some_and(|s| matches!(s.source, ShowSource::FromIndex))
                {
                    self.index_state = self.backed_index_state.take();
                }
                self.show_state = None;
            }
            KeyCode::Up => {
                if let Some(ref mut s) = self.show_state {
                    s.scroll_up();
                }
            }
            KeyCode::Down => {
                let visible = self.show_visible_count();
                if let Some(ref mut s) = self.show_state {
                    s.scroll_down(visible);
                }
            }
            _ => {}
        }
    }

    fn handle_index_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.index_state = None;
            }
            KeyCode::Up => {
                if let Some(ref mut s) = self.index_state {
                    s.scroll_up();
                }
            }
            KeyCode::Down => {
                let available = (self.tank_height() as usize).saturating_sub(7);
                if let Some(ref mut s) = self.index_state {
                    s.scroll_down(available);
                }
            }
            KeyCode::Left => {
                if let Some(ref mut s) = self.index_state {
                    s.scroll_left();
                }
            }
            KeyCode::Right => {
                let tw = self.terminal_width;
                if let Some(ref mut s) = self.index_state {
                    s.scroll_right(tw);
                }
            }
            KeyCode::Enter => {
                let fish_name = self
                    .index_state
                    .as_ref()
                    .and_then(|s| s.selected_fish_name())
                    .map(|s| s.to_string());
                let tank_name = self
                    .index_state
                    .as_ref()
                    .map(|s| s.selected_tank_name().to_string())
                    .unwrap_or_default();
                if let Some(fish_name) = fish_name {
                    let fish = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter())
                        .find(|f| f.name == fish_name)
                        .cloned();
                    if let Some(fish) = fish {
                        if fish.is_invisible() {
                            return;
                        }
                        let tank_kind = self
                            .tanks
                            .iter()
                            .find(|t| t.name == tank_name)
                            .map(|t| t.kind)
                            .unwrap_or(TankKind::Base);
                        let all_names: Vec<String> = self
                            .tanks
                            .iter()
                            .flat_map(|t| t.fish.iter().map(|f| f.name.clone()))
                            .collect();
                        let mut rng = rand::rng();
                        self.backed_index_state = self.index_state.take();
                        self.show_state = Some(ShowState::new(
                            &fish,
                            &tank_name,
                            tank_kind,
                            &all_names,
                            ShowSource::FromIndex,
                            false,
                            &mut rng,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_fishing_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        match key.kind {
            KeyEventKind::Press => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.running = false;
                    return;
                }
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    self.fishing_state = None;
                    return;
                }
                if let Some(ref mut s) = self.fishing_state
                    && !s.game_over
                    && !s.captured
                {
                    match key.code {
                        KeyCode::Left => s.is_pushing_left = true,
                        KeyCode::Right => s.is_pushing_right = true,
                        KeyCode::Down => s.is_reeling = true,
                        _ => {}
                    }
                }
            }
            KeyEventKind::Release => {
                if let Some(ref mut s) = self.fishing_state {
                    match key.code {
                        KeyCode::Left => s.is_pushing_left = false,
                        KeyCode::Right => s.is_pushing_right = false,
                        KeyCode::Down => s.is_reeling = false,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_catch_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        let is_fish = self.catch_state.as_ref().is_some_and(|s| s.is_fish());

        if !is_fish {
            match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    if let Some(state) = self.catch_state.take() {
                        self.apply_non_fish_loot(state.loot);
                    }
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Enter => {
                if !self
                    .catch_state
                    .as_ref()
                    .is_none_or(|s| s.name_input.is_empty())
                    && let Some(state) = self.catch_state.take()
                {
                    let name = names::title_case(state.name_input.as_str());
                    let mut rng = rand::rng();
                    let target_idx = if !self.tanks[self.current_tank].is_full() {
                        self.current_tank
                    } else {
                        self.tanks
                            .iter()
                            .position(|t| !t.is_full())
                            .unwrap_or(self.current_tank)
                    };
                    self.tanks[target_idx].place_fish(state.fish.unwrap(), name, &mut rng);
                }
            }
            _ => {
                if let Some(ref mut s) = self.catch_state {
                    s.name_input.handle_key(key.code);
                    s.reset_blink();
                }
            }
        }
    }

    fn apply_non_fish_loot(&mut self, loot: LootKind) {
        match loot {
            LootKind::Cash(cv) => {
                self.cash += cv.amount();
            }
            LootKind::Food(amount) => {
                self.food_supply += amount;
            }
            LootKind::Junk(_) => {
                *self.inventory.entry("Junk".to_string()).or_insert(0) += 1;
            }
            LootKind::Consumable(kind) => {
                let name = match kind {
                    ConsumableKind::Coffee => "Coffee",
                    ConsumableKind::Bait => "Bait",
                };
                *self.inventory.entry(name.to_string()).or_insert(0) += 1;
            }
            LootKind::GoldBar => {
                self.cash += crate::loot::GOLD_BAR_VALUE;
            }
            LootKind::Necronomicon => {
                *self
                    .inventory
                    .entry("Necronomicon".to_string())
                    .or_insert(0) += 1;
            }
            LootKind::Fish(_) => {}
        }
    }

    fn handle_inventory_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.inventory_state = None;
            }
            KeyCode::Up => {
                if let Some(ref mut s) = self.inventory_state {
                    s.scroll_up();
                }
            }
            KeyCode::Down => {
                let visible = self.inventory_visible_rows();
                if let Some(ref mut s) = self.inventory_state {
                    s.scroll_down(visible);
                }
            }
            KeyCode::Enter => {
                let selected_name = self
                    .inventory_state
                    .as_ref()
                    .and_then(|s| s.items.get(s.selected))
                    .map(|item| item.name.clone());
                if let Some(name) = selected_name {
                    match name.as_str() {
                        "Necronomicon" => {
                            self.inventory_state = None;
                            self.necronomicon_popup = Some(TextInput::new());
                        }
                        consumable_name => {
                            let kind = match consumable_name {
                                "Coffee" => Some(ConsumableKind::Coffee),
                                "Bait" => Some(ConsumableKind::Bait),
                                _ => None,
                            };
                            if let Some(kind) = kind {
                                self.consume_item(kind);
                                let entry = self.inventory.entry(name).or_insert(0);
                                *entry = entry.saturating_sub(1);
                                self.inventory.retain(|_, v| *v > 0);
                                if let Some(ref mut state) = self.inventory_state {
                                    state.update_from(&self.inventory, &mut rand::rng());
                                    if state.items.is_empty() {
                                        self.inventory_state = None;
                                    }
                                }
                                let bh = self.bar_height();
                                self.tanks[self.current_tank].resize(
                                    self.terminal_width,
                                    self.terminal_height.saturating_sub(bh),
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_necro_popup_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Esc => {
                self.necronomicon_popup = None;
                let mut rng = rand::rng();
                if let Some(mut state) = InventoryState::new(&self.inventory, &mut rng) {
                    if let Some(pos) = state.items.iter().position(|i| i.name == "Necronomicon") {
                        state.selected = pos;
                    }
                    self.inventory_state = Some(state);
                }
            }
            KeyCode::Enter => {
                if self
                    .necronomicon_popup
                    .as_ref()
                    .is_some_and(|i| !i.is_empty())
                {
                    let name =
                        names::title_case(self.necronomicon_popup.as_ref().unwrap().as_str());
                    let actual_name = names::unique_name_in(&self.used_tank_names, &name);
                    let entry = self
                        .inventory
                        .entry("Necronomicon".to_string())
                        .or_insert(0);
                    *entry = entry.saturating_sub(1);
                    self.inventory.retain(|_, v| *v > 0);
                    self.used_tank_names.insert(actual_name.clone());
                    self.tanks.push(Tank::new(actual_name, TankKind::Hell));
                    self.current_tank = self.tanks.len() - 1;
                    self.necronomicon_popup = None;
                }
            }
            _ => {
                if let Some(ref mut input) = self.necronomicon_popup {
                    input.handle_key(key.code);
                }
            }
        }
    }

    fn handle_fishtanks_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.fishtanks_state = None;
            }
            KeyCode::Up => {
                if let Some(ref mut s) = self.fishtanks_state {
                    s.scroll_up();
                }
            }
            KeyCode::Down => {
                let visible = self.fishtanks_visible_rows();
                if let Some(ref mut s) = self.fishtanks_state {
                    s.scroll_down(visible);
                }
            }
            KeyCode::Enter => {
                if let Some(ref s) = self.fishtanks_state
                    && s.selected != s.current_tank
                {
                    self.current_tank = s.selected;
                }
                self.fishtanks_state = None;
            }
            _ => {}
        }
    }

    fn handle_shop_input(&mut self, event: Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        let mut shop = match self.shop_state.take() {
            Some(s) => s,
            None => return,
        };

        let cash = self.cash;
        let visible = list_visible_rows(&shop, self.tank_height());

        match shop.page {
            ShopPage::Main { ref mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    return;
                }
                KeyCode::Up => {
                    if *selected > 0 {
                        let new_sel = selected.saturating_sub(1);
                        if new_sel == 0 && cash == 0 {
                        } else {
                            *selected = new_sel;
                        }
                    }
                    shop.reset_blink();
                }
                KeyCode::Down => {
                    if *selected < 1 {
                        *selected += 1;
                    }
                    shop.reset_blink();
                }
                KeyCode::Enter => {
                    match *selected {
                        0 => {
                            let init_sel = buy_cat_first_available(cash);
                            shop.page = ShopPage::BuyCategory {
                                selected: init_sel,
                                buy_popup: None,
                            };
                        }
                        _ => {
                            if let Some(sm) = self.build_sell_menu_state() {
                                shop.page = ShopPage::Sell(sm);
                            }
                        }
                    }
                    shop.reset_blink();
                }
                _ => {}
            },

            ShopPage::BuyCategory {
                ref mut selected,
                ref mut buy_popup,
            } => {
                if let Some(popup) = buy_popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            *buy_popup = None;
                        }
                        KeyCode::Left if popup.qty > 1 => {
                            popup.qty -= 1;
                        }
                        KeyCode::Right if popup.qty < popup.max_qty => {
                            popup.qty += 1;
                        }
                        KeyCode::Enter => {
                            let qty = popup.qty;
                            let unit_price = match popup.option_idx {
                                1 => COFFEE_BUY_PRICE,
                                2 => BAIT_BUY_PRICE,
                                _ => crate::ui::shop_overlay::FOOD_BUY_PRICE,
                            };
                            let cost = qty * unit_price;
                            if cash >= cost {
                                match popup.option_idx {
                                    1 => {
                                        *self.inventory.entry("Coffee".to_string()).or_insert(0) +=
                                            qty;
                                    }
                                    2 => {
                                        *self.inventory.entry("Bait".to_string()).or_insert(0) +=
                                            qty;
                                    }
                                    _ => {
                                        self.food_supply += qty;
                                    }
                                }
                                self.cash = self.cash.saturating_sub(cost);
                            }
                            *buy_popup = None;
                        }
                        _ => {}
                    }
                    shop.reset_blink();
                } else {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            shop.page = ShopPage::Main { selected: 0 };
                        }
                        KeyCode::Up => {
                            *selected = buy_cat_next(*selected, false, cash);
                            shop.reset_blink();
                        }
                        KeyCode::Down => {
                            *selected = buy_cat_next(*selected, true, cash);
                            shop.reset_blink();
                        }
                        KeyCode::Enter => {
                            match *selected {
                                0 => {
                                    shop.page = ShopPage::BuyFishList(FishListState::new(cash));
                                }
                                4 => {
                                    shop.page = ShopPage::BuyTankList(TankListState::new(cash));
                                }
                                idx => {
                                    let unit_price = match idx {
                                        1 => COFFEE_BUY_PRICE,
                                        2 => BAIT_BUY_PRICE,
                                        _ => crate::ui::shop_overlay::FOOD_BUY_PRICE,
                                    };
                                    if cash >= unit_price {
                                        let max_qty = cash / unit_price;
                                        *buy_popup = Some(BuyCategoryPopup {
                                            option_idx: idx,
                                            qty: 1,
                                            max_qty,
                                        });
                                    }
                                }
                            }
                            shop.reset_blink();
                        }
                        _ => {}
                    }
                }
            }

            ShopPage::BuyFishList(ref mut fl) => {
                let should_commit = fl
                    .popup
                    .as_ref()
                    .is_some_and(|p| key.code == KeyCode::Enter && !p.name_input.is_empty());

                if should_commit {
                    if let Some(FishNamePopup {
                        fish,
                        name_input,
                        catalog_idx,
                        ..
                    }) = fl.popup.take()
                    {
                        let price = FISH_CATALOG[catalog_idx].price;
                        if cash >= price {
                            let name = names::title_case(name_input.as_str());
                            self.cash = self.cash.saturating_sub(price);
                            self.place_purchased_fish(fish, name);
                        }
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                if let Some(ref mut popup) = fl.popup {
                    match key.code {
                        KeyCode::Esc => {
                            fl.popup = None;
                        }
                        _ => {
                            popup.name_input.handle_key(key.code);
                        }
                    }
                    shop.reset_blink();
                    self.shop_state = Some(shop);
                    return;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        shop.page = ShopPage::BuyCategory {
                            selected: buy_cat_first_available(cash),
                            buy_popup: None,
                        };
                    }
                    KeyCode::Up => {
                        fl.scroll_up(cash);
                        shop.reset_blink();
                    }
                    KeyCode::Down => {
                        fl.scroll_down(cash, visible);
                        shop.reset_blink();
                    }
                    KeyCode::Enter => {
                        let idx = fl.selected;
                        let entry = &FISH_CATALOG[idx];
                        if entry.price <= cash {
                            let species = entry.species;
                            let mut rng = rand::rng();
                            let fish =
                                crate::entities::fish::Fish::new_for_display(species, &mut rng);
                            fl.popup = Some(FishNamePopup {
                                catalog_idx: idx,
                                fish,
                                name_input: TextInput::new(),
                            });
                        }
                        shop.reset_blink();
                    }
                    _ => {}
                }
            }

            ShopPage::BuyTankList(ref mut tl) => {
                let should_commit = tl
                    .popup
                    .as_ref()
                    .is_some_and(|p| key.code == KeyCode::Enter && !p.name_input.is_empty());

                if should_commit {
                    if let Some(BuyTankPopup {
                        catalog_idx,
                        name_input,
                    }) = tl.popup.take()
                    {
                        let entry = &TANK_CATALOG[catalog_idx];
                        if cash >= entry.price {
                            let name = names::title_case(name_input.as_str());
                            let actual_name = names::unique_name_in(&self.used_tank_names, &name);
                            self.cash = self.cash.saturating_sub(entry.price);
                            self.used_tank_names.insert(actual_name.clone());
                            self.tanks.push(Tank::new(actual_name, entry.kind));
                            self.current_tank = self.tanks.len() - 1;
                        }
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                if let Some(ref mut popup) = tl.popup {
                    match key.code {
                        KeyCode::Esc => {
                            tl.popup = None;
                        }
                        _ => {
                            popup.name_input.handle_key(key.code);
                        }
                    }
                    shop.reset_blink();
                    self.shop_state = Some(shop);
                    return;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        shop.page = ShopPage::BuyCategory {
                            selected: 4,
                            buy_popup: None,
                        };
                    }
                    KeyCode::Up => {
                        tl.scroll_up(cash);
                        shop.reset_blink();
                    }
                    KeyCode::Down => {
                        tl.scroll_down(cash, visible);
                        shop.reset_blink();
                    }
                    KeyCode::Enter => {
                        let idx = tl.selected;
                        let entry = &TANK_CATALOG[idx];
                        if entry.price <= cash {
                            tl.popup = Some(BuyTankPopup {
                                catalog_idx: idx,
                                name_input: TextInput::new(),
                            });
                        }
                        shop.reset_blink();
                    }
                    _ => {}
                }
            }

            ShopPage::Sell(ref mut sm) => {
                if sm.confirm.is_some() {
                    let max_qty = sm.items[sm.confirm.as_ref().unwrap().item_idx].max_qty();
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            sm.confirm = None;
                        }
                        KeyCode::Left => {
                            sm.confirm.as_mut().unwrap().qty_down();
                        }
                        KeyCode::Right => {
                            sm.confirm.as_mut().unwrap().qty_up(max_qty);
                        }
                        KeyCode::Enter => {
                            let confirm = sm.confirm.take().unwrap();
                            let item = &sm.items[confirm.item_idx];
                            let earned = confirm.sell_qty * item.unit_price();
                            match item {
                                SellEntry::Fish { name, .. } => {
                                    let fish_name = name.clone();
                                    let mut sold = None;
                                    for tank in &mut self.tanks {
                                        if let Some(pos) =
                                            tank.fish.iter().position(|f| f.name == fish_name)
                                        {
                                            tank.used_names.remove(&fish_name);
                                            sold = Some(tank.fish.remove(pos));
                                            break;
                                        }
                                    }
                                    if let Some(fish) = sold {
                                        self.graveyard.push(fish);
                                    }
                                }
                                SellEntry::Junk { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty = self.inventory.entry("Junk".to_string()).or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Coffee { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty =
                                        self.inventory.entry("Coffee".to_string()).or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Bait { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty = self.inventory.entry("Bait".to_string()).or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Necronomicon { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty = self
                                        .inventory
                                        .entry("Necronomicon".to_string())
                                        .or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Tank { name, .. } => {
                                    let tank_name = name.clone();
                                    if let Some(pos) =
                                        self.tanks.iter().position(|t| t.name == tank_name)
                                    {
                                        self.tanks.remove(pos);
                                        self.used_tank_names.remove(&tank_name);
                                        if self.current_tank == pos {
                                            self.current_tank = if pos > 0 { pos - 1 } else { 0 };
                                        } else if self.current_tank > pos {
                                            self.current_tank -= 1;
                                        }
                                    }
                                }
                            }
                            self.inventory.retain(|_, v| *v > 0);
                            self.cash += earned;

                            match self.build_sell_menu_state() {
                                Some(new_sm) => shop.page = ShopPage::Sell(new_sm),
                                None => shop.page = ShopPage::Main { selected: 1 },
                            }
                        }
                        _ => {}
                    }
                    shop.reset_blink();
                } else {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            shop.page = ShopPage::Main { selected: 1 };
                        }
                        KeyCode::Up => {
                            sm.scroll_up();
                            shop.reset_blink();
                        }
                        KeyCode::Down => {
                            sm.scroll_down(visible);
                            shop.reset_blink();
                        }
                        KeyCode::Enter => {
                            sm.confirm = Some(SellConfirm {
                                item_idx: sm.selected,
                                sell_qty: 1,
                            });
                            shop.reset_blink();
                        }
                        _ => {}
                    }
                }
            }
        }

        self.shop_state = Some(shop);
    }

    fn build_sell_menu_state(&self) -> Option<SellMenuState> {
        let fish: Vec<(String, FishSpecies, u32)> = self
            .tanks
            .iter()
            .flat_map(|t| {
                t.fish.iter().map(|f| {
                    let mc = f.mutant.as_ref().map_or(0, |m| m.mutation_count);
                    let sv = f.species.sell_value(f.weight_g, f.size_category, mc);
                    (f.name.clone(), f.species, sv)
                })
            })
            .collect();
        let sellable_tanks = self.sellable_tanks_with_price();
        SellMenuState::new(&fish, &self.inventory, &sellable_tanks)
    }

    fn place_purchased_fish(&mut self, fish: crate::entities::fish::Fish, name: String) {
        let mut rng = rand::rng();
        let target_idx = if !self.tanks[self.current_tank].is_full() {
            self.current_tank
        } else {
            self.tanks
                .iter()
                .position(|t| !t.is_full())
                .unwrap_or(self.current_tank)
        };
        self.tanks[target_idx].place_fish(fish, name, &mut rng);
    }

    fn sellable_tanks_with_price(&self) -> Vec<(String, u32)> {
        if self.tanks.len() <= 1 {
            return vec![];
        }
        self.tanks
            .iter()
            .filter(|t| t.fish.is_empty())
            .map(|t| {
                let price = if matches!(t.kind, TankKind::Hell | TankKind::Void) {
                    HELL_TANK_SELL_PRICE
                } else {
                    TANK_SELL_PRICE
                };
                (t.name.clone(), price)
            })
            .collect()
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

    fn handle_void_ritual_input(&mut self, event: Event) {
        if matches!(self.void_ritual, VoidRitualState::Wish { .. }) {
            if self.index_state.is_some() {
                self.handle_index_input(event);
                return;
            }
            if self.show_state.is_some() {
                self.handle_show_input(event);
                return;
            }
            if self.fishtanks_state.is_some() {
                let before = self.current_tank;
                self.handle_fishtanks_input(event);
                if self.current_tank != before {
                    self.abort_void_ritual();
                }
                return;
            }
        }
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char(c) => {
                    self.command_input.push(c);
                    self.cursor_pos = self.command_input.len();
                    self.reset_blink();
                }
                KeyCode::Backspace => {
                    if !self.command_input.is_empty() {
                        self.command_input.pop();
                        self.cursor_pos = self.command_input.len();
                    }
                    self.reset_blink();
                }
                KeyCode::Enter => self.submit_ritual_input(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn submit_ritual_input(&mut self) {
        let input = self.command_input.clone();
        self.command_input.clear();
        self.cursor_pos = 0;

        match self.void_ritual {
            VoidRitualState::Prayer {
                prayer_idx,
                phrase_idx,
                ..
            } => {
                let expected = void_ritual::PRAYERS[prayer_idx][phrase_idx];
                if void_ritual::phrase_matches(&input, expected) {
                    let phrases_len = void_ritual::PRAYERS[prayer_idx].len();
                    if phrase_idx + 1 < phrases_len {
                        self.void_ritual = VoidRitualState::Prayer {
                            prayer_idx,
                            phrase_idx: phrase_idx + 1,
                            timeout: void_ritual::PRAYER_TIMEOUT_SECS,
                        };
                    } else {
                        self.void_ritual = VoidRitualState::FinalPhrase {
                            timeout: void_ritual::PRAYER_TIMEOUT_SECS,
                        };
                    }
                } else {
                    self.abort_void_ritual();
                }
            }
            VoidRitualState::FinalPhrase { .. } => {
                if void_ritual::phrase_matches(&input, void_ritual::FINAL_PRAYER_PHRASE) {
                    self.next_prayer = (self.next_prayer + 1) % void_ritual::PRAYERS.len();
                    self.void_ritual = VoidRitualState::Wish {
                        retries_left: void_ritual::MAX_WISH_RETRIES,
                    };
                } else {
                    self.abort_void_ritual();
                }
            }
            VoidRitualState::Wish { retries_left } => {
                if input.starts_with('/') {
                    let fish_names_ref: Vec<&str> = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter().map(|f| f.name.as_str()))
                        .collect();
                    let tank_names_ref: Vec<&str> =
                        self.tanks.iter().map(|t| t.name.as_str()).collect();
                    let action =
                        commands::parse(&input, &fish_names_ref, &tank_names_ref);
                    match action {
                        commands::Action::Index { .. }
                        | commands::Action::Show { .. }
                        | commands::Action::Fishtanks
                        | commands::Action::ToggleNames
                        | commands::Action::ToggleStats => {
                            self.apply(action);
                            return;
                        }
                        _ => {}
                    }
                }
                let fish_names: Vec<String> = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.fish.iter().map(|f| f.name.clone()))
                    .collect();
                let tank_names: Vec<String> = self.tanks.iter().map(|t| t.name.clone()).collect();
                let graveyard_names: Vec<String> =
                    self.graveyard.iter().map(|f| f.name.clone()).collect();
                let ctx = void_ritual::WishCtx {
                    fish_names: &fish_names,
                    tank_names: &tank_names,
                    graveyard_names: &graveyard_names,
                };
                match void_ritual::parse_wish(input.trim(), &ctx) {
                    None => {
                        if retries_left <= 1 {
                            self.abort_void_ritual();
                        } else {
                            self.void_ritual = VoidRitualState::Wish {
                                retries_left: retries_left - 1,
                            };
                        }
                    }
                    Some(action) => {
                        self.execute_wish(action);
                        self.abort_void_ritual();
                    }
                }
            }
            VoidRitualState::Idle { .. } => {}
        }
    }

    fn execute_wish(&mut self, action: WishAction) {
        match action {
            WishAction::Give(target) => self.execute_give(target),
            WishAction::Mutate {
                fish_name,
                mutation,
            } => {
                let tank_idx = self
                    .tanks
                    .iter()
                    .position(|t| t.fish.iter().any(|f| f.name == fish_name));
                if let Some(ti) = tank_idx {
                    self.tanks[ti].miracle_mutate_fish(&fish_name, &mutation);
                }
            }
            WishAction::Revive { fish_name } => {
                if let Some(pos) = self.graveyard.iter().position(|f| f.name == fish_name) {
                    let fish = self.graveyard.remove(pos);
                    let name = fish.name.clone();
                    let ct = self.current_tank;
                    let mut rng = rand::rng();
                    self.tanks[ct].place_fish(fish, name, &mut rng);
                }
            }
            WishAction::Clone { fish_name } => {
                let original = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.fish.iter())
                    .find(|f| f.name == fish_name)
                    .cloned();
                if let Some(orig) = original {
                    let clone_name = format!("{}'s Clone", orig.name);
                    let ct = self.current_tank;
                    if !self.tanks[ct].is_full() {
                        let mut rng = rand::rng();
                        let new_fish = orig;
                        self.tanks[ct].place_fish(new_fish, clone_name, &mut rng);
                    }
                }
            }
            WishAction::Bless { fish_name } => {
                for tank in &mut self.tanks {
                    if let Some(fish) = tank.fish.iter_mut().find(|f| f.name == fish_name) {
                        fish.devil_marked = false;
                        break;
                    }
                }
            }
            WishAction::Expand { tank_name } => {
                if let Some(tank) = self.tanks.iter_mut().find(|t| t.name == tank_name) {
                    tank.expand(void_ritual::EXPAND_AMOUNT);
                }
            }
            WishAction::Anything => {
                let mut rng = rand::rng();
                for _ in 0..2 {
                    match rng.random_range(0..4u32) {
                        0 => self.cash += void_ritual::GIVE_RESOURCE_AMOUNT,
                        1 => self.food_supply += void_ritual::GIVE_RESOURCE_AMOUNT,
                        2 => {
                            *self.inventory.entry("Coffee".to_string()).or_insert(0) +=
                                void_ritual::GIVE_COFFEE_QTY;
                        }
                        _ => {
                            *self.inventory.entry("Bait".to_string()).or_insert(0) +=
                                void_ritual::GIVE_BAIT_QTY;
                        }
                    }
                }
            }
            WishAction::Nothing => {
                self.nothing_stacks += 1;
            }
        }
    }

    fn execute_give(&mut self, target: GiveTarget) {
        match target {
            GiveTarget::Cash => self.cash += void_ritual::GIVE_RESOURCE_AMOUNT,
            GiveTarget::Food => self.food_supply += void_ritual::GIVE_RESOURCE_AMOUNT,
            GiveTarget::Item { name, qty } => {
                *self.inventory.entry(name.to_string()).or_insert(0) += qty;
            }
            GiveTarget::Fish(species) => {
                let ct = self.current_tank;
                if !self.tanks[ct].is_full() {
                    let un_name = format!("Un{}", species.config().name);
                    let mut rng = rand::rng();
                    self.tanks[ct].spawn_fish(species, un_name, &mut rng);
                }
            }
            GiveTarget::Tank(kind) => {
                let un_name = void_ritual::tank_kind_un_name(kind);
                let actual_name = names::unique_name_in(&self.used_tank_names, un_name);
                self.used_tank_names.insert(actual_name.clone());
                self.tanks.push(Tank::new(actual_name, kind));
            }
        }
    }

    fn apply(&mut self, action: commands::Action) {
        match action {
            commands::Action::Feed(count) => {
                let n = if count == 0 {
                    food::DEFAULT_COUNT
                } else {
                    count
                };
                let ct = self.current_tank;
                self.tanks[ct].feed(n, &mut self.food_supply);
            }
            commands::Action::SetFps(fps) => self.settings.fps = fps.clamp(FPS_MIN, FPS_MAX),
            commands::Action::ToggleNames => self.settings.show_names = !self.settings.show_names,
            commands::Action::ToggleStats => {
                self.settings.show_stats = !self.settings.show_stats;
                let bh = self.bar_height();
                self.tanks[self.current_tank]
                    .resize(self.terminal_width, self.terminal_height.saturating_sub(bh));
            }
            commands::Action::ModResource { name, delta } => match name.to_lowercase().as_str() {
                "cash" => {
                    self.cash = (self.cash as i64 + delta as i64).max(0) as u32;
                }
                "food" => {
                    self.food_supply = (self.food_supply as i64 + delta as i64).max(0) as u32;
                }
                _ => {
                    let current = self.inventory.get(&name).copied().unwrap_or(0);
                    let new_val = (current as i64 + delta as i64).max(0) as u32;
                    if new_val == 0 {
                        self.inventory.remove(&name);
                    } else {
                        self.inventory.insert(name, new_val);
                    }
                }
            },
            commands::Action::Spawn(species, name) => {
                let mut rng = rand::rng();
                self.tank_mut()
                    .spawn_fish(species, names::title_case(&name), &mut rng);
            }
            commands::Action::Mutate(fish_name, mutation_name) => {
                self.tank_mut()
                    .apply_named_mutation(&fish_name, &mutation_name);
            }
            commands::Action::Index { all, tank_filter } => {
                if let Some(filter) = tank_filter {
                    let tank_idx = self
                        .tanks
                        .iter()
                        .position(|t| t.name.eq_ignore_ascii_case(&filter));
                    if let Some(idx) = tank_idx {
                        let fish_with_tanks: Vec<(&str, &crate::entities::fish::Fish)> = self.tanks
                            [idx]
                            .fish
                            .iter()
                            .map(|f| (self.tanks[idx].name.as_str(), f))
                            .collect();
                        self.index_state = Some(IndexState::new(&fish_with_tanks, all, false));
                    }
                } else {
                    let fish_with_tanks: Vec<(&str, &crate::entities::fish::Fish)> = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter().map(|f| (t.name.as_str(), f)))
                        .collect();
                    self.index_state = Some(IndexState::new(&fish_with_tanks, all, true));
                }
            }
            commands::Action::Inventory => {
                if let Some(state) = InventoryState::new(&self.inventory, &mut rand::rng()) {
                    self.inventory_state = Some(state);
                }
            }
            commands::Action::Shop => {
                let cash = self.cash;
                let mut state = ShopState::new();
                if cash == 0
                    && let ShopPage::Main { ref mut selected } = state.page
                {
                    *selected = 1;
                }
                self.shop_state = Some(state);
            }
            commands::Action::Consume { name } => {
                let kind = match name.to_lowercase().as_str() {
                    "coffee" => ConsumableKind::Coffee,
                    "bait" => ConsumableKind::Bait,
                    _ => return,
                };
                let item_name = match kind {
                    ConsumableKind::Coffee => "Coffee",
                    ConsumableKind::Bait => "Bait",
                };
                let available = self.inventory.get(item_name).copied().unwrap_or(0);
                if available == 0 {
                    return;
                }
                self.consume_item(kind);
                let entry = self.inventory.entry(item_name.to_string()).or_insert(0);
                *entry = entry.saturating_sub(1);
                self.inventory.retain(|_, v| *v > 0);
                let bh = self.bar_height();
                self.tanks[self.current_tank]
                    .resize(self.terminal_width, self.terminal_height.saturating_sub(bh));
            }
            commands::Action::Fish { no_death, no_fish } => {
                let mut state = FishingState::new();
                state.no_death = no_death;
                state.no_fish = no_fish;
                self.fishing_state = Some(state);
            }
            commands::Action::Switch(tank_name) => {
                if self.tanks.len() <= 1 {
                    return;
                }
                if let Some(idx) = self
                    .tanks
                    .iter()
                    .position(|t| t.name.eq_ignore_ascii_case(&tank_name))
                {
                    self.current_tank = idx;
                }
            }
            commands::Action::Move { fish, tank } => {
                let target_idx = match self
                    .tanks
                    .iter()
                    .position(|t| t.name.eq_ignore_ascii_case(&tank))
                {
                    Some(i) => i,
                    None => return,
                };
                if self.tanks[target_idx].is_full() {
                    return;
                }
                if self.tanks[target_idx]
                    .fish
                    .iter()
                    .any(|f| f.name.eq_ignore_ascii_case(&fish))
                {
                    return;
                }
                let source_idx =
                    match (0..self.tanks.len())
                        .filter(|&i| i != target_idx)
                        .find(|&i| {
                            self.tanks[i]
                                .fish
                                .iter()
                                .any(|f| f.name.eq_ignore_ascii_case(&fish))
                        }) {
                        Some(i) => i,
                        None => return,
                    };
                let fish_pos = self.tanks[source_idx]
                    .fish
                    .iter()
                    .position(|f| f.name.eq_ignore_ascii_case(&fish))
                    .unwrap();
                let fish_obj = self.tanks[source_idx].fish.remove(fish_pos);
                let fish_name = fish_obj.name.clone();
                self.tanks[source_idx].used_names.remove(&fish_name);
                let mut rng = rand::rng();
                self.tanks[target_idx].place_fish(fish_obj, fish_name, &mut rng);
            }
            commands::Action::Fishtanks => {
                let visible = self.fishtanks_visible_rows();
                self.fishtanks_state =
                    Some(FishtanksState::new(&self.tanks, self.current_tank, visible));
            }
            commands::Action::Show {
                name: fish_name,
                all,
            } => {
                let found = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.fish.iter().map(move |f| (f, t.name.as_str())))
                    .find(|(f, _)| f.name.eq_ignore_ascii_case(&fish_name))
                    .map(|(f, tn)| (f.clone(), tn.to_string()));
                if let Some((fish, tank_name)) = found {
                    if fish.is_invisible() {
                        return;
                    }
                    let tank_kind = self
                        .tanks
                        .iter()
                        .find(|t| t.name == tank_name)
                        .map(|t| t.kind)
                        .unwrap_or(TankKind::Base);
                    let all_names: Vec<String> = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter().map(|f| f.name.clone()))
                        .collect();
                    let mut rng = rand::rng();
                    self.show_state = Some(ShowState::new(
                        &fish,
                        &tank_name,
                        tank_kind,
                        &all_names,
                        ShowSource::FromCommand,
                        all,
                        &mut rng,
                    ));
                }
            }
            commands::Action::Exit => self.running = false,
            commands::Action::VoidSpawn => {
                if self.tanks[self.current_tank].kind == TankKind::Void {
                    let mut rng = rand::rng();
                    self.tanks[self.current_tank].spawn_unfish(&mut rng);
                }
            }
            commands::Action::StartVoidWish { skip } => {
                if self.tanks[self.current_tank].kind == TankKind::Void {
                    if skip {
                        self.command_input.clear();
                        self.cursor_pos = 0;
                        self.void_ritual = VoidRitualState::Wish {
                            retries_left: void_ritual::MAX_WISH_RETRIES,
                        };
                    } else {
                        self.start_void_ritual();
                    }
                }
            }
            commands::Action::Unknown => {}
        }
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
    }
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let mut i = pos - 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut i = pos + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}
