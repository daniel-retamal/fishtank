use std::collections::{HashMap, HashSet};

use rand::RngExt;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    abduction::Abductable,
    commands,
    consumable::{ActiveMilkStatus, CONSUMABLE_STACK_BONUS},
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    loot::{
        ConsumableKind, CowCounts, ItemKind, LootKind, LootPool, StockItem, roll_loot,
        roll_loot_no_fish,
    },
    names,
    settings::Settings,
    tank::{ActiveConsumable, Tank, TankEvent, TankKind},
    ui::{
        catch_overlay::{CatchOverlay, CatchState},
        command_bar::{self, CommandBar},
        consume_picker::{ConsumePickerOverlay, ConsumePickerState},
        fishing_overlay::{FishingOverlay, FishingState},
        fishtanks_overlay::{FishtanksOverlay, FishtanksState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
        line_editor::{CommandHistory, LineEditor},
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

enum Overlay {
    Index(IndexState),
    Show {
        state: ShowState,
        backed_index: Option<Box<IndexState>>,
    },
    Inventory(InventoryState),
    Fishing(FishingState),
    Catch(CatchState),
    Shop(ShopState),
    Fishtanks(FishtanksState),
    ConsumePicker(ConsumePickerState),
    Necronomicon(TextInput),
}

pub struct App {
    pub settings: Settings,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    used_tank_names: HashSet<String>,
    pub cash: u32,
    pub food_supply: u32,
    pub inventory: HashMap<StockItem, u32>,
    pub active_consumables: Vec<ActiveConsumable>,
    pub active_statuses: Vec<ActiveMilkStatus>,
    pub editor: LineEditor,
    pub running: bool,
    history: CommandHistory,
    active_overlay: Option<Overlay>,
    terminal_height: u16,
    terminal_width: u16,
    pub graveyard: Vec<Fish>,
    pub void_ritual: VoidRitualState,
    pub next_prayer: usize,
    pub nothing_stacks: u32,
    pending_ufo_dest: HashMap<String, usize>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let mut rng = rand::rng();
        let initial_ritual_timer = sample_exponential(&mut rng, void_ritual::VOID_RITUAL_MEAN_SECS);

        let initial_name = names::unique_name_in(&HashSet::new(), "Fishtank");
        let mut used_tank_names = HashSet::new();
        used_tank_names.insert(initial_name.clone());
        let mut first_tank = Tank::new(initial_name, TankKind::Base, &[]);

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
                inv.insert(StockItem::COFFEE, 1);
                inv.insert(StockItem::BAIT, 1);
                inv.insert(StockItem::NECRONOMICON, 1);
                inv
            },
            active_consumables: Vec::new(),
            active_statuses: Vec::new(),
            editor: LineEditor::new(),
            running: true,
            history: CommandHistory::new(),
            active_overlay: None,
            terminal_height: TERMINAL_HEIGHT_DEFAULT,
            terminal_width: TERMINAL_WIDTH_DEFAULT,
            graveyard: Vec::new(),
            void_ritual: VoidRitualState::Idle {
                timer: initial_ritual_timer,
            },
            next_prayer: 0,
            nothing_stacks: 0,
            pending_ufo_dest: HashMap::new(),
        }
    }

    fn index_state(&self) -> Option<&IndexState> {
        match &self.active_overlay {
            Some(Overlay::Index(s)) => Some(s),
            _ => None,
        }
    }

    fn index_state_mut(&mut self) -> Option<&mut IndexState> {
        match &mut self.active_overlay {
            Some(Overlay::Index(s)) => Some(s),
            _ => None,
        }
    }

    fn show_state_mut(&mut self) -> Option<&mut ShowState> {
        match &mut self.active_overlay {
            Some(Overlay::Show { state, .. }) => Some(state),
            _ => None,
        }
    }

    fn inventory_state(&self) -> Option<&InventoryState> {
        match &self.active_overlay {
            Some(Overlay::Inventory(s)) => Some(s),
            _ => None,
        }
    }

    fn inventory_state_mut(&mut self) -> Option<&mut InventoryState> {
        match &mut self.active_overlay {
            Some(Overlay::Inventory(s)) => Some(s),
            _ => None,
        }
    }

    fn fishing_state(&self) -> Option<&FishingState> {
        match &self.active_overlay {
            Some(Overlay::Fishing(s)) => Some(s),
            _ => None,
        }
    }

    fn fishing_state_mut(&mut self) -> Option<&mut FishingState> {
        match &mut self.active_overlay {
            Some(Overlay::Fishing(s)) => Some(s),
            _ => None,
        }
    }

    fn catch_state(&self) -> Option<&CatchState> {
        match &self.active_overlay {
            Some(Overlay::Catch(s)) => Some(s),
            _ => None,
        }
    }

    fn catch_state_mut(&mut self) -> Option<&mut CatchState> {
        match &mut self.active_overlay {
            Some(Overlay::Catch(s)) => Some(s),
            _ => None,
        }
    }

    fn consume_picker_state(&self) -> Option<&ConsumePickerState> {
        match &self.active_overlay {
            Some(Overlay::ConsumePicker(s)) => Some(s),
            _ => None,
        }
    }

    fn consume_picker_state_mut(&mut self) -> Option<&mut ConsumePickerState> {
        match &mut self.active_overlay {
            Some(Overlay::ConsumePicker(s)) => Some(s),
            _ => None,
        }
    }

    fn necronomicon_popup(&self) -> Option<&TextInput> {
        match &self.active_overlay {
            Some(Overlay::Necronomicon(input)) => Some(input),
            _ => None,
        }
    }

    fn necronomicon_popup_mut(&mut self) -> Option<&mut TextInput> {
        match &mut self.active_overlay {
            Some(Overlay::Necronomicon(input)) => Some(input),
            _ => None,
        }
    }

    pub fn index_overlay_open(&self) -> bool {
        matches!(self.active_overlay, Some(Overlay::Index(_)))
    }

    pub fn fishtanks_overlay_open(&self) -> bool {
        matches!(self.active_overlay, Some(Overlay::Fishtanks(_)))
    }

    pub fn fishtanks_state_mut(&mut self) -> Option<&mut FishtanksState> {
        match &mut self.active_overlay {
            Some(Overlay::Fishtanks(s)) => Some(s),
            _ => None,
        }
    }

    fn take_catch_state(&mut self) -> Option<CatchState> {
        match self.active_overlay.take() {
            Some(Overlay::Catch(s)) => Some(s),
            other => {
                self.active_overlay = other;
                None
            }
        }
    }

    fn take_shop_state(&mut self) -> Option<ShopState> {
        match self.active_overlay.take() {
            Some(Overlay::Shop(s)) => Some(s),
            other => {
                self.active_overlay = other;
                None
            }
        }
    }

    fn set_overlay(&mut self, overlay: Overlay) {
        self.active_overlay = Some(overlay);
    }

    fn close_overlay(&mut self) {
        self.active_overlay = None;
    }

    fn tank(&self) -> &Tank {
        &self.tanks[self.current_tank]
    }

    fn tank_mut(&mut self) -> &mut Tank {
        &mut self.tanks[self.current_tank]
    }

    fn graveyard_names(&self) -> Vec<String> {
        self.graveyard.iter().map(|f| f.name.clone()).collect()
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

    fn milk_buffs(&self) -> crate::ui::fishing_overlay::MilkBuffs {
        use crate::consumable::MilkStatus;
        let stacks_of = |kind: MilkStatus| -> u32 {
            self.active_statuses
                .iter()
                .filter(|s| s.kind == kind)
                .map(|s| s.stacks)
                .sum()
        };
        crate::ui::fishing_overlay::MilkBuffs {
            visual_calculus: stacks_of(MilkStatus::VisualCalculus),
            volition: stacks_of(MilkStatus::Volition),
            physical_instrument: stacks_of(MilkStatus::PhysicalInstrument),
            reaction_speed: stacks_of(MilkStatus::ReactionSpeed),
        }
    }

    pub fn entity_mut_flags(&self) -> Vec<(String, commands::EntityMutFlags)> {
        use crate::fishes::mutations::Mutatable;
        use crate::fishes::species::{BodyTemplate, FishSpecies};
        use crate::fishes::unfish::UnfishKind;
        let mut out: Vec<(String, commands::EntityMutFlags)> = Vec::new();
        let tank = self.tank();
        for fish in &tank.fish {
            let is_unfish = fish.species == FishSpecies::Unfish;
            let unfish_kind = fish.unfish_state.as_ref().map(|u| u.kind);
            let is_double = fish.mutant.as_ref().is_some_and(|m| m.is_double)
                || unfish_kind.is_some_and(|k| {
                    matches!(k, UnfishKind::Worm)
                        && fish.unfish_state.as_ref().is_some_and(|u| u.worm_is_double)
                });
            let has_glisten = fish
                .mutant
                .as_ref()
                .is_some_and(|m| m.glistening_color.is_some())
                || fish
                    .unfish_state
                    .as_ref()
                    .is_some_and(|u| u.slime_glisten_enabled);
            let allows_tail = !is_unfish && fish.allows_tail_variant();
            let allows_size = match (is_unfish, unfish_kind) {
                (true, Some(UnfishKind::Worm)) => true,
                (true, _) => false,
                (false, _) => match fish.species.config().body {
                    BodyTemplate::Standard(_) | BodyTemplate::Alternating(_, _) => true,
                    BodyTemplate::Fixed { .. } => false,
                },
            };
            let allows_body_variant =
                !is_unfish && !matches!(fish.species.config().body, BodyTemplate::Fixed { .. });
            let allows_mouth =
                !is_unfish && !matches!(fish.species.config().body, BodyTemplate::Fixed { .. });
            out.push((
                fish.name.clone(),
                commands::EntityMutFlags {
                    is_double,
                    has_glisten,
                    allows_tail,
                    allows_size,
                    allows_body_variant,
                    allows_mouth,
                },
            ));
        }
        for cow in &tank.cows {
            let is_double = cow.mutant.is_double;
            let has_glisten = cow.mutant.glistening_color.is_some();
            out.push((
                cow.name.clone(),
                commands::EntityMutFlags {
                    is_double,
                    has_glisten,
                    allows_tail: false,
                    allows_size: true,
                    allows_body_variant: false,
                    allows_mouth: true,
                },
            ));
        }
        out
    }

    fn tank_cow_counts(&self) -> CowCounts {
        use crate::entities::cow::CowVariant;
        let mut c = CowCounts {
            plain: 0,
            chocolate: 0,
            strawberry: 0,
            vanilla: 0,
            alien: 0,
        };
        for cow in &self.tank().cows {
            match cow.variant {
                CowVariant::Brown => c.chocolate += 1,
                CowVariant::WhiteBlack => c.plain += 1,
                CowVariant::Pink => c.strawberry += 1,
                CowVariant::LightYellow => c.vanilla += 1,
                CowVariant::LightGreen => c.alien += 1,
            }
        }
        c
    }

    fn devils_luck(&self) -> u32 {
        if self.tank().kind == TankKind::Hell {
            self.tank().fish.len() as u32
        } else {
            0
        }
    }

    fn consume_item(&mut self, kind: ConsumableKind) {
        let Some(duration) = kind.active_duration_secs() else {
            return;
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

    pub fn try_consume_kind(
        &mut self,
        kind: ConsumableKind,
        source: crate::ui::consume_picker::ConsumePickerSource,
    ) {
        let stock = StockItem::Consumable(kind);
        if self.inventory.get(&stock).copied().unwrap_or(0) == 0 {
            return;
        }
        match kind {
            ConsumableKind::Necronomicon => {
                self.set_overlay(Overlay::Necronomicon(
                    crate::ui::text_input::TextInput::new(),
                ));
            }
            ConsumableKind::Milk(crate::loot::MilkVariant::Plain) => {
                self.apply_plain_milk(stock);
            }
            ConsumableKind::Milk(variant) => {
                self.open_consume_picker(variant, kind.display_name().to_string(), source);
            }
            ConsumableKind::Coffee | ConsumableKind::Bait => {
                self.consume_item(kind);
                let entry = self.inventory.entry(stock).or_insert(0);
                *entry = entry.saturating_sub(1);
                self.inventory.retain(|_, v| *v > 0);
                let now_empty = if let Some(Overlay::Inventory(state)) = &mut self.active_overlay {
                    state.update_from(&self.inventory, &mut rand::rng());
                    state.items.is_empty()
                } else {
                    false
                };
                if now_empty {
                    self.close_overlay();
                }
                let bh = self.bar_height();
                let tw = self.terminal_width;
                let th = self.terminal_height.saturating_sub(bh);
                let dead_names = self.graveyard_names();
                self.tanks[self.current_tank].resize(tw, th, &dead_names);
            }
        }
    }

    fn tick_fishing(&mut self) {
        let fps = self.settings.fps;
        let coffee = self.coffee_stacks();
        let milk = self.milk_buffs();
        if let Some(s) = self.fishing_state_mut() {
            s.tick(fps, coffee, milk);
        }
        let (game_over, captured) = match self.fishing_state() {
            Some(s) => (s.game_over, s.captured),
            None => return,
        };
        if game_over {
            self.close_overlay();
            return;
        }
        if !captured {
            return;
        }
        let mut rng = rand::rng();
        let bait = self.bait_stacks();
        let all_tanks_full = self.tanks.iter().all(|t| t.is_full());
        let devils_luck = self.devils_luck();
        let cow_counts = self.tank_cow_counts();
        let in_candy_tank = self.tank().kind == TankKind::Candy;
        let loot = if all_tanks_full {
            roll_loot_no_fish(&mut rng, devils_luck, &cow_counts)
        } else if in_candy_tank {
            LootPool::default_pool()
                .with_candyfish()
                .with_bait(bait)
                .with_devils_luck(devils_luck)
                .with_cows(&cow_counts)
                .roll(&mut rng)
        } else {
            roll_loot(&mut rng, bait, devils_luck, &cow_counts)
        };
        let item_qty = match &loot {
            LootKind::Item(ItemKind::GoldBar) => 0,
            LootKind::Item(item) => {
                StockItem::from_item(item)
                    .and_then(|stock| self.inventory.get(&stock).copied())
                    .unwrap_or(0)
                    + 1
            }
            _ => 0,
        };
        let mut cs = CatchState::new(loot, &mut rng);
        cs.item_qty = item_qty;
        self.set_overlay(Overlay::Catch(cs));
    }

    pub fn tick(&mut self) {
        if matches!(self.active_overlay, Some(Overlay::Fishing(_))) {
            self.tick_fishing();
            return;
        }
        match &mut self.active_overlay {
            Some(Overlay::Catch(catch)) => {
                catch.tick(self.settings.fps);
                return;
            }
            Some(Overlay::Necronomicon(_))
            | Some(Overlay::Inventory(_))
            | Some(Overlay::Fishtanks(_)) => return,
            Some(Overlay::Show { state, .. }) => {
                state.tick_animation(1.0 / self.settings.fps);
                return;
            }
            Some(Overlay::Index(idx)) => {
                idx.tick_animation(1.0 / self.settings.fps);
                return;
            }
            Some(Overlay::Shop(shop)) => {
                shop.tick(self.settings.fps);
                return;
            }
            Some(Overlay::Fishing(_)) => return,
            Some(Overlay::ConsumePicker(_)) | None => {}
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

        for s in &mut self.active_statuses {
            s.time_remaining -= dt;
        }
        self.active_statuses.retain(|s| s.time_remaining > 0.0);

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
                    TankEvent::UfoTimerFired => {
                        self.handle_ufo_timer_fired(i);
                    }
                    TankEvent::UfoLockFish { fish_name } => {
                        if let Some(fish) =
                            self.tanks[i].fish.iter_mut().find(|f| f.name == fish_name)
                        {
                            fish.abduction_lock = true;
                        }
                    }
                    TankEvent::UfoTakeFish { fish_name } => {
                        self.handle_ufo_take_fish(i, &fish_name);
                    }
                    TankEvent::UfoReleaseFish(fish) => {
                        let mut rng = rand::rng();
                        let name = fish.name.clone();
                        self.tanks[i].place_fish(*fish, name, &mut rng);
                    }
                    TankEvent::UfoReleaseCow(cow) => {
                        self.tanks[i].place_cow_dropped(*cow);
                        self.tanks[i].cow_abduction_count =
                            self.tanks[i].cow_abduction_count.saturating_add(1);
                    }
                    TankEvent::UfoFinished => {}
                }
            }
        }
        self.tick_blink();
    }

    fn bar_height(&self) -> u16 {
        command_bar::height(
            self.settings.show_stats,
            self.terminal_width,
            &command_bar::StatsBar {
                active_consumables: &self.active_consumables,
                active_statuses: &self.active_statuses,
                cash: self.cash,
                food_supply: self.food_supply,
                fish_count: self.tank().fish.len(),
                fish_capacity: self.tank().capacity(),
                tank_name: &self.tank().name,
                devils_luck: self.devils_luck(),
            },
        )
    }

    fn tank_height(&self) -> u16 {
        self.terminal_height.saturating_sub(self.bar_height())
    }

    fn show_visible_count(&self) -> usize {
        let Some(Overlay::Show { state, .. }) = &self.active_overlay else {
            return 0;
        };
        ShowState::visible_count(state.overlay_h(self.tank_height(), self.terminal_width))
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

        let dead_names = self.graveyard_names();
        self.tanks[self.current_tank].resize(tank_area.width, tank_area.height, &dead_names);

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
        let fish_names: Vec<&str> = self
            .tank()
            .fish
            .iter()
            .map(|f| f.name.as_str())
            .chain(self.tank().cows.iter().map(|c| c.name.as_str()))
            .collect();
        let owned_consumable_strings: Vec<String> = crate::loot::ConsumableKind::all()
            .iter()
            .filter(|k| {
                self.inventory
                    .get(&StockItem::Consumable(**k))
                    .copied()
                    .unwrap_or(0)
                    > 0
            })
            .map(|k| k.lowercase_name())
            .collect();
        let consumable_names: Vec<&str> = owned_consumable_strings
            .iter()
            .map(String::as_str)
            .collect();
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
        let ghost = if ritual_blocking {
            String::new()
        } else {
            commands::autocomplete(
                &self.editor.text,
                &commands::CompletionCtx {
                    fish_names: &fish_names,
                    consumable_names: &consumable_names,
                    tank_names: &tank_names,
                    current_tank: self.tank().name.as_str(),
                    fish_in_tanks: &fish_in_tanks,
                    has_cow_in_current: self.tank().has_cow(),
                    entity_flags: &entity_flags_slice,
                },
            )
            .map(|c| c.ghost)
            .unwrap_or_default()
        };
        let devils_luck = self.devils_luck();
        frame.render_widget(
            CommandBar {
                input: &self.editor.text,
                cursor_pos: self.editor.cursor,
                cursor_visible: self.editor.visible,
                ghost: &ghost,
                fish_count: self.tank().fish.len(),
                fish_capacity: self.tank().capacity(),
                food_supply: self.food_supply,
                cash: self.cash,
                show_stats: self.settings.show_stats,
                active_consumables: &self.active_consumables,
                active_statuses: &self.active_statuses,
                tank_name: &self.tank().name,
                devils_luck,
            },
            command_area,
        );

        match &self.active_overlay {
            Some(Overlay::Index(state)) => frame.render_widget(IndexOverlay::new(state), tank_area),
            Some(Overlay::Show { state, .. }) => {
                frame.render_widget(ShowOverlay::new(state), tank_area)
            }
            Some(Overlay::Inventory(state)) => {
                frame.render_widget(InventoryOverlay::new(state), tank_area)
            }
            Some(Overlay::Fishtanks(state)) => {
                frame.render_widget(FishtanksOverlay::new(state), tank_area)
            }
            Some(Overlay::ConsumePicker(state)) => {
                frame.render_widget(ConsumePickerOverlay { state }, tank_area)
            }
            Some(Overlay::Fishing(state)) => {
                frame.render_widget(FishingOverlay::new(state), tank_area)
            }
            Some(Overlay::Catch(state)) => frame.render_widget(CatchOverlay::new(state), tank_area),
            Some(Overlay::Shop(state)) => {
                frame.render_widget(ShopOverlay::new(state, self.cash), tank_area)
            }
            Some(Overlay::Necronomicon(input)) => frame.render_widget(
                NecroPopupWidget {
                    input,
                    cursor_visible: self.editor.visible,
                },
                tank_area,
            ),
            None => {}
        }
    }

    fn has_any_overlay(&self) -> bool {
        self.active_overlay.is_some()
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
        self.editor.clear();
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
        self.editor.clear();
    }

    fn handle_ufo_timer_fired(&mut self, source_idx: usize) {
        if self.tanks[source_idx].ufo.is_some() {
            return;
        }
        let mut rng = rand::rng();
        let source_kind = self.tanks[source_idx].kind;
        match source_kind {
            TankKind::Alien => self.plan_cow_delivery(source_idx, &mut rng),
            TankKind::Desert => {
                let night = self.tanks[source_idx]
                    .desert_bg
                    .as_ref()
                    .is_some_and(|s| s.is_night());
                if night {
                    self.plan_abduction(source_idx, &mut rng);
                }
            }
            _ => {}
        }
    }

    fn plan_cow_delivery(&mut self, tank_idx: usize, rng: &mut impl RngExt) {
        use crate::entities::cow::{Cow, CowVariant, random_cow_color};
        use crate::entities::ufo::{UFO_CENTER_COL, Ufo};
        const UFO_BAY_LEFT_EYE_COL: i32 = 7;
        const COW_HEAD_EYE_OFFSET: i32 = 1;
        let variant: CowVariant = random_cow_color(rng);
        let tank = &mut self.tanks[tank_idx];
        let name = tank.unique_cow_name("Vaquita");
        let cow_floor = (tank.height as f32) - (Cow::sprite_height() as f32);
        let probe = Cow::new(name.clone(), variant, 0.0, cow_floor.max(0.0), rng);
        let cow_w = probe.display_width as i32;
        let max_x = (tank.width as i32 - cow_w).max(0);
        let x = pick_cow_drop_x(&tank.cows, cow_w, max_x, rng);
        let cow = Cow {
            position: crate::entities::components::Position {
                x: x as f32,
                y: cow_floor.max(0.0),
            },
            ..probe
        };
        let is_active = tank_idx == self.current_tank;
        if is_active {
            let target_y =
                (tank.height as f32 - crate::entities::ufo::UFO_SPRITE_HEIGHT as f32).max(0.0);
            let ufo_x = x + COW_HEAD_EYE_OFFSET - UFO_BAY_LEFT_EYE_COL;
            let _ = UFO_CENTER_COL;
            tank.ufo = Some(Ufo::new_drop_cow(ufo_x as f32, target_y, cow));
        } else {
            tank.place_cow_dropped(cow);
            tank.cow_abduction_count = tank.cow_abduction_count.saturating_add(1);
        }
    }

    fn plan_abduction(&mut self, source_idx: usize, rng: &mut impl RngExt) {
        use crate::entities::ufo::Ufo;
        if self.tanks[source_idx].kind == TankKind::Alien {
            return;
        }
        let abductable: Vec<usize> = self.tanks[source_idx]
            .fish
            .iter()
            .enumerate()
            .filter(|(_, f)| f.can_be_abducted())
            .map(|(i, _)| i)
            .collect();
        if abductable.is_empty() {
            return;
        }
        let pick = abductable[rng.random_range(0..abductable.len())];
        let fish_name = self.tanks[source_idx].fish[pick].name.clone();

        let dest_idx = self.choose_abduction_dest(source_idx);
        let dest_idx = match dest_idx {
            Some(i) => i,
            None => self.create_alien_base_tank(rng),
        };

        let source_active = source_idx == self.current_tank;
        let dest_active = dest_idx == self.current_tank;

        if source_idx == dest_idx {
            return;
        }

        if source_active {
            use crate::entities::ufo::{UFO_CENTER_COL, UFO_PAYLOAD_CONE_ROW, UFO_SHIP_ROWS};
            let source = &self.tanks[source_idx];
            let fish_ref = &source.fish[pick];
            let fx =
                fish_ref.position.x + fish_ref.display_width as f32 / 2.0 - UFO_CENTER_COL as f32;
            let target_y =
                (fish_ref.position.y - (UFO_SHIP_ROWS + UFO_PAYLOAD_CONE_ROW) as f32).max(0.0);
            self.tanks[source_idx].ufo = Some(Ufo::new_abduct(fx, target_y, fish_name.clone()));
            self.pending_ufo_dest.insert(fish_name, dest_idx);
        } else if dest_active {
            let fish = self.tanks[source_idx].fish.remove(pick);
            let name = fish.name.clone();
            self.tanks[source_idx].used_names.remove(&name);
            let dest = &self.tanks[dest_idx];
            let max_x = (dest.width as i32 - 20).max(6);
            let x = rng.random_range(5..max_x) as f32;
            let target_y =
                (dest.height as f32 - 6.0 - crate::entities::ufo::UFO_SPRITE_HEIGHT as f32)
                    .max(0.0);
            self.tanks[dest_idx].ufo = Some(Ufo::new_drop_fish(x, target_y, fish));
        } else {
            let fish = self.tanks[source_idx].fish.remove(pick);
            let name = fish.name.clone();
            self.tanks[source_idx].used_names.remove(&name);
            self.tanks[dest_idx].place_fish(fish, name, rng);
        }
    }

    fn handle_ufo_take_fish(&mut self, tank_idx: usize, fish_name: &str) {
        use crate::fishes::mutations::{Mutation, apply_mutation_to_fish};
        let pos = match self.tanks[tank_idx]
            .fish
            .iter()
            .position(|f| f.name == fish_name)
        {
            Some(p) => p,
            None => return,
        };
        let dest_idx = self.pending_ufo_dest.remove(fish_name);
        let mut fish = self.tanks[tank_idx].fish.remove(pos);
        fish.abduction_lock = false;
        let name = fish.name.clone();
        self.tanks[tank_idx].used_names.remove(&name);
        let mut rng = rand::rng();
        let dest = dest_idx.unwrap_or(tank_idx);
        self.tanks[dest].place_fish(fish, name.clone(), &mut rng);
        if self.tanks[dest].kind == TankKind::Alien
            && let Some(p) = self.tanks[dest].fish.iter().position(|f| f.name == name)
        {
            apply_mutation_to_fish(
                &mut self.tanks[dest].fish[p],
                Mutation::Alienation,
                &mut rng,
            );
        }
    }

    fn choose_abduction_dest(&self, source_idx: usize) -> Option<usize> {
        const COW_DELIVERIES_PER_NEW_BASE: u32 = 10;
        for (i, t) in self.tanks.iter().enumerate().rev() {
            if i == source_idx {
                continue;
            }
            if t.kind == TankKind::Alien
                && !t.is_full()
                && t.cow_abduction_count < COW_DELIVERIES_PER_NEW_BASE
            {
                return Some(i);
            }
        }
        None
    }

    fn create_alien_base_tank(&mut self, rng: &mut impl RngExt) -> usize {
        let n = rng.random_range(1000..=9999);
        let base_name = format!("Alien Base #{}", n);
        let name = names::unique_name_in(&self.used_tank_names, &base_name);
        self.used_tank_names.insert(name.clone());
        let mut tank = Tank::new(name, TankKind::Alien, &[]);
        tank.resize(self.terminal_width, self.tank_height(), &[]);
        self.tanks.push(tank);
        self.tanks.len() - 1
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
        self.editor
            .tick_blink(self.settings.fps, self.settings.cursor_blink);
    }

    fn reset_blink(&mut self) {
        self.editor.reset_blink();
        match &mut self.active_overlay {
            Some(Overlay::Shop(s)) => s.reset_blink(),
            Some(Overlay::Catch(s)) => s.reset_blink(),
            _ => {}
        }
    }
}

fn pick_cow_drop_x(
    cows: &[crate::entities::cow::Cow],
    cow_w: i32,
    max_x: i32,
    rng: &mut impl RngExt,
) -> i32 {
    if max_x <= 0 {
        return 0;
    }
    const ATTEMPTS: u32 = 32;
    for _ in 0..ATTEMPTS {
        let candidate = rng.random_range(0..=max_x);
        let overlap = cows.iter().any(|c| {
            let c_left = c.position.x as i32;
            let c_right = c_left + c.display_width as i32;
            let cand_right = candidate + cow_w;
            candidate < c_right && cand_right > c_left
        });
        if !overlap {
            return candidate;
        }
    }
    rng.random_range(0..=max_x)
}
