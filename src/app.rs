use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    commands,
    entities::food,
    entities::species::FishSpecies,
    loot::{LootKind, roll_loot},
    settings::{FPS_MAX, FPS_MIN, Settings},
    tank::Tank,
    ui::{
        catch_overlay::{CatchOverlay, CatchState, title_case},
        command_bar::{self, CommandBar},
        fishing_overlay::{FishingOverlay, FishingState},
        index_overlay::{IndexOverlay, IndexState},
        inventory_overlay::{InventoryOverlay, InventoryState},
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
            self.fishing_state.as_mut().unwrap().tick(self.settings.fps);
            let game_over = self.fishing_state.as_ref().unwrap().game_over;
            let captured = self.fishing_state.as_ref().unwrap().captured;
            if game_over {
                self.fishing_state = None;
            } else if captured {
                let mut rng = rand::rng();
                let loot = roll_loot(&mut rng);
                let junk_qty = if matches!(loot, LootKind::Junk(_)) {
                    self.tank.inventory.get("Junk").copied().unwrap_or(0) + 1
                } else {
                    0
                };
                let mut cs = CatchState::new(loot, &mut rng);
                cs.junk_qty = junk_qty;
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
                    if let Some(new_input) =
                        commands::tab_complete(&self.command_input, &fish_names)
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
                self.tank.resize(
                    w,
                    h.saturating_sub(command_bar::height(self.settings.show_stats)),
                );
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
            _ => {}
        }
    }

    fn tank_height(&self) -> u16 {
        self.terminal_height
            .saturating_sub(command_bar::height(self.settings.show_stats))
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

        let [tank_area, command_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(command_bar::height(self.settings.show_stats)),
        ])
        .areas(full_area);

        self.tank.resize(tank_area.width, tank_area.height);

        frame.render_widget(
            TankView::new(&self.tank, self.settings.show_names),
            tank_area,
        );

        let fish_names: Vec<&str> = self.tank.fish.iter().map(|f| f.name.as_str()).collect();
        let ghost = commands::autocomplete(&self.command_input, &fish_names)
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
            commands::Action::ToggleStats => self.settings.show_stats = !self.settings.show_stats,
            commands::Action::ModFood(delta) => {
                self.tank.food_supply = (self.tank.food_supply as i64 + delta as i64).max(0) as u32;
            }
            commands::Action::ModMoney(delta) => {
                self.tank.money = (self.tank.money as i64 + delta as i64).max(0) as u32;
            }
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
                if let Some(state) = InventoryState::new(&self.tank.inventory) {
                    self.inventory_state = Some(state);
                }
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
