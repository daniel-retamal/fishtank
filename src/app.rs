use std::collections::{HashMap, HashSet};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    consumable::{BAIT_DURATION, COFFEE_DURATION, CONSUMABLE_STACK_BONUS},
    entities::food,
    entities::species::FishSpecies,
    loot::{ConsumableKind, LootKind, roll_loot, roll_loot_no_fish},
    names,
    settings::{FPS_MAX, FPS_MIN, Settings},
    tank::{ActiveConsumable, Tank},
    ui::{
        catch_overlay::{CatchOverlay, CatchState},
        command_bar::{self, CommandBar},
        fishing_overlay::{FishingOverlay, FishingState},
        fishtanks_overlay::{FishtanksOverlay, FishtanksState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
        shop_overlay::{
            BAIT_BUY_PRICE, BuyCategoryPopup, BuyTankPopup, COFFEE_BUY_PRICE, FISH_CATALOG,
            FishListState, FishNamePopup, SellConfirm, SellEntry, SellMenuState, ShopOverlay,
            ShopPage, ShopState, TANK_BUY_PRICE, buy_cat_first_available, buy_cat_next,
            list_visible_rows,
        },
        tank_view::TankView,
    },
};

pub struct App {
    pub settings: Settings,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    used_tank_names: HashSet<String>,
    pub money: u32,
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
    index_state: Option<IndexState>,
    inventory_state: Option<InventoryState>,
    fishing_state: Option<FishingState>,
    catch_state: Option<CatchState>,
    shop_state: Option<ShopState>,
    fishtanks_state: Option<FishtanksState>,
    terminal_height: u16,
    terminal_width: u16,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let mut rng = rand::rng();

        let initial_name = names::unique_name_in(&HashSet::new(), "Fishtank");
        let mut used_tank_names = HashSet::new();
        used_tank_names.insert(initial_name.clone());
        let mut first_tank = Tank::new(initial_name);

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
            money: 0,
            food_supply: 0,
            inventory: HashMap::new(),
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
            inventory_state: None,
            fishing_state: None,
            catch_state: None,
            shop_state: None,
            fishtanks_state: None,
            terminal_height: 24,
            terminal_width: 80,
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
                let loot = if all_tanks_full {
                    roll_loot_no_fish(&mut rng)
                } else {
                    roll_loot(&mut rng, bait)
                };
                let item_qty = match &loot {
                    LootKind::Junk(_) => self.inventory.get("Junk").copied().unwrap_or(0) + 1,
                    LootKind::Consumable(ConsumableKind::Coffee) => {
                        self.inventory.get("Coffee").copied().unwrap_or(0) + 1
                    }
                    LootKind::Consumable(ConsumableKind::Bait) => {
                        self.inventory.get("Bait").copied().unwrap_or(0) + 1
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
        for ac in &mut self.active_consumables {
            ac.time_remaining -= dt;
        }
        self.active_consumables.retain(|ac| ac.time_remaining > 0.0);

        let coffee = self.coffee_stacks();
        self.tanks[self.current_tank].tick(&self.settings, coffee);
        self.tick_blink();
    }

    pub fn handle_input(&mut self, event: Event) {
        if self.catch_state.is_some() {
            self.handle_catch_input(event);
            return;
        }
        if self.fishing_state.is_some() {
            self.handle_fishing_input(event);
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
                    let action = commands::parse(&self.command_input);
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
                    let consumable_names: Vec<&str> = ["coffee", "bait"]
                        .iter()
                        .filter(|&&n| {
                            let cap = n[..1].to_uppercase() + &n[1..];
                            self.inventory.get(&cap).copied().unwrap_or(0) > 0
                        })
                        .copied()
                        .collect();
                    let tank_names: Vec<&str> =
                        self.tanks.iter().map(|t| t.name.as_str()).collect();
                    let fish_in_tanks: Vec<(&str, &str)> = self
                        .tanks
                        .iter()
                        .flat_map(|t| t.fish.iter().map(move |f| (f.name.as_str(), t.name.as_str())))
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
                let visible = self.index_visible_rows();
                if let Some(ref mut s) = self.index_state {
                    s.scroll_down(visible);
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
                    let name = names::title_case(&state.name_input);
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
            KeyCode::Left => {
                if let Some(ref mut s) = self.catch_state {
                    s.cursor_pos = prev_char_boundary(&s.name_input, s.cursor_pos);
                }
            }
            KeyCode::Right => {
                if let Some(ref mut s) = self.catch_state {
                    s.cursor_pos = next_char_boundary(&s.name_input, s.cursor_pos);
                }
            }
            KeyCode::Home => {
                if let Some(ref mut s) = self.catch_state {
                    s.cursor_pos = 0;
                }
            }
            KeyCode::End => {
                if let Some(ref mut s) = self.catch_state {
                    s.cursor_pos = s.name_input.len();
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut s) = self.catch_state {
                    if s.cursor_pos > 0 {
                        let prev = prev_char_boundary(&s.name_input, s.cursor_pos);
                        s.name_input.drain(prev..s.cursor_pos);
                        s.cursor_pos = prev;
                    }
                    s.reset_blink();
                }
            }
            KeyCode::Delete => {
                if let Some(ref mut s) = self.catch_state {
                    if s.cursor_pos < s.name_input.len() {
                        let next = next_char_boundary(&s.name_input, s.cursor_pos);
                        s.name_input.drain(s.cursor_pos..next);
                    }
                    s.reset_blink();
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut s) = self.catch_state {
                    s.name_input.insert(s.cursor_pos, c);
                    s.cursor_pos += c.len_utf8();
                    s.reset_blink();
                }
            }
            _ => {}
        }
    }

    fn apply_non_fish_loot(&mut self, loot: LootKind) {
        match loot {
            LootKind::Cash(cv) => {
                self.money += cv.amount();
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
                    let kind = match name.as_str() {
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
                        self.tanks[self.current_tank]
                            .resize(self.terminal_width, self.terminal_height.saturating_sub(bh));
                    }
                }
            }
            _ => {}
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
                if let Some(ref s) = self.fishtanks_state {
                    if s.selected != s.current_tank {
                        self.current_tank = s.selected;
                    }
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

        let money = self.money;
        let visible = list_visible_rows(&shop, self.tank_height());

        match shop.page {
            ShopPage::Main { ref mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    return;
                }
                KeyCode::Up => {
                    if *selected > 0 {
                        let new_sel = selected.saturating_sub(1);
                        if new_sel == 0 && money == 0 {
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
                            let init_sel = buy_cat_first_available(money);
                            shop.page = ShopPage::BuyCategory {
                                selected: init_sel,
                                buy_popup: None,
                                buy_tank_popup: None,
                            };
                        }
                        _ => {
                            let fish: Vec<(String, FishSpecies)> = self
                                .tanks
                                .iter()
                                .flat_map(|t| t.fish.iter().map(|f| (f.name.clone(), f.species)))
                                .collect();
                            let sellable_tanks = self.sellable_tank_names();
                            if let Some(sm) =
                                SellMenuState::new(&fish, &self.inventory, &sellable_tanks)
                            {
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
                ref mut buy_tank_popup,
            } => {
                if let Some(popup) = buy_tank_popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            *buy_tank_popup = None;
                        }
                        KeyCode::Enter if !popup.name_input.is_empty() => {
                            let name = names::title_case(&popup.name_input);
                            let actual_name = names::unique_name_in(&self.used_tank_names, &name);
                            if self.money >= TANK_BUY_PRICE {
                                self.money = self.money.saturating_sub(TANK_BUY_PRICE);
                                self.used_tank_names.insert(actual_name.clone());
                                self.tanks.push(Tank::new(actual_name));
                                self.current_tank = self.tanks.len() - 1;
                            }
                            *buy_tank_popup = None;
                        }
                        KeyCode::Left => {
                            let pos = popup.cursor_pos;
                            popup.cursor_pos = prev_char_boundary(&popup.name_input, pos);
                        }
                        KeyCode::Right => {
                            let pos = popup.cursor_pos;
                            popup.cursor_pos = next_char_boundary(&popup.name_input, pos);
                        }
                        KeyCode::Home => popup.cursor_pos = 0,
                        KeyCode::End => popup.cursor_pos = popup.name_input.len(),
                        KeyCode::Backspace if popup.cursor_pos > 0 => {
                            let prev = prev_char_boundary(&popup.name_input, popup.cursor_pos);
                            popup.name_input.drain(prev..popup.cursor_pos);
                            popup.cursor_pos = prev;
                        }
                        KeyCode::Delete if popup.cursor_pos < popup.name_input.len() => {
                            let next = next_char_boundary(&popup.name_input, popup.cursor_pos);
                            popup.name_input.drain(popup.cursor_pos..next);
                        }
                        KeyCode::Char(c) => {
                            popup.name_input.insert(popup.cursor_pos, c);
                            popup.cursor_pos += c.len_utf8();
                        }
                        _ => {}
                    }
                    shop.reset_blink();
                } else if let Some(popup) = buy_popup {
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
                            if money >= cost {
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
                                self.money = self.money.saturating_sub(cost);
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
                            *selected = buy_cat_next(*selected, false, money);
                            shop.reset_blink();
                        }
                        KeyCode::Down => {
                            *selected = buy_cat_next(*selected, true, money);
                            shop.reset_blink();
                        }
                        KeyCode::Enter => {
                            match *selected {
                                0 => {
                                    shop.page = ShopPage::BuyFishList(FishListState::new(money));
                                }
                                4 => {
                                    if money >= TANK_BUY_PRICE {
                                        *buy_tank_popup = Some(BuyTankPopup {
                                            name_input: String::new(),
                                            cursor_pos: 0,
                                        });
                                    }
                                }
                                idx => {
                                    let unit_price = match idx {
                                        1 => COFFEE_BUY_PRICE,
                                        2 => BAIT_BUY_PRICE,
                                        _ => crate::ui::shop_overlay::FOOD_BUY_PRICE,
                                    };
                                    if money >= unit_price {
                                        let max_qty = money / unit_price;
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
                        if money >= price {
                            let name = names::title_case(&name_input);
                            self.money = self.money.saturating_sub(price);
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
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                if let Some(ref mut popup) = fl.popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            fl.popup = None;
                        }
                        KeyCode::Left => {
                            popup.cursor_pos =
                                prev_char_boundary(&popup.name_input, popup.cursor_pos);
                        }
                        KeyCode::Right => {
                            popup.cursor_pos =
                                next_char_boundary(&popup.name_input, popup.cursor_pos);
                        }
                        KeyCode::Home => popup.cursor_pos = 0,
                        KeyCode::End => popup.cursor_pos = popup.name_input.len(),
                        KeyCode::Backspace if popup.cursor_pos > 0 => {
                            let prev = prev_char_boundary(&popup.name_input, popup.cursor_pos);
                            popup.name_input.drain(prev..popup.cursor_pos);
                            popup.cursor_pos = prev;
                        }
                        KeyCode::Delete if popup.cursor_pos < popup.name_input.len() => {
                            let next = next_char_boundary(&popup.name_input, popup.cursor_pos);
                            popup.name_input.drain(popup.cursor_pos..next);
                        }
                        KeyCode::Char(c) => {
                            popup.name_input.insert(popup.cursor_pos, c);
                            popup.cursor_pos += c.len_utf8();
                        }
                        _ => {}
                    }
                    shop.reset_blink();
                    self.shop_state = Some(shop);
                    return;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        shop.page = ShopPage::BuyCategory {
                            selected: buy_cat_first_available(money),
                            buy_popup: None,
                            buy_tank_popup: None,
                        };
                    }
                    KeyCode::Up => {
                        fl.scroll_up(money);
                        shop.reset_blink();
                    }
                    KeyCode::Down => {
                        fl.scroll_down(money, visible);
                        shop.reset_blink();
                    }
                    KeyCode::Enter => {
                        let idx = fl.selected;
                        let entry = &FISH_CATALOG[idx];
                        if entry.price <= money {
                            let species = entry.species;
                            let mut rng = rand::rng();
                            use crate::entities::fish::Direction;
                            let mut fish = crate::entities::fish::Fish::new(
                                species,
                                String::new(),
                                0.0,
                                0.0,
                                &mut rng,
                            );
                            fish.facing = Direction::Right;
                            fish.velocity.dx = fish.velocity.dx.abs();
                            fl.popup = Some(FishNamePopup {
                                catalog_idx: idx,
                                fish,
                                name_input: String::new(),
                                cursor_pos: 0,
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
                                    for tank in &mut self.tanks {
                                        if let Some(pos) =
                                            tank.fish.iter().position(|f| f.name == fish_name)
                                        {
                                            tank.used_names.remove(&fish_name);
                                            tank.fish.remove(pos);
                                            break;
                                        }
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
                                SellEntry::Tank { name } => {
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
                            self.money += earned;

                            let fish: Vec<(String, FishSpecies)> = self
                                .tanks
                                .iter()
                                .flat_map(|t| t.fish.iter().map(|f| (f.name.clone(), f.species)))
                                .collect();
                            let sellable_tanks = self.sellable_tank_names();
                            match SellMenuState::new(&fish, &self.inventory, &sellable_tanks) {
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

    fn sellable_tank_names(&self) -> Vec<String> {
        if self.tanks.len() <= 1 {
            return vec![];
        }
        self.tanks
            .iter()
            .filter(|t| t.fish.is_empty())
            .map(|t| t.name.clone())
            .collect()
    }

    fn bar_height(&self) -> u16 {
        command_bar::height(
            self.settings.show_stats,
            self.terminal_width,
            &self.active_consumables,
            self.money,
            self.food_supply,
            self.tank().fish.len(),
            &self.tank().name,
        )
    }

    fn tank_height(&self) -> u16 {
        self.terminal_height.saturating_sub(self.bar_height())
    }

    fn index_visible_rows(&self) -> usize {
        (self.tank_height() as usize).saturating_sub(6).max(1)
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

        frame.render_widget(
            TankView::new(self.tank(), self.settings.show_names),
            tank_area,
        );

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
            .flat_map(|t| t.fish.iter().map(move |f| (f.name.as_str(), t.name.as_str())))
            .collect();
        let ghost = commands::autocomplete(
            &self.command_input,
            &fish_names,
            &consumable_names,
            &tank_names,
            self.tank().name.as_str(),
            &fish_in_tanks,
        )
        .map(|c| c.ghost)
        .unwrap_or_default();
        frame.render_widget(
            CommandBar {
                input: &self.command_input,
                cursor_pos: self.cursor_pos,
                cursor_visible: self.cursor_visible,
                ghost: &ghost,
                fish_count: self.tank().fish.len(),
                food_supply: self.food_supply,
                money: self.money,
                show_stats: self.settings.show_stats,
                active_consumables: &self.active_consumables,
                tank_name: &self.tank().name,
            },
            command_area,
        );

        if let Some(ref state) = self.index_state {
            frame.render_widget(IndexOverlay::new(state), tank_area);
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
            frame.render_widget(ShopOverlay::new(state, self.money), tank_area);
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
                "money" => {
                    self.money = (self.money as i64 + delta as i64).max(0) as u32;
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
                let money = self.money;
                let mut state = ShopState::new();
                if money == 0
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
            commands::Action::Exit => self.running = false,
            commands::Action::Unknown => {}
        }
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
