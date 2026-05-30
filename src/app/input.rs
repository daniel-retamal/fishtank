use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use rand::RngExt;

use crate::{
    commands,
    entities::food,
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    loot::{ConsumableKind, ItemKind, LootKind},
    names,
    settings::{FPS_MAX, FPS_MIN},
    tank::{Tank, TankKind},
    ui::{
        fishing_overlay::FishingState,
        fishtanks_overlay::FishtanksState,
        index_overlay::IndexState,
        input_action::{InputAction, classify},
        inventory_overlay::InventoryState,
        shop_overlay::{
            BuyCategoryPopup, BuyTankPopup, FishListState, FishNamePopup, SellConfirm, SellEntry,
            SellMenuState, ShopPage, ShopState, TankListState, buy_cat_first_available,
            buy_cat_next, list_visible_rows,
        },
        show_overlay::{ShowSource, ShowState},
        text_input::TextInput,
    },
    void_ritual::{self, GiveTarget, VoidRitualState, WishAction},
};

use super::App;

impl App {
    pub fn handle_input(&mut self, event: Event) {
        if let Event::Resize(w, h) = event {
            self.terminal_height = h;
            self.terminal_width = w;
            let bh = self.bar_height();
            self.tanks[self.current_tank].resize(w, h.saturating_sub(bh));
            return;
        }
        let is_key_press = matches!(&event, Event::Key(k) if k.kind == KeyEventKind::Press);
        if self.void_ritual.is_blocking() {
            self.handle_void_ritual_input(event);
        } else if self.catch_state.is_some() {
            self.handle_catch_input(event);
        } else if self.necronomicon_popup.is_some() {
            self.handle_necro_popup_input(event);
        } else if self.fishing_state.is_some() {
            self.handle_fishing_input(event);
        } else if self.show_state.is_some() {
            self.handle_show_input(event);
        } else if self.index_state.is_some() {
            self.handle_index_input(event);
        } else if self.inventory_state.is_some() {
            self.handle_inventory_input(event);
        } else if self.fishtanks_state.is_some() {
            self.handle_fishtanks_input(event);
        } else if self.consume_picker_state.is_some() {
            self.handle_consume_picker_input(event);
        } else if self.shop_state.is_some() {
            self.handle_shop_input(event);
        } else {
            self.handle_command_input(event);
        }
        if is_key_press {
            self.reset_blink();
        }
    }

    fn handle_command_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Confirm => {
                let fish_names_for_parse: Vec<&str> = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.fish.iter().map(|f| f.name.as_str()))
                    .chain(
                        self.tanks
                            .iter()
                            .flat_map(|t| t.cows.iter().map(|c| c.name.as_str())),
                    )
                    .collect();
                let tank_names_for_parse: Vec<&str> =
                    self.tanks.iter().map(|t| t.name.as_str()).collect();
                let cmd_action = commands::parse(
                    &self.command_input,
                    &fish_names_for_parse,
                    &tank_names_for_parse,
                );
                if !self.command_input.trim().is_empty() {
                    self.history.push(self.command_input.clone());
                }
                self.apply(cmd_action);
                self.command_input.clear();
                self.cursor_pos = 0;
                self.history_pos = None;
                self.draft.clear();
            }
            InputAction::Up if !self.history.is_empty() => {
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
            }
            InputAction::Down => {
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
            }
            InputAction::Left => {
                self.cursor_pos = prev_char_boundary(&self.command_input, self.cursor_pos);
            }
            InputAction::Right => {
                self.cursor_pos = next_char_boundary(&self.command_input, self.cursor_pos);
            }
            InputAction::Home => {
                self.cursor_pos = 0;
            }
            InputAction::End => {
                self.cursor_pos = self.command_input.len();
            }
            InputAction::Backspace if self.cursor_pos > 0 => {
                let prev = prev_char_boundary(&self.command_input, self.cursor_pos);
                self.command_input.drain(prev..self.cursor_pos);
                self.cursor_pos = prev;
            }
            InputAction::Delete if self.cursor_pos < self.command_input.len() => {
                let next = next_char_boundary(&self.command_input, self.cursor_pos);
                self.command_input.drain(self.cursor_pos..next);
            }
            InputAction::Tab => {
                let fish_names: Vec<&str> = self
                    .tank()
                    .fish
                    .iter()
                    .map(|f| f.name.as_str())
                    .chain(self.tank().cows.iter().map(|c| c.name.as_str()))
                    .collect();
                let consumable_name_strings: Vec<String> = ConsumableKind::all()
                    .iter()
                    .filter(|k| self.inventory.get(k.display_name()).copied().unwrap_or(0) > 0)
                    .map(|k| k.lowercase_name())
                    .collect();
                let consumable_names: Vec<&str> =
                    consumable_name_strings.iter().map(String::as_str).collect();
                let tank_names: Vec<&str> = self.tanks.iter().map(|t| t.name.as_str()).collect();
                let fish_in_tanks: Vec<(&str, &str)> = self
                    .tanks
                    .iter()
                    .flat_map(|t| {
                        t.fish
                            .iter()
                            .map(move |f| (f.name.as_str(), t.name.as_str()))
                            .chain(
                                t.cows
                                    .iter()
                                    .map(move |c| (c.name.as_str(), t.name.as_str())),
                            )
                    })
                    .collect();
                let entity_flags = self.entity_mut_flags();
                let entity_flags_slice: Vec<(&str, commands::EntityMutFlags)> =
                    entity_flags.iter().map(|(n, f)| (n.as_str(), *f)).collect();
                if let Some(new_input) = commands::tab_complete(
                    &self.command_input,
                    &commands::CompletionCtx {
                        fish_names: &fish_names,
                        consumable_names: &consumable_names,
                        tank_names: &tank_names,
                        current_tank: self.tank().name.as_str(),
                        fish_in_tanks: &fish_in_tanks,
                        has_cow_in_current: self.tank().has_cow(),
                        entity_flags: &entity_flags_slice,
                    },
                ) {
                    self.command_input = new_input;
                    self.cursor_pos = self.command_input.len();
                }
            }
            InputAction::Char(c) => {
                self.command_input.insert(self.cursor_pos, c);
                self.cursor_pos += c.len_utf8();
            }
            _ => {}
        }
    }

    fn handle_show_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel | InputAction::Char('q') => {
                if self
                    .show_state
                    .as_ref()
                    .is_some_and(|s| matches!(s.source, ShowSource::FromIndex))
                {
                    self.index_state = self.backed_index_state.take();
                }
                self.show_state = None;
            }
            InputAction::Up => {
                if let Some(ref mut s) = self.show_state {
                    s.scroll_up();
                }
            }
            InputAction::Down => {
                let visible = self.show_visible_count();
                if let Some(ref mut s) = self.show_state {
                    s.scroll_down(visible);
                }
            }
            _ => {}
        }
    }

    fn handle_index_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel | InputAction::Char('q') => {
                self.index_state = None;
            }
            InputAction::Up => {
                if let Some(ref mut s) = self.index_state {
                    s.scroll_up();
                }
            }
            InputAction::Down => {
                let available = (self.tank_height() as usize).saturating_sub(7);
                if let Some(ref mut s) = self.index_state {
                    s.scroll_down(available);
                }
            }
            InputAction::Left => {
                if let Some(ref mut s) = self.index_state {
                    s.scroll_left();
                }
            }
            InputAction::Right => {
                let tw = self.terminal_width;
                if let Some(ref mut s) = self.index_state {
                    s.scroll_right(tw);
                }
            }
            InputAction::Confirm => {
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
        let Some(action) = classify(&event) else {
            return;
        };
        if matches!(action, InputAction::Quit) {
            self.running = false;
            return;
        }

        let is_fish = self.catch_state.as_ref().is_some_and(|s| s.is_fish());

        if !is_fish {
            match action {
                InputAction::Confirm | InputAction::Cancel | InputAction::Char('q') => {
                    if let Some(state) = self.catch_state.take() {
                        self.apply_non_fish_loot(state.loot);
                    }
                }
                _ => {}
            }
            return;
        }

        match action {
            InputAction::Confirm => {
                if self
                    .catch_state
                    .as_ref()
                    .is_some_and(|s| !s.name_input.is_empty())
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
                    s.name_input.handle_action(&action);
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
            LootKind::Item(ItemKind::GoldBar) => {
                self.cash += crate::loot::GOLD_BAR_VALUE;
            }
            LootKind::Item(item) => {
                *self
                    .inventory
                    .entry(item.display_name().to_string())
                    .or_insert(0) += 1;
            }
            LootKind::Fish(_) => {}
        }
    }

    fn handle_inventory_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel | InputAction::Char('q') => {
                self.inventory_state = None;
            }
            InputAction::Up => {
                if let Some(ref mut s) = self.inventory_state {
                    s.scroll_up();
                }
            }
            InputAction::Down => {
                let visible = self.inventory_visible_rows();
                if let Some(ref mut s) = self.inventory_state {
                    s.scroll_down(visible);
                }
            }
            InputAction::Confirm => {
                let selected_name = self
                    .inventory_state
                    .as_ref()
                    .and_then(|s| s.items.get(s.selected))
                    .map(|item| item.name.clone());
                if let Some(name) = selected_name
                    && let Some(kind) = ConsumableKind::all()
                        .into_iter()
                        .find(|k| k.display_name() == name)
                {
                    self.try_consume_kind(
                        kind,
                        crate::ui::consume_picker::ConsumePickerSource::FromInventory,
                    );
                }
            }
            _ => {}
        }
    }

    fn reopen_inventory_on(&mut self, item_name: &str) {
        let mut rng = rand::rng();
        if let Some(mut state) = InventoryState::new(&self.inventory, &mut rng) {
            if let Some(pos) = state.items.iter().position(|i| i.name == item_name) {
                state.selected = pos;
            }
            self.inventory_state = Some(state);
        }
    }

    fn reopen_inventory_on_first(&mut self) {
        let mut rng = rand::rng();
        if let Some(state) = InventoryState::new(&self.inventory, &mut rng) {
            self.inventory_state = Some(state);
        }
    }

    fn build_picker_entries(&self) -> Vec<crate::ui::consume_picker::ConsumePickerEntry> {
        let mut out = Vec::new();
        for (ti, tank) in self.tanks.iter().enumerate() {
            for (fi, fish) in tank.fish.iter().enumerate() {
                if fish.unfish_state.is_some() {
                    continue;
                }
                out.push(crate::ui::consume_picker::ConsumePickerEntry {
                    fish_name: fish.name.clone(),
                    species_display: fish.species.display_name().to_string(),
                    tank_name: tank.name.clone(),
                    tank_idx: ti,
                    fish_idx: fi,
                });
            }
        }
        out
    }

    pub fn open_consume_picker(
        &mut self,
        milk: crate::loot::MilkVariant,
        item_name: String,
        source: crate::ui::consume_picker::ConsumePickerSource,
    ) {
        if matches!(milk, crate::loot::MilkVariant::Plain) {
            self.apply_plain_milk(&item_name);
            return;
        }
        let entries = self.build_picker_entries();
        if entries.is_empty() {
            return;
        }
        let remaining = self.inventory.get(&item_name).copied().unwrap_or(0);
        self.inventory_state = None;
        self.consume_picker_state = crate::ui::consume_picker::ConsumePickerState::new(
            milk, item_name, remaining, entries, source,
        );
    }

    pub fn apply_plain_milk(&mut self, item_name: &str) {
        let entry = self.inventory.entry(item_name.to_string()).or_insert(0);
        *entry = entry.saturating_sub(1);
        self.inventory.retain(|_, v| *v > 0);
        let mut rng = rand::rng();
        let pick = crate::consumable::MilkStatus::random(&mut rng);
        if let Some(existing) = self.active_statuses.iter_mut().find(|s| s.kind == pick) {
            existing.stacks += 1;
            existing.time_remaining += crate::consumable::MILK_STATUS_STACK_BONUS;
        } else {
            self.active_statuses
                .push(crate::consumable::ActiveMilkStatus {
                    kind: pick,
                    stacks: 1,
                    time_remaining: crate::consumable::MILK_STATUS_DURATION,
                });
        }
        if let Some(ref mut s) = self.inventory_state {
            s.update_from(&self.inventory, &mut rng);
            if s.items.is_empty() {
                self.inventory_state = None;
            }
        }
    }

    fn handle_consume_picker_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel | InputAction::Char('q') => {
                let restore = self
                    .consume_picker_state
                    .as_ref()
                    .map(|s| (s.source, s.item_name.clone()));
                self.consume_picker_state = None;
                if let Some((source, item_name)) = restore
                    && source == crate::ui::consume_picker::ConsumePickerSource::FromInventory
                {
                    self.reopen_inventory_on(&item_name);
                }
            }
            InputAction::Up => {
                if let Some(ref mut s) = self.consume_picker_state {
                    s.scroll_up();
                }
            }
            InputAction::Down => {
                let area_h = self.tank_height();
                let visible = crate::ui::consume_picker::ConsumePickerOverlay::visible_rows(
                    area_h.clamp(8, 25),
                );
                if let Some(ref mut s) = self.consume_picker_state {
                    s.scroll_down(visible);
                }
            }
            InputAction::Confirm => {
                let (milk, item_name, sel_entry) = {
                    let Some(s) = self.consume_picker_state.as_ref() else {
                        return;
                    };
                    let Some(e) = s.entries.get(s.selected) else {
                        return;
                    };
                    (
                        s.milk,
                        s.item_name.clone(),
                        (e.tank_idx, e.fish_idx, e.fish_name.clone()),
                    )
                };
                let (ti, _, fn_name) = sel_entry;
                let mut rng = rand::rng();
                if let Some(pos) = self.tanks[ti].fish.iter().position(|f| f.name == fn_name) {
                    crate::consumable::apply_milk_to_fish(
                        milk,
                        &mut self.tanks[ti].fish[pos],
                        &mut rng,
                    );
                }
                let entry = self.inventory.entry(item_name).or_insert(0);
                *entry = entry.saturating_sub(1);
                self.inventory.retain(|_, v| *v > 0);
                let remaining = self
                    .inventory
                    .get(
                        self.consume_picker_state
                            .as_ref()
                            .map(|s| s.item_name.as_str())
                            .unwrap_or(""),
                    )
                    .copied()
                    .unwrap_or(0);
                if remaining == 0 {
                    let source = self.consume_picker_state.as_ref().map(|s| s.source);
                    self.consume_picker_state = None;
                    if source == Some(crate::ui::consume_picker::ConsumePickerSource::FromInventory)
                    {
                        self.reopen_inventory_on_first();
                    }
                } else {
                    let new_entries = self.build_picker_entries();
                    if let Some(ref mut s) = self.consume_picker_state {
                        s.refresh_after_consume(remaining, || {
                            new_entries
                                .iter()
                                .map(|e| crate::ui::consume_picker::ConsumePickerEntry {
                                    fish_name: e.fish_name.clone(),
                                    species_display: e.species_display.clone(),
                                    tank_name: e.tank_name.clone(),
                                    tank_idx: e.tank_idx,
                                    fish_idx: e.fish_idx,
                                })
                                .collect()
                        });
                        if s.entries.is_empty() {
                            self.consume_picker_state = None;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_necro_popup_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel => {
                self.necronomicon_popup = None;
                let mut rng = rand::rng();
                if let Some(mut state) = InventoryState::new(&self.inventory, &mut rng) {
                    if let Some(pos) = state.items.iter().position(|i| i.name == "Necronomicon") {
                        state.selected = pos;
                    }
                    self.inventory_state = Some(state);
                }
            }
            InputAction::Confirm => {
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
                    input.handle_action(&action);
                }
            }
        }
    }

    fn handle_fishtanks_input(&mut self, event: Event) {
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Quit => {
                self.running = false;
            }
            InputAction::Cancel | InputAction::Char('q') => {
                self.fishtanks_state = None;
            }
            InputAction::Up => {
                if let Some(ref mut s) = self.fishtanks_state {
                    s.scroll_up();
                }
            }
            InputAction::Down => {
                let visible = self.fishtanks_visible_rows();
                if let Some(ref mut s) = self.fishtanks_state {
                    s.scroll_down(visible);
                }
            }
            InputAction::Confirm => {
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
        let Some(action) = classify(&event) else {
            return;
        };
        if matches!(action, InputAction::Quit) {
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
            ShopPage::Main { ref mut selected } => match action {
                InputAction::Cancel | InputAction::Char('q') => {
                    return;
                }
                InputAction::Up if *selected > 0 => {
                    let new_sel = selected.saturating_sub(1);
                    if new_sel == 0 && cash == 0 {
                    } else {
                        *selected = new_sel;
                    }
                }
                InputAction::Down if *selected < 1 => {
                    *selected += 1;
                }
                InputAction::Confirm => match *selected {
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
                },
                _ => {}
            },

            ShopPage::BuyCategory {
                ref mut selected,
                ref mut buy_popup,
            } => {
                if let Some(popup) = buy_popup {
                    match action {
                        InputAction::Cancel | InputAction::Char('q') => {
                            *buy_popup = None;
                        }
                        InputAction::Left if popup.qty > 1 => {
                            popup.qty -= 1;
                        }
                        InputAction::Right if popup.qty < popup.max_qty => {
                            popup.qty += 1;
                        }
                        InputAction::Confirm => {
                            let qty = popup.qty;
                            let unit_price = match popup.option_idx {
                                1 => ConsumableKind::Coffee.buy_price(),
                                2 => ConsumableKind::Bait.buy_price(),
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
                } else {
                    match action {
                        InputAction::Cancel | InputAction::Char('q') => {
                            shop.page = ShopPage::Main { selected: 0 };
                        }
                        InputAction::Up => {
                            *selected = buy_cat_next(*selected, false, cash);
                        }
                        InputAction::Down => {
                            *selected = buy_cat_next(*selected, true, cash);
                        }
                        InputAction::Confirm => match *selected {
                            0 => {
                                shop.page = ShopPage::BuyFishList(FishListState::new(cash));
                            }
                            4 => {
                                shop.page = ShopPage::BuyTankList(TankListState::new(cash));
                            }
                            idx => {
                                let unit_price = match idx {
                                    1 => ConsumableKind::Coffee.buy_price(),
                                    2 => ConsumableKind::Bait.buy_price(),
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
                        },
                        _ => {}
                    }
                }
            }

            ShopPage::BuyFishList(ref mut fl) => {
                let should_commit = fl.popup.as_ref().is_some_and(|p| {
                    matches!(action, InputAction::Confirm) && !p.name_input.is_empty()
                });

                if should_commit {
                    if let Some(FishNamePopup {
                        fish,
                        name_input,
                        catalog_idx,
                        ..
                    }) = fl.popup.take()
                    {
                        let price = FishSpecies::all_buyable()[catalog_idx].buy_price();
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
                    match action {
                        InputAction::Cancel => {
                            fl.popup = None;
                        }
                        _ => {
                            popup.name_input.handle_action(&action);
                        }
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                match action {
                    InputAction::Cancel | InputAction::Char('q') => {
                        shop.page = ShopPage::BuyCategory {
                            selected: buy_cat_first_available(cash),
                            buy_popup: None,
                        };
                    }
                    InputAction::Up => {
                        fl.scroll_up(cash);
                    }
                    InputAction::Down => {
                        fl.scroll_down(cash, visible);
                    }
                    InputAction::Confirm => {
                        let idx = fl.selected;
                        let species = FishSpecies::all_buyable()[idx];
                        if species.buy_price() <= cash {
                            let mut rng = rand::rng();
                            let fish = Fish::new_for_display(species, &mut rng);
                            fl.popup = Some(FishNamePopup {
                                catalog_idx: idx,
                                fish,
                                name_input: TextInput::new(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            ShopPage::BuyTankList(ref mut tl) => {
                let should_commit = tl.popup.as_ref().is_some_and(|p| {
                    matches!(action, InputAction::Confirm) && !p.name_input.is_empty()
                });

                if should_commit {
                    if let Some(BuyTankPopup {
                        catalog_idx,
                        name_input,
                    }) = tl.popup.take()
                    {
                        let kind = TankKind::all()[catalog_idx];
                        if cash >= kind.buy_price() {
                            let name = names::title_case(name_input.as_str());
                            let actual_name = names::unique_name_in(&self.used_tank_names, &name);
                            self.cash = self.cash.saturating_sub(kind.buy_price());
                            self.used_tank_names.insert(actual_name.clone());
                            self.tanks.push(Tank::new(actual_name, kind));
                            self.current_tank = self.tanks.len() - 1;
                        }
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                if let Some(ref mut popup) = tl.popup {
                    match action {
                        InputAction::Cancel => {
                            tl.popup = None;
                        }
                        _ => {
                            popup.name_input.handle_action(&action);
                        }
                    }
                    self.shop_state = Some(shop);
                    return;
                }

                match action {
                    InputAction::Cancel | InputAction::Char('q') => {
                        shop.page = ShopPage::BuyCategory {
                            selected: 4,
                            buy_popup: None,
                        };
                    }
                    InputAction::Up => {
                        tl.scroll_up(cash);
                    }
                    InputAction::Down => {
                        tl.scroll_down(cash, visible);
                    }
                    InputAction::Confirm => {
                        let idx = tl.selected;
                        if TankKind::all()[idx].buy_price() <= cash {
                            tl.popup = Some(BuyTankPopup {
                                catalog_idx: idx,
                                name_input: TextInput::new(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            ShopPage::Sell(ref mut sm) => {
                if sm.confirm.is_some() {
                    let max_qty = sm.items[sm.confirm.as_ref().unwrap().item_idx].max_qty();
                    match action {
                        InputAction::Cancel | InputAction::Char('q') => {
                            sm.confirm = None;
                        }
                        InputAction::Left => {
                            sm.confirm.as_mut().unwrap().qty_down();
                        }
                        InputAction::Right => {
                            sm.confirm.as_mut().unwrap().qty_up(max_qty);
                        }
                        InputAction::Confirm => {
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
                                SellEntry::Milk { name, .. } => {
                                    let sell_qty = confirm.sell_qty;
                                    let qty = self.inventory.entry(name.clone()).or_insert(0);
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
                } else {
                    match action {
                        InputAction::Cancel | InputAction::Char('q') => {
                            shop.page = ShopPage::Main { selected: 1 };
                        }
                        InputAction::Up => {
                            sm.scroll_up();
                        }
                        InputAction::Down => {
                            sm.scroll_down(visible);
                        }
                        InputAction::Confirm => {
                            sm.confirm = Some(SellConfirm {
                                item_idx: sm.selected,
                                sell_qty: 1,
                            });
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
                    let mc = f.mutations.as_ref().map_or(0, |mr| mr.count);
                    let sv = f.species.sell_value(f.weight_g, f.size_category, mc);
                    (f.name.clone(), f.species, sv)
                })
            })
            .collect();
        let sellable_tanks = self.sellable_tanks_with_price();
        SellMenuState::new(&fish, &self.inventory, &sellable_tanks)
    }

    fn place_purchased_fish(&mut self, fish: Fish, name: String) {
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
            .map(|t| (t.name.clone(), t.kind.sell_price()))
            .collect()
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
        let Some(action) = classify(&event) else {
            return;
        };
        match action {
            InputAction::Char(c) => {
                self.command_input.push(c);
                self.cursor_pos = self.command_input.len();
            }
            InputAction::Backspace if !self.command_input.is_empty() => {
                self.command_input.pop();
                self.cursor_pos = self.command_input.len();
            }
            InputAction::Confirm => self.submit_ritual_input(),
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
                    let action = commands::parse(&input, &fish_names_ref, &tank_names_ref);
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
                let cow_names: Vec<String> = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.cows.iter().map(|c| c.name.clone()))
                    .collect();
                let ctx = void_ritual::WishCtx {
                    fish_names: &fish_names,
                    tank_names: &tank_names,
                    graveyard_names: &graveyard_names,
                    cow_names: &cow_names,
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
                let fish_tank_idx = self
                    .tanks
                    .iter()
                    .position(|t| t.fish.iter().any(|f| f.name == fish_name));
                if let Some(ti) = fish_tank_idx {
                    self.tanks[ti].miracle_mutate_fish(&fish_name, &mutation);
                } else if let Some(ti) = self
                    .tanks
                    .iter()
                    .position(|t| t.cows.iter().any(|c| c.name == fish_name))
                {
                    self.tanks[ti].apply_named_mutation_to_cow(&fish_name, &mutation);
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
                let original_fish = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.fish.iter())
                    .find(|f| f.name == fish_name)
                    .cloned();
                if let Some(orig) = original_fish {
                    let clone_name = format!("{}'s Clone", orig.name);
                    let ct = self.current_tank;
                    if !self.tanks[ct].is_full() {
                        let mut rng = rand::rng();
                        self.tanks[ct].place_fish(orig, clone_name, &mut rng);
                    }
                    return;
                }
                let original_cow = self
                    .tanks
                    .iter()
                    .flat_map(|t| t.cows.iter())
                    .find(|c| c.name == fish_name)
                    .cloned();
                if let Some(mut orig) = original_cow {
                    orig.name = format!("{}'s Clone", orig.name);
                    let ct = self.current_tank;
                    let mut rng = rand::rng();
                    self.tanks[ct].place_cow(orig, &mut rng);
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
            WishAction::Restore { name } => {
                use crate::restore::Restorable;
                for tank in &mut self.tanks {
                    if let Some(fish) = tank.fish.iter_mut().find(|f| f.name == name) {
                        fish.restore();
                        return;
                    }
                    if let Some(cow) = tank.cows.iter_mut().find(|c| c.name == name) {
                        cow.restore();
                        return;
                    }
                }
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
                let un_name = kind.un_name();
                let actual_name = names::unique_name_in(&self.used_tank_names, un_name);
                self.used_tank_names.insert(actual_name.clone());
                self.tanks.push(Tank::new(actual_name, kind));
            }
            GiveTarget::Cow(variant_opt) => {
                let mut rng = rand::rng();
                let variant =
                    variant_opt.unwrap_or_else(|| crate::entities::cow::random_cow_color(&mut rng));
                let ct = self.current_tank;
                self.tanks[ct].spawn_cow(variant, &mut rng);
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
                let ct = self.current_tank;
                if self.tanks[ct]
                    .fish
                    .iter()
                    .any(|f| f.name.eq_ignore_ascii_case(&fish_name))
                {
                    self.tanks[ct].apply_named_mutation(&fish_name, &mutation_name);
                } else {
                    self.tanks[ct].apply_named_mutation_to_cow(&fish_name, &mutation_name);
                }
            }
            commands::Action::Index { all, tank_filter } => {
                if let Some(filter) = tank_filter {
                    let tank_idx = self
                        .tanks
                        .iter()
                        .position(|t| t.name.eq_ignore_ascii_case(&filter));
                    if let Some(idx) = tank_idx {
                        let fish_with_tanks: Vec<(&str, &Fish)> = self.tanks[idx]
                            .fish
                            .iter()
                            .map(|f| (self.tanks[idx].name.as_str(), f))
                            .collect();
                        self.index_state = Some(IndexState::new(&fish_with_tanks, all, false));
                    }
                } else {
                    let fish_with_tanks: Vec<(&str, &Fish)> = self
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
                let normalized: String = name.to_ascii_lowercase().split_whitespace().collect();
                if let Some(kind) = ConsumableKind::all().into_iter().find(|k| {
                    let dn: String = k
                        .display_name()
                        .to_ascii_lowercase()
                        .split_whitespace()
                        .collect();
                    dn == normalized
                }) {
                    self.try_consume_kind(
                        kind,
                        crate::ui::consume_picker::ConsumePickerSource::FromCommand,
                    );
                }
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
                if let Some(source_idx) =
                    (0..self.tanks.len())
                        .filter(|&i| i != target_idx)
                        .find(|&i| {
                            self.tanks[i]
                                .fish
                                .iter()
                                .any(|f| f.name.eq_ignore_ascii_case(&fish))
                        })
                {
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
                    return;
                }
                if let Some(source_idx) =
                    (0..self.tanks.len())
                        .filter(|&i| i != target_idx)
                        .find(|&i| {
                            self.tanks[i]
                                .cows
                                .iter()
                                .any(|c| c.name.eq_ignore_ascii_case(&fish))
                        })
                {
                    if self.tanks[target_idx]
                        .cows
                        .iter()
                        .any(|c| c.name.eq_ignore_ascii_case(&fish))
                    {
                        return;
                    }
                    let cow_pos = self.tanks[source_idx]
                        .cows
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(&fish))
                        .unwrap();
                    let cow_obj = self.tanks[source_idx].cows.remove(cow_pos);
                    let cow_name = cow_obj.name.clone();
                    self.tanks[source_idx].used_cow_names.remove(&cow_name);
                    let mut rng = rand::rng();
                    self.tanks[target_idx].place_cow(cow_obj, &mut rng);
                }
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
            commands::Action::Cowsay(text) => {
                let mut rng = rand::rng();
                let tank = &mut self.tanks[self.current_tank];
                if !tank.cows.is_empty() {
                    let idx = rng.random_range(0..tank.cows.len());
                    tank.cows[idx].say(text);
                }
            }
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
            commands::Action::StartFishAbduction => {
                let mut rng = rand::rng();
                let ct = self.current_tank;
                self.plan_abduction(ct, &mut rng);
            }
            commands::Action::StartCowAbduction => {
                let mut rng = rand::rng();
                let ct = self.current_tank;
                self.plan_cow_delivery(ct, &mut rng);
            }
            commands::Action::Unknown => {}
        }
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
