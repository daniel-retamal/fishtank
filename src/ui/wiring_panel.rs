use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::colors::{DARK_GRAY, WHITE};
use crate::fishes::botfish::BotfishState;
use crate::fishes::chip::Chip;
use crate::fishes::parts::{ConfigSpec, OUTPUT_PIN, Part, Pin, PinOwner};
use crate::ui::circuit_overlay::{owner_name, stacked_name};
use crate::ui::hint_bar::HintBar;
use crate::ui::hints::{HINT_ENTER_SAVE, HINT_ESC_CANCEL, HINT_FIELD};
use crate::ui::layout::{FlexItem, Screen, Scroll, Scrollbar, shrink};
use crate::ui::modal::{BORDER, Frame, Modal};
use crate::ui::table;
use crate::ui::text_input::{TextInput, draw_text_cursor};

pub const SCRIPT_SLOTS: usize = 8;

const LISTENS_LABEL: &str = "listens";
const DRIVES_LABEL: &str = "drives";
const PARTS_LABEL: &str = "parts";
const TRIGGER_LABEL: &str = "trigger";
const CONFIG_RULE: &str = "config";
const PINS_RULE: &str = "pins";
const PROGRAM_RULE: &str = "program";
const RULE_JOINS_LEFT: char = '├';
const RULE_JOINS_RIGHT: char = '┤';
const CHANNEL_JOIN: &str = ", ";
const CHANNEL_SPLIT: char = ',';
const PIN_INDENT: &str = "  ";
const LABEL_GAP: u16 = 1;
const EDGE_PAD: u16 = 1;
const CURSOR_PREFIX_W: u16 = 2;
const MIN_INNER_W: u16 = 44;
const MIN_LABEL_W: u16 = 4;
const MIN_VALUE_W: u16 = 8;
const BACKGROUND: Color = Color::Reset;

const LISTENS_FIELD: usize = 0;

enum PanelRow {
    Field { label: String, field: usize },
    Reading { label: String, text: String },
    Rule(String),
}

#[derive(PartialEq, Eq)]
enum Binding {
    Listens,
    Pin(PinOwner, String),
    Config(Part, &'static ConfigSpec),
    Trigger,
    Script,
}

pub struct WiringPanelState {
    pub tank_idx: usize,
    pub fish_name: String,
    pub selected: usize,
    fields: Vec<TextInput>,
    bindings: Vec<Binding>,
    rows: Vec<PanelRow>,
    scroll: Scroll,
}

impl WiringPanelState {
    pub fn new(tank_idx: usize, fish_name: String, bot: &BotfishState) -> Self {
        let mut panel = Self {
            tank_idx,
            fish_name,
            selected: LISTENS_FIELD,
            fields: Vec::new(),
            bindings: Vec::new(),
            rows: Vec::new(),
            scroll: Scroll::default(),
        };
        panel.build_wiring(bot);
        panel.build_parts(bot);
        panel.build_hardware(bot);
        panel.build_program(bot);
        panel
    }

    fn add_field(&mut self, label: String, value: String, binding: Binding) {
        let field = self.fields.len();
        self.fields.push(TextInput::with_value(value));
        self.bindings.push(binding);
        self.rows.push(PanelRow::Field { label, field });
    }

    fn build_wiring(&mut self, bot: &BotfishState) {
        let listened: Vec<&str> = bot.listens().collect();
        self.add_field(
            LISTENS_LABEL.to_string(),
            listened.join(CHANNEL_JOIN),
            Binding::Listens,
        );
        self.add_field(
            DRIVES_LABEL.to_string(),
            wired_channel(bot, &Pin::output()),
            Binding::Pin(PinOwner::Body, OUTPUT_PIN.name.to_string()),
        );
    }

    fn build_parts(&mut self, bot: &BotfishState) {
        let hardware = bot
            .parts()
            .iter()
            .map(|(part, count)| stacked_name(part, count))
            .chain(bot.chip().map(Chip::display_name));
        for (position, text) in hardware.enumerate() {
            let label = if position == 0 {
                PARTS_LABEL.to_string()
            } else {
                String::new()
            };
            self.rows.push(PanelRow::Reading { label, text });
        }
    }

    fn build_hardware(&mut self, bot: &BotfishState) {
        let pins: Vec<Pin> = bot
            .declared_pins()
            .into_iter()
            .filter(|pin| !pin.is_output())
            .collect();
        let owners: Vec<PinOwner> = bot
            .parts()
            .iter()
            .map(|(part, _)| PinOwner::Part(part))
            .chain(std::iter::once(PinOwner::Body))
            .filter(|&owner| {
                !configured_by(owner).is_empty() || pins.iter().any(|pin| pin.owner == owner)
            })
            .collect();
        let name_each = owners.len() > 1;
        for owner in owners {
            if name_each {
                self.rows
                    .push(PanelRow::Rule(owner_name(bot, owner, &self.fish_name)));
            }
            self.add_config(bot, owner, !name_each);
            self.add_pins(bot, owner, &pins, !name_each);
        }
    }

    fn add_config(&mut self, bot: &BotfishState, owner: PinOwner, ruled: bool) {
        let PinOwner::Part(part) = owner else {
            return;
        };
        let specs = part.config();
        if specs.is_empty() {
            return;
        }
        if ruled {
            self.rows.push(PanelRow::Rule(CONFIG_RULE.to_string()));
        }
        let config = bot.config(part);
        for spec in specs {
            self.add_field(
                format!("{}{}", PIN_INDENT, spec.name),
                config.text(spec).to_string(),
                Binding::Config(part, spec),
            );
        }
    }

    fn add_pins(&mut self, bot: &BotfishState, owner: PinOwner, pins: &[Pin], ruled: bool) {
        let owned: Vec<&Pin> = pins.iter().filter(|pin| pin.owner == owner).collect();
        if owned.is_empty() {
            return;
        }
        if ruled {
            self.rows.push(PanelRow::Rule(PINS_RULE.to_string()));
        }
        for pin in owned {
            self.add_field(
                format!("{}{}[{}]", PIN_INDENT, pin.name, pin.width),
                wired_channel(bot, pin),
                Binding::Pin(pin.owner, pin.name.to_string()),
            );
        }
    }

    fn build_program(&mut self, bot: &BotfishState) {
        self.rows.push(PanelRow::Rule(PROGRAM_RULE.to_string()));
        let trigger = if bot.trigger.is_empty() {
            BotfishState::run_trigger(&self.fish_name)
        } else {
            bot.trigger.clone()
        };
        self.add_field(TRIGGER_LABEL.to_string(), trigger, Binding::Trigger);
        for slot in 0..SCRIPT_SLOTS {
            self.add_field(
                (slot + 1).to_string(),
                bot.script.get(slot).cloned().unwrap_or_default(),
                Binding::Script,
            );
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.fields.len() {
            self.selected += 1;
        }
    }

    pub fn selected_input_mut(&mut self) -> &mut TextInput {
        &mut self.fields[self.selected]
    }

    fn value(&self, field: usize) -> &str {
        self.fields[field].value.trim()
    }

    pub fn apply_to(&self, bot: &mut BotfishState) {
        let heard: Vec<String> = bot.listens().map(str::to_string).collect();
        for channel in heard {
            bot.unlisten(&channel);
        }
        let mut trigger = String::new();
        let mut script = Vec::new();
        for (field, binding) in self.bindings.iter().enumerate() {
            let value = self.value(field);
            match binding {
                Binding::Listens => {
                    for channel in value.split(CHANNEL_SPLIT) {
                        bot.listen(channel);
                    }
                }
                Binding::Pin(owner, pin) => {
                    if value.is_empty() {
                        bot.unwire(*owner, pin);
                        continue;
                    }
                    bot.wire(*owner, pin, value);
                }
                Binding::Config(part, spec) => {
                    bot.configure(*part, spec, value);
                }
                Binding::Trigger => trigger = value.to_string(),
                Binding::Script => {
                    if value.is_empty() {
                        continue;
                    }
                    script.push(value.to_string());
                }
            }
        }
        bot.program(trigger, script);
    }

    fn label_w(&self) -> u16 {
        self.rows
            .iter()
            .map(|row| match row {
                PanelRow::Field { label, .. } | PanelRow::Reading { label, .. } => {
                    table::visual_width(label) as u16
                }
                PanelRow::Rule(_) => 0,
            })
            .max()
            .unwrap_or(0)
    }

    fn value_w(&self) -> u16 {
        self.fields
            .iter()
            .map(|field| table::visual_width(&field.value) as u16)
            .max()
            .unwrap_or(0)
    }

    fn selected_row(&self) -> usize {
        self.rows
            .iter()
            .position(|row| matches!(row, PanelRow::Field { field, .. } if *field == self.selected))
            .unwrap_or(0)
    }

    fn shown_rows(&self, room: usize) -> Range<usize> {
        self.scroll
            .follow(&vec![1; self.rows.len()], self.selected_row(), room)
    }

    fn hints(&self, overflowing: bool) -> HintBar {
        HintBar::new(HINT_ESC_CANCEL)
            .counted(
                HINT_FIELD,
                overflowing.then_some((self.selected + 1, self.fields.len())),
            )
            .action(HINT_ENTER_SAVE)
    }
}

fn configured_by(owner: PinOwner) -> &'static [ConfigSpec] {
    match owner {
        PinOwner::Part(part) => part.config(),
        PinOwner::Body => &[],
    }
}

fn wired_channel(bot: &BotfishState, pin: &Pin) -> String {
    bot.pins().channel_of(pin).unwrap_or_default().to_string()
}

pub struct WiringPanel<'a> {
    state: &'a WiringPanelState,
    cursor_visible: bool,
    screen: Screen,
}

impl<'a> WiringPanel<'a> {
    pub fn new(state: &'a WiringPanelState, cursor_visible: bool, screen: Screen) -> Self {
        Self {
            state,
            cursor_visible,
            screen,
        }
    }

    fn columns(state: &WiringPanelState) -> [FlexItem; 2] {
        [
            FlexItem::new(state.label_w() + LABEL_GAP, MIN_LABEL_W),
            FlexItem::new(CURSOR_PREFIX_W + state.value_w(), MIN_VALUE_W).gives_first(),
        ]
    }
}

impl Widget for WiringPanel<'_> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let state = self.state;
        let title = format!(" {} ", state.fish_name);
        let columns = Self::columns(state);
        let natural_w = (EDGE_PAD * 2 + columns.iter().map(|column| column.basis).sum::<u16>())
            .max(MIN_INNER_W);
        let frame = Frame {
            title: &title,
            border: WHITE,
            background: BACKGROUND,
        };
        let content = (natural_w, state.rows.len() as u16);
        let (modal, _) = Modal::open_fitting(buf, self.screen, &frame, content, |overflowing| {
            state.hints(overflowing)
        });
        let body = modal.body;
        if body.height == 0 {
            return;
        }
        let shown = state.shown_rows(body.height as usize);
        Scrollbar {
            x: modal.scrollbar_x(),
            top: body.y,
            height: body.height,
        }
        .draw(buf, shown.clone(), state.rows.len(), WHITE);
        let widths = shrink(&columns, body.width.saturating_sub(EDGE_PAD * 2));
        let label_x = body.x + EDGE_PAD;
        let value_x = label_x + widths[0];
        let geometry = RowGeometry {
            body,
            label_x,
            label_w: widths[0].saturating_sub(LABEL_GAP),
            value_x,
            value_w: body.right().saturating_sub(value_x),
        };
        for (y, index) in (body.y..).zip(shown) {
            draw_row(
                buf,
                state,
                &state.rows[index],
                &geometry,
                y,
                self.cursor_visible,
            );
        }
    }
}

struct RowGeometry {
    body: Rect,
    label_x: u16,
    label_w: u16,
    value_x: u16,
    value_w: u16,
}

fn draw_row(
    buf: &mut Buffer,
    state: &WiringPanelState,
    row: &PanelRow,
    geometry: &RowGeometry,
    y: u16,
    cursor_visible: bool,
) {
    let dim = Style::default().fg(DARK_GRAY).bg(BACKGROUND);
    let label_room = geometry.label_w as usize;
    match row {
        PanelRow::Rule(label) => draw_rule(buf, label, geometry.body, y),
        PanelRow::Reading { label, text } => {
            buf.set_stringn(
                geometry.label_x,
                y,
                table::ellipsize(label, label_room),
                label_room,
                dim,
            );
            buf.set_stringn(
                geometry.value_x,
                y,
                table::ellipsize(text, geometry.value_w as usize),
                geometry.value_w as usize,
                Style::default().fg(WHITE).bg(BACKGROUND),
            );
        }
        PanelRow::Field { label, field } => {
            buf.set_stringn(
                geometry.label_x,
                y,
                table::ellipsize(label, label_room),
                label_room,
                dim,
            );
            draw_text_cursor(
                buf,
                &state.fields[*field],
                cursor_visible && *field == state.selected,
                geometry.value_x,
                y,
                geometry.value_w,
                BACKGROUND,
            );
        }
    }
}

fn draw_rule(buf: &mut Buffer, label: &str, body: Rect, y: u16) {
    let style = Style::default()
        .fg(DARK_GRAY)
        .add_modifier(Modifier::BOLD)
        .bg(BACKGROUND);
    let text = format!("── {} ", label);
    buf.set_stringn(body.x, y, &text, body.width as usize, style);
    for x in body.x + table::visual_width(&text) as u16..body.right() {
        buf[(x, y)].set_char('─').set_style(style);
    }
    let frame = Style::default().fg(WHITE).bg(BACKGROUND);
    buf[(body.x - BORDER, y)]
        .set_char(RULE_JOINS_LEFT)
        .set_style(frame);
    buf[(body.right(), y)]
        .set_char(RULE_JOINS_RIGHT)
        .set_style(frame);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishes::parts::Part;

    const TANK: usize = 0;

    fn panel_for(bot: &BotfishState) -> WiringPanelState {
        WiringPanelState::new(TANK, "Neo".to_string(), bot)
    }

    const DRIVES_FIELD: usize = 1;

    fn field_bound(panel: &WiringPanelState, wanted: &Binding) -> usize {
        panel
            .bindings
            .iter()
            .position(|binding| binding == wanted)
            .expect("the panel binds that field")
    }

    fn first_script_field(panel: &WiringPanelState) -> usize {
        field_bound(panel, &Binding::Script)
    }

    fn typed(panel: &mut WiringPanelState, field: usize, value: &str) {
        panel.selected = field;
        panel.selected_input_mut().value = value.to_string();
    }

    fn labels(panel: &WiringPanelState) -> Vec<String> {
        panel
            .rows
            .iter()
            .map(|row| match row {
                PanelRow::Field { label, .. } | PanelRow::Reading { label, .. } => label.clone(),
                PanelRow::Rule(label) => format!("── {}", label),
            })
            .collect()
    }

    #[test]
    fn a_bare_fish_shows_its_wiring_above_its_program() {
        let panel = panel_for(&BotfishState::new());
        let labels = labels(&panel);
        assert_eq!(labels[0], LISTENS_LABEL);
        assert_eq!(labels[1], DRIVES_LABEL);
        assert_eq!(labels[2], format!("── {}", PROGRAM_RULE));
        assert_eq!(labels[3], TRIGGER_LABEL);
        assert_eq!(labels[4], "1");
    }

    #[test]
    fn a_fish_with_no_parts_has_no_parts_rows_and_no_pins_section() {
        let panel = panel_for(&BotfishState::new());
        assert!(!labels(&panel).iter().any(|label| label == PARTS_LABEL));
        assert!(
            !labels(&panel).iter().any(|label| label.contains(PINS_RULE)),
            "the fish's own output is the drives row, not a pins section"
        );
    }

    #[test]
    fn installed_parts_are_listed_one_per_row_with_their_count() {
        let mut bot = BotfishState::new();
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);
        let panel = panel_for(&bot);
        let labels = labels(&panel);
        assert_eq!(labels[2], PARTS_LABEL);
        assert_eq!(labels[3], "", "a stacked list only labels its first row");
        let readings: Vec<&str> = panel
            .rows
            .iter()
            .filter_map(|row| match row {
                PanelRow::Reading { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(readings, vec!["Inverter Coil", "Delay Spool ×2"]);
    }

    #[test]
    fn the_panel_opens_on_the_channels_the_fish_already_hears() {
        let mut bot = BotfishState::new();
        bot.listen("nripe");
        bot.listen("clkd");
        bot.drive("harvest");
        let panel = panel_for(&bot);
        assert_eq!(panel.value(LISTENS_FIELD), "clkd, nripe");
        assert_eq!(panel.value(DRIVES_FIELD), "harvest");
    }

    #[test]
    fn saving_wires_every_channel_that_was_typed() {
        let mut bot = BotfishState::new();
        let mut panel = panel_for(&bot);
        typed(&mut panel, LISTENS_FIELD, "nripe, clkd");
        typed(&mut panel, DRIVES_FIELD, "harvest");

        panel.apply_to(&mut bot);

        assert!(bot.hears("nripe"));
        assert!(bot.hears("clkd"));
        assert_eq!(bot.drives(), Some("harvest"));
        assert!(bot.is_wired());
    }

    #[test]
    fn a_channel_name_may_contain_spaces_because_only_commas_separate() {
        let mut bot = BotfishState::new();
        let mut panel = panel_for(&bot);
        typed(&mut panel, LISTENS_FIELD, "carry in, bit 0");

        panel.apply_to(&mut bot);

        assert!(bot.hears("carry in"));
        assert!(bot.hears("bit 0"));
        assert_eq!(bot.listens().count(), 2);
    }

    #[test]
    fn clearing_a_field_disconnects_that_wire() {
        let mut bot = BotfishState::new();
        bot.listen("clk");
        bot.drive("q");
        let mut panel = panel_for(&bot);
        let trigger_field = field_bound(&panel, &Binding::Trigger);
        typed(&mut panel, LISTENS_FIELD, "");
        typed(&mut panel, DRIVES_FIELD, "  ");

        panel.apply_to(&mut bot);

        assert_eq!(bot.listens().count(), 0);
        assert_eq!(bot.drives(), None);
        assert!(
            bot.is_wired(),
            "the default trigger is itself a wire, so the fish is still in the circuit"
        );

        typed(&mut panel, trigger_field, "");
        panel.apply_to(&mut bot);

        assert!(!bot.is_wired(), "a stripped fish goes back to being a fish");
    }

    #[test]
    fn rewiring_replaces_what_the_fish_used_to_hear() {
        let mut bot = BotfishState::new();
        bot.listen("old");
        let mut panel = panel_for(&bot);
        typed(&mut panel, LISTENS_FIELD, "new");

        panel.apply_to(&mut bot);

        assert!(
            !bot.hears("old"),
            "the panel is the whole truth, not an addition"
        );
        assert!(bot.hears("new"));
    }

    #[test]
    fn the_trigger_defaults_to_running_the_fish_by_name() {
        let mut bot = BotfishState::new();
        let panel = panel_for(&bot);
        panel.apply_to(&mut bot);
        assert_eq!(bot.trigger, "/run Neo");
    }

    #[test]
    fn the_program_section_still_saves_its_lines_in_order() {
        let mut bot = BotfishState::new();
        let mut panel = panel_for(&bot);
        let first_line = first_script_field(&panel);
        typed(&mut panel, first_line, "/feed 1");
        typed(&mut panel, first_line + 2, "/feed 3");

        panel.apply_to(&mut bot);

        assert_eq!(bot.script, vec!["/feed 1", "/feed 3"]);
    }

    const TARGET_FIELD: &str = "target";
    const THRESHOLD_FIELD: &str = "threshold";

    fn sensing_fish(part: Part) -> BotfishState {
        let mut bot = BotfishState::new();
        bot.install(part);
        bot
    }

    fn config_spec(part: Part, name: &str) -> &'static ConfigSpec {
        part.config()
            .iter()
            .find(|spec| spec.name == name)
            .unwrap_or_else(|| panic!("{part:?} has no {name} field"))
    }

    fn config_field(panel: &WiringPanelState, part: Part, name: &str) -> usize {
        field_bound(panel, &Binding::Config(part, config_spec(part, name)))
    }

    fn configured(bot: &BotfishState, part: Part, name: &str) -> String {
        bot.config(part).text(config_spec(part, name)).to_string()
    }

    #[test]
    fn a_sense_grows_a_config_section_above_its_pins() {
        let panel = panel_for(&sensing_fish(Part::ShoalCounter));
        let labels = labels(&panel);
        let config_at = labels
            .iter()
            .position(|label| label == &format!("── {}", CONFIG_RULE))
            .expect("a configurable part opens a config section");
        let pins_at = labels
            .iter()
            .position(|label| label == &format!("── {}", PINS_RULE))
            .expect("a sense declares pins");
        assert!(config_at < pins_at, "you configure it, then you wire it");
        assert_eq!(labels[config_at + 1], format!("{PIN_INDENT}{TARGET_FIELD}"));
        assert_eq!(
            labels[config_at + 2],
            format!("{PIN_INDENT}{THRESHOLD_FIELD}")
        );
    }

    #[test]
    fn a_config_field_opens_on_the_value_the_part_shipped_with() {
        let panel = panel_for(&sensing_fish(Part::AssayScale));
        let target = config_field(&panel, Part::AssayScale, TARGET_FIELD);
        assert_eq!(panel.value(target), "heaviest");
    }

    #[test]
    fn saving_the_panel_configures_the_part() {
        let mut bot = sensing_fish(Part::AssayScale);
        let mut panel = panel_for(&bot);
        let target = config_field(&panel, Part::AssayScale, TARGET_FIELD);
        typed(&mut panel, target, "richest mutantfish");

        panel.apply_to(&mut bot);

        assert_eq!(
            configured(&bot, Part::AssayScale, TARGET_FIELD),
            "richest mutantfish"
        );
    }

    #[test]
    fn clearing_a_config_field_hands_it_back_to_the_default() {
        let mut bot = sensing_fish(Part::AssayScale);
        let mut panel = panel_for(&bot);
        let target = config_field(&panel, Part::AssayScale, TARGET_FIELD);
        typed(&mut panel, target, "lightest");
        panel.apply_to(&mut bot);
        assert_eq!(configured(&bot, Part::AssayScale, TARGET_FIELD), "lightest");

        let mut panel = panel_for(&bot);
        typed(&mut panel, target, "   ");
        panel.apply_to(&mut bot);

        assert_eq!(configured(&bot, Part::AssayScale, TARGET_FIELD), "heaviest");
    }

    #[test]
    fn two_configurable_parts_each_get_their_own_named_section() {
        let mut bot = BotfishState::new();
        bot.install(Part::ShoalCounter);
        bot.install(Part::GeigerCoil);
        let labels = labels(&panel_for(&bot));

        assert!(
            !labels.contains(&format!("── {}", CONFIG_RULE)),
            "one bare config rule would leave two threshold rows ambiguous"
        );
        assert!(labels.contains(&format!("── {}", Part::ShoalCounter.display_name())));
        assert!(labels.contains(&format!("── {}", Part::GeigerCoil.display_name())));
    }

    const SHORT_TERMINAL_ROWS: usize = 12;

    fn tall_panel() -> WiringPanelState {
        let mut bot = BotfishState::new();
        bot.install(Part::ShoalCounter);
        bot.install(Part::AssayScale);
        panel_for(&bot)
    }

    #[test]
    fn a_panel_that_fits_never_scrolls() {
        let panel = panel_for(&BotfishState::new());
        assert_eq!(panel.shown_rows(panel.rows.len()), 0..panel.rows.len());
    }

    #[test]
    fn a_panel_taller_than_the_terminal_keeps_every_selected_field_on_screen() {
        let mut panel = tall_panel();
        assert!(
            panel.rows.len() > SHORT_TERMINAL_ROWS,
            "the fixture overflows"
        );
        for _ in 0..panel.fields.len() {
            let shown = panel.shown_rows(SHORT_TERMINAL_ROWS);
            let row = panel.selected_row();
            assert!(
                shown.contains(&row),
                "field {} sits on row {row}, outside rows {shown:?}",
                panel.selected
            );
            panel.select_next();
        }
        assert_eq!(
            panel.shown_rows(SHORT_TERMINAL_ROWS),
            panel.rows.len() - SHORT_TERMINAL_ROWS..panel.rows.len(),
            "the last field scrolls the panel all the way down and no further"
        );
    }

    #[test]
    fn each_part_keeps_its_own_threshold() {
        let mut bot = BotfishState::new();
        bot.install(Part::ShoalCounter);
        bot.install(Part::GeigerCoil);
        let mut panel = panel_for(&bot);
        let shoal = config_field(&panel, Part::ShoalCounter, THRESHOLD_FIELD);
        let geiger = config_field(&panel, Part::GeigerCoil, THRESHOLD_FIELD);
        assert_ne!(shoal, geiger, "two parts, two thresholds");
        typed(&mut panel, shoal, "3");
        typed(&mut panel, geiger, "9");

        panel.apply_to(&mut bot);

        assert_eq!(configured(&bot, Part::ShoalCounter, THRESHOLD_FIELD), "3");
        assert_eq!(configured(&bot, Part::GeigerCoil, THRESHOLD_FIELD), "9");
    }

    #[test]
    fn a_fabric_part_adds_no_config_section() {
        let panel = panel_for(&sensing_fish(Part::InverterCoil));
        assert!(
            !labels(&panel).contains(&format!("── {}", CONFIG_RULE)),
            "a coil has nothing to configure"
        );
    }

    #[test]
    fn a_sense_pin_is_wired_to_a_channel_like_any_other() {
        let mut bot = sensing_fish(Part::ShoalCounter);
        let mut panel = panel_for(&bot);
        let count = field_bound(
            &panel,
            &Binding::Pin(PinOwner::Part(Part::ShoalCounter), "count".to_string()),
        );
        typed(&mut panel, count, "shoal");

        panel.apply_to(&mut bot);

        assert_eq!(
            bot.pins().channel(Part::ShoalCounter, "count"),
            Some("shoal")
        );
        assert_eq!(bot.drives(), None, "the count bus is not the fish's output");
    }

    #[test]
    fn two_parts_each_get_a_section_holding_their_config_then_their_pins() {
        let mut bot = BotfishState::new();
        bot.install(Part::Cochlea);
        bot.install(Part::GlyphPanel);
        let labels = labels(&panel_for(&bot));
        let ear = labels
            .iter()
            .position(|label| label == &format!("── {}", Part::Cochlea.display_name()))
            .expect("the Cochlea heads its own section");
        let lcd = labels
            .iter()
            .position(|label| label == &format!("── {}", Part::GlyphPanel.display_name()))
            .expect("the panel heads its own section");
        assert!(ear < lcd, "sections follow the part order");
        assert_eq!(labels[ear + 1], format!("{PIN_INDENT}char[8]"));
        assert_eq!(
            labels[lcd + 1..lcd + 6],
            [
                format!("{PIN_INDENT}width"),
                format!("{PIN_INDENT}height"),
                format!("{PIN_INDENT}char[8]"),
                format!("{PIN_INDENT}write[1]"),
                format!("{PIN_INDENT}clear[1]"),
            ],
            "a part is configured, then wired"
        );
        assert!(!labels.contains(&format!("── {}", PINS_RULE)));
        assert!(!labels.contains(&format!("── {}", CONFIG_RULE)));
    }

    #[test]
    fn two_char_pins_on_one_fish_are_wired_to_two_channels() {
        let mut bot = BotfishState::new();
        bot.install(Part::Cochlea);
        bot.install(Part::GlyphPanel);
        let mut panel = panel_for(&bot);
        let ear = field_bound(
            &panel,
            &Binding::Pin(PinOwner::Part(Part::Cochlea), "char".to_string()),
        );
        let lcd = field_bound(
            &panel,
            &Binding::Pin(PinOwner::Part(Part::GlyphPanel), "char".to_string()),
        );
        typed(&mut panel, ear, "kbd");
        typed(&mut panel, lcd, "lcd");

        panel.apply_to(&mut bot);

        assert_eq!(bot.pins().channel(Part::Cochlea, "char"), Some("kbd"));
        assert_eq!(bot.pins().channel(Part::GlyphPanel, "char"), Some("lcd"));
    }

    #[test]
    fn navigation_stops_at_the_first_and_last_field() {
        let mut panel = panel_for(&BotfishState::new());
        panel.select_prev();
        assert_eq!(panel.selected, LISTENS_FIELD);
        for _ in 0..panel.fields.len() * 2 {
            panel.select_next();
        }
        assert_eq!(panel.selected, panel.fields.len() - 1);
    }
}
