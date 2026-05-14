use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    entities::food,
    entities::species::FishSpecies,
    loot::{ConsumableKind, LootKind, roll_loot},
    settings::{FPS_MAX, FPS_MIN, Settings},
    tank::Tank,
    ui::{
        catch_overlay::{CatchOverlay, CatchState, title_case},
        command_bar::{self, CommandBar},
        fishing_overlay::{FishingOverlay, FishingState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
        shop_overlay::{
            BAIT_BUY_PRICE, BuyCategoryPopup, COFFEE_BUY_PRICE, FISH_CATALOG, FishListState,
            FishNamePopup, SellConfirm, SellEntry, SellMenuState, ShopOverlay, ShopPage, ShopState,
            buy_cat_first_available, buy_cat_next, list_visible_rows,
        },
        tank_view::TankView,
    },
};

pub struct App {
    pub settings: Settings,
    pub tank: Tank,
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
    terminal_height: u16,
    terminal_width: u16,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let mut tank = Tank::new();
        let mut rng = rand::rng();

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
            tank.spawn_fish(species, name.to_string(), &mut rng);
        }

        Self {
            settings,
            tank,
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
            terminal_height: 24,
            terminal_width: 80,
        }
    }

    pub fn tick(&mut self) {
        if let Some(ref mut catch) = self.catch_state {
            catch.tick(self.settings.fps);
            return;
        }

        if self.fishing_state.is_some() {
            let coffee = self.tank.coffee_stacks();
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
                let bait = self.tank.bait_stacks();
                let loot = roll_loot(&mut rng, bait);
                let item_qty = match &loot {
                    LootKind::Junk(_) => self.tank.inventory.get("Junk").copied().unwrap_or(0) + 1,
                    LootKind::Consumable(ConsumableKind::Coffee) => {
                        self.tank.inventory.get("Coffee").copied().unwrap_or(0) + 1
                    }
                    LootKind::Consumable(ConsumableKind::Bait) => {
                        self.tank.inventory.get("Bait").copied().unwrap_or(0) + 1
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

        if self.inventory_state.is_some() {
            return;
        }
        if let Some(ref mut shop) = self.shop_state {
            shop.tick(self.settings.fps);
            return;
        }
        self.tank.tick(&self.settings);
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
                        self.tank.fish.iter().map(|f| f.name.as_str()).collect();
                    let consumable_names: Vec<&str> = ["coffee", "bait"]
                        .iter()
                        .filter(|&&n| {
                            let cap = n[..1].to_uppercase() + &n[1..];
                            self.tank.inventory.get(&cap).copied().unwrap_or(0) > 0
                        })
                        .copied()
                        .collect();
                    if let Some(new_input) =
                        commands::tab_complete(&self.command_input, &fish_names, &consumable_names)
                    {
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
                self.tank.resize(w, h.saturating_sub(self.bar_height()));
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
                    let name = title_case(&state.name_input);
                    let mut rng = rand::rng();
                    self.tank.place_fish(state.fish.unwrap(), name, &mut rng);
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
                self.tank.money += cv.amount();
            }
            LootKind::Food(amount) => {
                self.tank.food_supply += amount;
            }
            LootKind::Junk(_) => {
                self.tank.add_to_inventory("Junk");
            }
            LootKind::Consumable(kind) => {
                let name = match kind {
                    ConsumableKind::Coffee => "Coffee",
                    ConsumableKind::Bait => "Bait",
                };
                self.tank.add_to_inventory(name);
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
                        self.tank.consume(kind);
                        let entry = self.tank.inventory.entry(name).or_insert(0);
                        *entry = entry.saturating_sub(1);
                        self.tank.inventory.retain(|_, v| *v > 0);
                        if let Some(ref mut state) = self.inventory_state {
                            state.update_from(&self.tank.inventory, &mut rand::rng());
                            if state.items.is_empty() {
                                self.inventory_state = None;
                            }
                        }
                        self.tank.resize(
                            self.terminal_width,
                            self.terminal_height.saturating_sub(self.bar_height()),
                        );
                    }
                }
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

        let money = self.tank.money;
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
                            // Can't navigate to Buy with no money
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
                            }
                        }
                        _ => {
                            let fish: Vec<(String, FishSpecies)> = self
                                .tank
                                .fish
                                .iter()
                                .map(|f| (f.name.clone(), f.species))
                                .collect();
                            if let Some(sm) = SellMenuState::new(&fish, &self.tank.inventory) {
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
                            if money >= cost {
                                match popup.option_idx {
                                    1 => {
                                        *self
                                            .tank
                                            .inventory
                                            .entry("Coffee".to_string())
                                            .or_insert(0) += qty;
                                    }
                                    2 => {
                                        *self
                                            .tank
                                            .inventory
                                            .entry("Bait".to_string())
                                            .or_insert(0) += qty;
                                    }
                                    _ => {
                                        self.tank.food_supply += qty;
                                    }
                                }
                                self.tank.money = self.tank.money.saturating_sub(cost);
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
                            let name = title_case(&name_input);
                            self.tank.money = self.tank.money.saturating_sub(price);
                            let mut rng = rand::rng();
                            self.tank.place_fish(fish, name, &mut rng);
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
                            {
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
                                    if let Some(pos) =
                                        self.tank.fish.iter().position(|f| f.name == fish_name)
                                    {
                                        self.tank.used_names.remove(&fish_name);
                                        self.tank.fish.remove(pos);
                                    }
                                }
                                SellEntry::Junk { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty =
                                        self.tank.inventory.entry("Junk".to_string()).or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Coffee { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty = self
                                        .tank
                                        .inventory
                                        .entry("Coffee".to_string())
                                        .or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                                SellEntry::Bait { .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty =
                                        self.tank.inventory.entry("Bait".to_string()).or_insert(0);
                                    *qty = qty.saturating_sub(sell_qty);
                                }
                            }
                            self.tank.inventory.retain(|_, v| *v > 0);
                            self.tank.money += earned;

                            let fish: Vec<(String, FishSpecies)> = self
                                .tank
                                .fish
                                .iter()
                                .map(|f| (f.name.clone(), f.species))
                                .collect();
                            match SellMenuState::new(&fish, &self.tank.inventory) {
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

    fn bar_height(&self) -> u16 {
        command_bar::height(
            self.settings.show_stats,
            self.terminal_width,
            &self.tank.active_consumables,
            self.tank.money,
            self.tank.food_supply,
            self.tank.fish.len(),
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

    pub fn draw(&mut self, frame: &mut Frame) {
        let full_area = frame.area();
        self.terminal_height = full_area.height;
        self.terminal_width = full_area.width;

        let [tank_area, command_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(self.bar_height())])
                .areas(full_area);

        self.tank.resize(tank_area.width, tank_area.height);

        frame.render_widget(
            TankView::new(&self.tank, self.settings.show_names),
            tank_area,
        );

        let fish_names: Vec<&str> = self.tank.fish.iter().map(|f| f.name.as_str()).collect();
        let consumable_names: Vec<&str> = ["coffee", "bait"]
            .iter()
            .filter(|&&n| {
                let cap = n[..1].to_uppercase() + &n[1..];
                self.tank.inventory.get(&cap).copied().unwrap_or(0) > 0
            })
            .copied()
            .collect();
        let ghost = commands::autocomplete(&self.command_input, &fish_names, &consumable_names)
            .map(|c| c.ghost)
            .unwrap_or_default();
        frame.render_widget(
            CommandBar {
                input: &self.command_input,
                cursor_pos: self.cursor_pos,
                cursor_visible: self.cursor_visible,
                ghost: &ghost,
                fish_count: self.tank.fish.len(),
                food_supply: self.tank.food_supply,
                money: self.tank.money,
                show_stats: self.settings.show_stats,
                active_consumables: &self.tank.active_consumables,
            },
            command_area,
        );

        if let Some(ref state) = self.index_state {
            frame.render_widget(IndexOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.inventory_state {
            frame.render_widget(InventoryOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.fishing_state {
            frame.render_widget(FishingOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.catch_state {
            frame.render_widget(CatchOverlay::new(state), tank_area);
        }

        if let Some(ref state) = self.shop_state {
            frame.render_widget(ShopOverlay::new(state, self.tank.money), tank_area);
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
                self.tank.feed(n);
            }
            commands::Action::SetFps(fps) => self.settings.fps = fps.clamp(FPS_MIN, FPS_MAX),
            commands::Action::ToggleNames => self.settings.show_names = !self.settings.show_names,
            commands::Action::ToggleStats => {
                self.settings.show_stats = !self.settings.show_stats;
                self.tank.resize(
                    self.terminal_width,
                    self.terminal_height.saturating_sub(self.bar_height()),
                );
            }
            commands::Action::ModResource { name, delta } => match name.to_lowercase().as_str() {
                "money" => {
                    self.tank.money = (self.tank.money as i64 + delta as i64).max(0) as u32;
                }
                "food" => {
                    self.tank.food_supply =
                        (self.tank.food_supply as i64 + delta as i64).max(0) as u32;
                }
                _ => {
                    let current = self.tank.inventory.get(&name).copied().unwrap_or(0);
                    let new_val = (current as i64 + delta as i64).max(0) as u32;
                    if new_val == 0 {
                        self.tank.inventory.remove(&name);
                    } else {
                        self.tank.inventory.insert(name, new_val);
                    }
                }
            },
            commands::Action::Spawn(species, name) => {
                let mut rng = rand::rng();
                self.tank.spawn_fish(species, title_case(&name), &mut rng);
            }
            commands::Action::Mutate(fish_name, mutation_name) => {
                self.tank.apply_named_mutation(&fish_name, &mutation_name);
            }
            commands::Action::Index { all } => {
                self.index_state = Some(IndexState::new(&self.tank.fish, all));
            }
            commands::Action::Inventory => {
                if let Some(state) = InventoryState::new(&self.tank.inventory, &mut rand::rng()) {
                    self.inventory_state = Some(state);
                }
            }
            commands::Action::Shop => {
                let money = self.tank.money;
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
                let available = self.tank.inventory.get(item_name).copied().unwrap_or(0);
                if available == 0 {
                    return;
                }
                self.tank.consume(kind);
                let entry = self
                    .tank
                    .inventory
                    .entry(item_name.to_string())
                    .or_insert(0);
                *entry = entry.saturating_sub(1);
                self.tank.inventory.retain(|_, v| *v > 0);
                self.tank.resize(
                    self.terminal_width,
                    self.terminal_height.saturating_sub(self.bar_height()),
                );
            }
            commands::Action::Fish { no_death, no_fish } => {
                let mut state = FishingState::new();
                state.no_death = no_death;
                state.no_fish = no_fish;
                self.fishing_state = Some(state);
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
