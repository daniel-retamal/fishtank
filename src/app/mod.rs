use std::collections::{HashMap, HashSet};

use rand::RngExt;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    consumable::{ActiveMilkStatus, CONSUMABLE_STACK_BONUS},
    fishes::fish::Fish,
    fishes::species::FishSpecies,
    loot::{ConsumableKind, CowCounts, ItemKind, LootKind, LootPool, roll_loot, roll_loot_no_fish},
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
    pub active_statuses: Vec<ActiveMilkStatus>,
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
    pub consume_picker_state: Option<ConsumePickerState>,
    necronomicon_popup: Option<TextInput>,
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
                inv.insert("Coffee".to_string(), 1);
                inv.insert("Bait".to_string(), 1);
                inv.insert("Necronomicon".to_string(), 1);
                inv
            },
            active_consumables: Vec::new(),
            active_statuses: Vec::new(),
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
            consume_picker_state: None,
            necronomicon_popup: None,
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
        let item_name = kind.display_name().to_string();
        if self.inventory.get(&item_name).copied().unwrap_or(0) == 0 {
            return;
        }
        match kind {
            ConsumableKind::Necronomicon => {
                self.inventory_state = None;
                self.necronomicon_popup = Some(crate::ui::text_input::TextInput::new());
            }
            ConsumableKind::Milk(crate::loot::MilkVariant::Plain) => {
                self.apply_plain_milk(&item_name);
            }
            ConsumableKind::Milk(variant) => {
                self.open_consume_picker(variant, item_name, source);
            }
            ConsumableKind::Coffee | ConsumableKind::Bait => {
                self.consume_item(kind);
                let entry = self.inventory.entry(item_name).or_insert(0);
                *entry = entry.saturating_sub(1);
                self.inventory.retain(|_, v| *v > 0);
                if let Some(ref mut state) = self.inventory_state {
                    state.update_from(&self.inventory, &mut rand::rng());
                    if state.items.is_empty() {
                        self.inventory_state = None;
                    }
                }
                let bh = self.bar_height();
                let tw = self.terminal_width;
                let th = self.terminal_height.saturating_sub(bh);
                let dead_names = self.graveyard_names();
                self.tanks[self.current_tank].resize(tw, th, &dead_names);
            }
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
            let milk = self.milk_buffs();
            self.fishing_state
                .as_mut()
                .unwrap()
                .tick(self.settings.fps, coffee, milk);
            let game_over = self.fishing_state.as_ref().unwrap().game_over;
            let captured = self.fishing_state.as_ref().unwrap().captured;
            if game_over {
                self.fishing_state = None;
            } else if captured {
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
                        self.inventory
                            .get(item.display_name())
                            .copied()
                            .unwrap_or(0)
                            + 1
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
            .filter(|k| self.inventory.get(k.display_name()).copied().unwrap_or(0) > 0)
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
                active_statuses: &self.active_statuses,
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

        if let Some(ref state) = self.consume_picker_state {
            frame.render_widget(ConsumePickerOverlay { state }, tank_area);
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
            || self.consume_picker_state.is_some()
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

    fn handle_ufo_timer_fired(&mut self, source_idx: usize) {
        if self.tanks[source_idx].ufo.is_some() {
            return;
        }
        let mut rng = rand::rng();
        let source_kind = self.tanks[source_idx].kind;
        if source_kind == TankKind::Alien {
            self.plan_cow_delivery(source_idx, &mut rng);
        } else {
            self.plan_abduction(source_idx, &mut rng);
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
            .filter(|(_, f)| f.unfish_state.is_none())
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
